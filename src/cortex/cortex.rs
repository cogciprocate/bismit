use std::collections::{HashMap};
use time;

use ocl::{self, Platform, Context, Device};
use cortex::{CorticalArea, CorticalAreas, CorticalAreaSettings};
use thalamus::{Thalamus};
use map::{LayerMapSchemeList, LayerMapKind, AreaSchemeList};
// use cmn::{CmnResult};
// use thalamus::{ExternalPathway, ExternalPathwayFrame};
use subcortex::Subcortex;

pub struct Cortex {
    areas: CorticalAreas,
    thal: Thalamus,
    sub: Option<Subcortex>,
}

impl Cortex {
    pub fn new(layer_map_sl: LayerMapSchemeList, area_sl: AreaSchemeList,
                    ca_settings: Option<CorticalAreaSettings>) -> Cortex {
        println!("\nInitializing Cortex... ");
        let time_start = time::get_time();
        let thal = Thalamus::new(layer_map_sl, area_sl).unwrap();
        let area_sl = thal.area_maps().clone();
        let platform = Platform::new(ocl::core::default_platform().unwrap());
        let device_type = ocl::core::default_device_type().unwrap();
        // println!("Cortex::new(): device_type: {:?}", device_type);
        let ocl_context: Context = Context::builder()
            .platform(platform)
            .devices(Device::specifier().type_flags(device_type))
            .build().expect("CorticalArea::new(): ocl_context creation error");
        // println!("Cortex::new(): ocl_context.devices(): {:?}", ocl_context.devices());
        let mut areas = HashMap::new();
        let mut device_idx = 1;

        for (&area_name, _) in area_sl.iter().filter(|&(_, pamap)|
                pamap.lm_kind_tmp() != &LayerMapKind::Subcortical)
        {
            areas.insert(area_name, Box::new(
                // [TODO]: convert unwrap to try (change Cortex sig):
                CorticalArea::new(thal.area_map(area_name).clone(),
                device_idx, &ocl_context, ca_settings.clone()).unwrap()
                ));

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

    pub fn area_mut(&mut self, area_name: &str) -> &mut Box<CorticalArea> {
        let emsg = format!("cortex::Cortex::area_mut(): Area: '{}' not found. ", area_name);
        self.areas.get_mut(area_name).expect(&emsg)
    }

    pub fn area(&self, area_name: &str) -> &Box<CorticalArea> {
        let emsg = format!("cortex::Cortex::area_mut(): Area: '{}' not found. ", area_name);
        self.areas.get(area_name).expect(&emsg)
    }

    pub fn areas(&self) -> &CorticalAreas {
        &self.areas
    }

    pub fn cycle(&mut self) {
        self.thal.cycle_external_pathways(&mut self.areas);

        for (_, area) in self.areas.iter_mut() {
            area.cycle(&mut self.thal).expect("Cortex::cycle(): Cortical area cycling error");
        }
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