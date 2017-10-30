// #![allow(unused_imports, dead_code, unused_variables)]

use std::thread::{self, JoinHandle};
use time;
use futures::{Sink, Stream, Future};
use futures::sync::mpsc::{self, Sender};
use futures_cpupool::{CpuPool, Builder as CpuPoolBuilder};
use tokio_core::reactor::{Core, /*Remote,*/ /*Handle*/};
// use cpuprofiler::PROFILER;
use ocl::{self, Platform, Context, Device};
use cmn::{/*CmnError,*/ CmnResult, MapStore};
use cortex::{CorticalArea, CorticalAreaSettings};
use map::{LayerMapSchemeList, LayerMapKind, AreaSchemeList};
use subcortex::{Subcortex, Thalamus};


const WORK_POOL_BUFFER_SIZE: usize = 32;


pub struct WorkPool {
    cpu_pool: CpuPool,
    reactor_tx: Option<Sender<Box<Future<Item=(), Error=()> + Send>>>,
    _reactor_thread: Option<JoinHandle<()>>,
}

impl WorkPool {
    pub fn new() -> WorkPool {
        let (reactor_tx, reactor_rx) = mpsc::channel(0);
        let reactor_thread_name = "bismit_work_pool_reactor_core".to_owned();

        let reactor_thread: JoinHandle<_> = thread::Builder::new()
                .name(reactor_thread_name).spawn(move || {
            let mut core = Core::new().unwrap();
            let work = reactor_rx.buffer_unordered(WORK_POOL_BUFFER_SIZE).for_each(|_| Ok(()));
            core.run(work).unwrap();
        }).unwrap();

        let cpu_pool = CpuPoolBuilder::new().name_prefix("bismit_work_pool_worker_").create();

        WorkPool {
            cpu_pool,
            reactor_tx: Some(reactor_tx),
            _reactor_thread: Some(reactor_thread),
        }
    }

    pub fn complete(&mut self, future: Box<Future<Item=(), Error=()> + Send>)
            -> CmnResult<()> {
        let tx = self.reactor_tx.take().unwrap();
        self.reactor_tx.get_or_insert(tx.send(future).wait()?);
        Ok(())
    }

    pub fn submit_work(&mut self, work: Box<Future<Item=(), Error=()> + Send>)
            -> CmnResult<()> {
        let future = self.cpu_pool.spawn(work);
        let tx = self.reactor_tx.take().unwrap();
        self.reactor_tx.get_or_insert(tx.send(Box::new(future)).wait()?);
        Ok(())
    }
}

impl Drop for WorkPool {
    fn drop(&mut self) {
        self.reactor_tx.take().unwrap().close().unwrap();
        self._reactor_thread.take().unwrap().join().expect("error joining reactor thread");
    }
}


// Prints the time it took to start up.
fn print_startup_time(time_start: time::Timespec) {
    let time_elapsed = time::get_time() - time_start;
    let t_sec = time_elapsed.num_seconds();
    let t_ms = time_elapsed.num_milliseconds() - (t_sec * 1000);
    println!("\n\n... Cortex initialized in: {}.{} seconds.", t_sec, t_ms);
}


pub struct Cortex {
    areas: MapStore<&'static str, CorticalArea>,
    thal: Thalamus,
    sub: Subcortex,
    work_pool: WorkPool,
}

impl Cortex {
    pub fn builder(layer_map_sl: LayerMapSchemeList, area_sl: AreaSchemeList) -> Builder {
        Builder::new(layer_map_sl, area_sl)
    }

