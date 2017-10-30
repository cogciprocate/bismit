#![allow(unused_imports, dead_code, unused_variables)]

use std::thread::{self, JoinHandle};
use time;
use futures::{Sink, Stream, Future};
use futures::sync::mpsc::{self, Sender};
use tokio_core::reactor::{Core, Remote};
// use cpuprofiler::PROFILER;
use ocl::{self, Platform, Context, Device};
use cortex::{CorticalArea, CorticalAreaSettings};

use cmn::{MapStore, CmnResult};
use map::{LayerMapSchemeList, LayerMapKind, AreaSchemeList};
use subcortex::{Subcortex, Thalamus};


pub struct WorkPool {
    work_tx: Sender<Box<Future<Item=(), Error=()> + Send>>,
    reactor_thread: Option<JoinHandle<()>>,
}

impl WorkPool {
    pub fn new() -> WorkPool {
        let (tx, rx) = mpsc::channel(0);
        // let thread_name = format!("BismitWorkPool_{}", area_name.clone());
        let reactor_thread_name = "bismit_work_pool_reactor_core";

        let core_thread: JoinHandle<_> = thread::Builder::new().name(core_thread_name).spawn(move || {
            // let rx = rx;
            let mut core = Core::new().unwrap();
            let work = rx.buffer_unordered(8).for_each(|_| Ok(()));
            core.run(work).unwrap();
        }).unwrap();

        WorkPool {
            work_tx: tx
            reactor_thread: thread
        }
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

        self.thal.cycle_pathways();
        // self.thal.cycle_input_generators();

        // if let Some(ref mut s) = self.sub {
        //     s.pre_cycle(&mut self.thal)
        // }
        self.sub.pre_cycle(&mut self.thal);

        for area in self.areas.values_mut() {
            area.cycle(&mut self.thal).expect("Cortex::cycle(): Cortical area cycling error");
        }

        // if let Some(ref mut s) = self.sub {
        //     s.post_cycle(&mut self.thal)
        // }
        self.sub.post_cycle(&mut self.thal);

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
