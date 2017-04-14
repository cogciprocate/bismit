// use std::collections::{HashMap};
use time;

// use cpuprofiler::PROFILER;

use ocl::{self, Platform, Context, Device};
use cortex::{CorticalArea, CorticalAreaSettings};
use ::{Thalamus};
use cmn::MapStore;
use map::{LayerMapSchemeList, LayerMapKind, AreaSchemeList};
// use cmn::{CmnResult};
// use thalamus::{ExternalPathway, ExternalPathwayFrame};
use subcortex::Subcortex;

pub struct Cortex {
    areas: MapStore<&'static str, CorticalArea>,
    thal: Thalamus,
    sub: Option<Subcortex>,
}

impl Cortex {
    pub fn new(layer_map_sl: LayerMapSchemeList, area_sl: AreaSchemeList,
                ca_settings: Option<CorticalAreaSettings>) -> Cortex {
        println!("\nInitializing Cortex... ");
        let time_start = time::get_time();
        let platform = Platform::new(ocl::core::default_platform().unwrap());
        let device_type = ocl::core::default_device_type().unwrap();
        // println!("Cortex::new(): device_type: {:?}", device_type);
        let ocl_context: Context = Context::builder()
            .platform(platform)
            .devices(Device::specifier().type_flags(device_type))
            .build().expect("CorticalArea::new(): ocl_context creation error");
        // println!("Cortex::new(): ocl_context.devices(): {:?}", ocl_context.devices());
        let mut thal = Thalamus::new(layer_map_sl, area_sl, &ocl_context).unwrap();
        // let area_maps = thal.area_maps().values().clone();
        let mut areas = MapStore::new();
        let mut device_idx = 1;

        let area_maps = thal.area_maps().to_owned();

        for area_map in area_maps.values().into_iter().filter(|area_map|
                area_map.lm_kind_tmp() != &LayerMapKind::Subcortical)
        {
            areas.insert(area_map.area_name(), CorticalArea::new(area_map.clone(),
                device_idx, &ocl_context, ca_settings.clone(), &mut thal).unwrap());
            device_idx += 1;
        }

        print_startup_time(time_start);

        Cortex {
            areas: areas,
            thal: thal,
            sub: None,
        }
    }

    pub fn sub(mut self, sub: Subcortex) -> Cortex {
        self.sub = Some(sub);
        self
    }

    pub fn areas(&self) -> &MapStore<&'static str, CorticalArea> {
        &self.areas
    }

    pub fn areas_mut(&mut self) -> &mut MapStore<&'static str, CorticalArea> {
        &mut self.areas
    }

    pub fn cycle(&mut self) {
        // PROFILER.lock().unwrap().start("./bismit.profile").unwrap();

        self.thal.cycle_external_pathways();

        if let Some(ref mut s) = self.sub {
            s.pre_cycle(&mut self.thal)
        }

        for area in self.areas.values_mut() {
            area.cycle(&mut self.thal).expect("Cortex::cycle(): Cortical area cycling error");
        }

        if let Some(ref mut s) = self.sub {
            s.post_cycle(&mut self.thal)
        }

        // PROFILER.lock().unwrap().stop().unwrap();
    }

    pub fn thal_mut(&mut self) -> &mut Thalamus {
        &mut self.thal
    }

    pub fn thal(&self) -> &Thalamus {
        &self.thal
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


// Prints the time it took to start up.
fn print_startup_time(time_start: time::Timespec) {
    let time_elapsed = time::get_time() - time_start;
    let t_sec = time_elapsed.num_seconds();
    let t_ms = time_elapsed.num_milliseconds() - (t_sec * 1000);
    println!("\n\n... Cortex initialized in: {}.{} seconds.", t_sec, t_ms);
}