    pub fn new(layer_map_sl: LayerMapSchemeList, area_sl: AreaSchemeList,
            ca_settings: Option<CorticalAreaSettings>, sub_opt: Option<Subcortex>)
            -> CmnResult<Cortex> {
        println!("\nInitializing Cortex... ");
        let time_start = time::get_time();
        let platform = Platform::new(ocl::core::default_platform().unwrap());
        let device_type = ocl::core::default_device_type().unwrap();
        let ocl_context: Context = Context::builder()
            .platform(platform)
            .devices(Device::specifier().type_flags(device_type))
            .build().expect("CorticalArea::new(): ocl_context creation error");

        let mut sub = match sub_opt {
            Some(s) => s,
            None => Subcortex::new(),
        };

        let mut thal = Thalamus::new(layer_map_sl, area_sl, &sub, &ocl_context).unwrap();
        let mut areas = MapStore::new();
        let mut device_idx = 1;

        let area_maps = thal.area_maps().to_owned();

        for area_map in area_maps.values().into_iter().filter(|area_map|
                area_map.lm_kind_tmp() != &LayerMapKind::Subcortical) {
            areas.insert(area_map.area_name(), CorticalArea::new(area_map.clone(),
                device_idx, &ocl_context, ca_settings.clone(), &mut thal).unwrap());
            device_idx += 1;
        }

        // Wire up subcortical pathways (channels):
        for nucleus in sub.iter_mut() {
            nucleus.create_pathways(&mut thal);
        }

        print_startup_time(time_start);

        Ok(Cortex {
            areas: areas,
            thal: thal,
            sub: sub,
            work_pool: WorkPool::new(),
        })
    }

    pub fn areas(&self) -> &MapStore<&'static str, CorticalArea> {
        &self.areas
    }

    pub fn areas_mut(&mut self) -> &mut MapStore<&'static str, CorticalArea> {
        &mut self.areas
    }

    pub fn cycle(&mut self) {
        // PROFILER.lock().unwrap().start("./bismit.profile").unwrap();

        self.thal.cycle_pathways(&mut self.work_pool);
        // self.thal.cycle_input_generators();

        // if let Some(ref mut s) = self.sub {
        //     s.pre_cycle(&mut self.thal)
        // }
        self.sub.pre_cycle(&mut self.thal, &mut self.work_pool);

        for area in self.areas.values_mut() {
            area.cycle(&mut self.thal, &mut self.work_pool)
                .expect("Cortex::cycle(): Cortical area cycling error");
        }

        // if let Some(ref mut s) = self.sub {
        //     s.post_cycle(&mut self.thal)
        // }
        self.sub.post_cycle(&mut self.thal, &mut self.work_pool);

        // PROFILER.lock().unwrap().stop().unwrap();
    }

    pub fn thal_mut(&mut self) -> &mut Thalamus {
        &mut self.thal
    }

    pub fn thal(&self) -> &Thalamus {
        &self.thal
    }

    /// Blocks until all command queues are finished.
    pub fn finish_queues(&self) {
        for area in self.areas.values() {
            area.finish_queues();
        }
    }
}

impl Drop for Cortex {
    /// Just for informational purposes. The context will have "dropped"
    /// (ref count --> 0) when `self.areas` is dropped.
    fn drop(&mut self) {
        print!("Releasing OpenCL components... ");
        print!("[ Context ]");
        print!(" ...complete. \n");
    }
}


pub struct Builder {
    layer_maps: LayerMapSchemeList,
    areas: AreaSchemeList,
    ca_settings: Option<CorticalAreaSettings>,
    sub: Option<Subcortex>,
}

impl Builder {
    pub fn new(layer_maps: LayerMapSchemeList, areas: AreaSchemeList) -> Builder {
        Builder {
            layer_maps,
            areas,
            ca_settings: None,
            sub: None,
        }
    }

    pub fn ca_settings(mut self, ca_settings: CorticalAreaSettings) -> Builder {
        self.ca_settings = Some(ca_settings);
        self
    }

    pub fn sub(mut self, sub: Subcortex) -> Builder {
        self.sub = Some(sub);
        self
    }

    pub fn build(self) -> CmnResult<Cortex> {
        Cortex::new(self.layer_maps, self.areas, self.ca_settings, self.sub)
    }
}
