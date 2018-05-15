// #![allow(unused_imports, dead_code, unused_variables)]

// use std::thread::{self, JoinHandle};
use time;
// use futures::{executor, SinkExt, StreamExt, Future};
// use futures::StreamExt;
// use futures::channel::mpsc::{self, Sender};
// use futures_cpupool::{CpuPool, Builder as CpuPoolBuilder};
// use tokio_core::reactor::Core;
#[cfg(feature = "profile")]
use cpuprofiler::PROFILER;
use ocl::{self, Platform, Context, Device};
use cmn::{CmnResult, MapStore};
use cortex::{CorticalArea, CorticalAreaSettings, CompletionPool, /*CompletionPoolRemote*/};
use map::{LayerMapSchemeList, LayerMapKind, AreaSchemeList};
use subcortex::{Subcortex, SubcorticalNucleus, Thalamus};


// This will need to be increased as the amount of work the pool is expected
// to do increases.
const WORK_POOL_BUFFER_SIZE: usize = 32;


pub type CorticalAreas = MapStore<&'static str, CorticalArea>;



// Prints the time it took to start up.
fn print_startup_time(time_start: time::Timespec) {
    let time_elapsed = time::get_time() - time_start;
    let t_sec = time_elapsed.num_seconds();
    let t_ms = time_elapsed.num_milliseconds() - (t_sec * 1000);
    println!("\n\n... Cortex initialized in: {}.{} seconds.", t_sec, t_ms);
}


pub struct Cortex {
    areas: CorticalAreas,
    thal: Thalamus,
    sub: Subcortex,
    completion_pool: CompletionPool,
}

impl Cortex {
    /// Returns a new `CortexBuilder`;
    pub fn builder(layer_map_sl: LayerMapSchemeList, area_sl: AreaSchemeList) -> Builder {
        Builder::new(layer_map_sl, area_sl)
    }

    /// Creates and returns a new `Cortex`;
    pub fn new(layer_map_sl: LayerMapSchemeList, area_sl: AreaSchemeList,
            ca_settings: Option<CorticalAreaSettings>, mut subcortex: Subcortex,
            completion_pool: Option<CompletionPool>) -> CmnResult<Cortex> {
        println!("\nInitializing Cortex... ");
        let time_start = time::get_time();
        let platform = Platform::new(ocl::core::default_platform().unwrap());
        let device_type = ocl::core::default_device_type().unwrap();
        let ocl_context: Context = Context::builder()
            .platform(platform)
            .devices(Device::specifier().type_flags(device_type))
            .build().expect("CorticalArea::new(): ocl_context creation error");

        let mut thal = Thalamus::new(layer_map_sl, area_sl, &subcortex, &ocl_context).unwrap();
        let mut areas = MapStore::new();
        let mut device_idx = 1;

        let area_maps = thal.area_maps().to_owned();

        // Construct cortical areas:
        for area_map in area_maps.values().into_iter().filter(|area_map|
                area_map.layer_map().layer_map_kind() != &LayerMapKind::Subcortical) {
            areas.insert(area_map.area_name(), CorticalArea::new(area_map.clone(),
                device_idx, &ocl_context, ca_settings.clone(), &mut thal)?);
            device_idx += 1;
        }

        // Wire up subcortical pathway channels:
        for nucleus in subcortex.iter_mut() {
            nucleus.create_pathways(&mut thal, &mut areas)?;
        }

        print_startup_time(time_start);

        Ok(Cortex {
            areas: areas,
            thal: thal,
            sub: subcortex,
            completion_pool: completion_pool.unwrap_or(CompletionPool::new(WORK_POOL_BUFFER_SIZE)?),
        })
    }

    pub fn areas(&self) -> &CorticalAreas {
        &self.areas
    }

    pub fn areas_mut(&mut self) -> &mut CorticalAreas {
        &mut self.areas
    }

    pub fn cycle(&mut self) -> CmnResult<()> {
        #[cfg(feature = "profile")]
        PROFILER.lock().unwrap().start("./bismit.profile").unwrap();

        self.thal.cycle_pathways(&mut self.completion_pool);
        // self.thal.cycle_input_generators();

        self.sub.pre_cycle(&mut self.thal, &mut self.areas, &mut self.completion_pool)?;

        for area in self.areas.values_mut() {
            area.cycle(&mut self.thal, &mut self.completion_pool)
                .expect("Cortex::cycle(): Cortical area cycling error");
        }

        self.sub.post_cycle(&mut self.thal, &mut self.areas, &mut self.completion_pool)?;

        #[cfg(feature = "profile")]
        PROFILER.lock().unwrap().stop().unwrap();

        Ok(())
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
    subcortex: Subcortex,
    completion_pool: Option<CompletionPool>,
}

impl Builder {
    pub fn new(layer_maps: LayerMapSchemeList, areas: AreaSchemeList) -> Builder {
        Builder {
            layer_maps,
            areas,
            ca_settings: None,
            subcortex: Subcortex::new(),
            completion_pool: None,
        }
    }

    pub fn get_layer_map_schemes(&self) -> &LayerMapSchemeList {
        &self.layer_maps
    }

    pub fn get_area_schemes(&self) -> &AreaSchemeList {
        &self.areas
    }

    // pub fn get_completion_pool_remote(&self) -> CompletionPoolRemote {
    //     self.completion_pool.remote()
    // }

    pub fn ca_settings(mut self, ca_settings: CorticalAreaSettings) -> Builder {
        self.ca_settings = Some(ca_settings);
        self
    }

    // pub fn sub(mut self, sub: Subcortex) -> Builder {
    //     self.sub = Some(sub);
    //     self
    // }

    pub fn subcortical_nucleus<N>(mut self, nucl: N) -> Builder
            where N: SubcorticalNucleus {
        self.subcortex.add_nucleus(nucl);
        self
    }

    pub fn build(self) -> CmnResult<Cortex> {
        let completion_pool = self.completion_pool.unwrap_or(CompletionPool::new(WORK_POOL_BUFFER_SIZE)?);
        Cortex::new(self.layer_maps, self.areas, self.ca_settings, self.subcortex,
            Some(completion_pool))
    }
}
