use std::collections::{HashMap};
use time;

use super::{CorticalArea, CorticalAreas};
use thalamus::{Thalamus};
use proto::{ProtolayerMaps, ProtoareaMaps, Thalamic};
use ocl::{self, Platform, Context, Device};

pub struct Cortex {
    // AREAS: CURRENTLY PUBLIC FOR DEBUG/TESTING PURPOSES - need a "disable stuff" struct to pass to it
    pub areas: CorticalAreas,
    thal: Thalamus,
}

impl Cortex {
    pub fn new(plmaps: ProtolayerMaps, pamaps: ProtoareaMaps) -> Cortex {
        println!("\nInitializing Cortex... ");
        let time_start = time::get_time();
        let thal = Thalamus::new(plmaps, pamaps);
        let pamaps = thal.area_maps().clone();
        let platform = Platform::new(ocl::core::default_platform().expect("Cortex::new()"));
        let device_type = ocl::core::default_device_type().expect("Cortex::new()");
        // println!("Cortex::new(): device_type: {:?}", device_type);
        let ocl_context: Context = Context::builder()
            .platform(platform)
            .devices(Device::specifier().type_flags(device_type))
            .build().expect("CorticalArea::new(): ocl_context creation error");
        // println!("Cortex::new(): ocl_context.devices(): {:?}", ocl_context.devices());
        let mut areas = HashMap::new();
        let mut device_idx = 1;

        for (&area_name, _) in pamaps.iter().filter(|&(_, pamap)| 
                pamap.lm_kind_tmp() != &Thalamic)
        {    
            areas.insert(area_name, Box::new(CorticalArea::new(thal.area_map(area_name).clone(), 
                device_idx, &ocl_context)));

            device_idx += 1;
        }    

        print_startup_time(time_start);

        Cortex {
            areas: areas,
            thal: thal,
        }
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
        self.thal.cycle_external_tracts(&mut self.areas);

        for (_, area) in self.areas.iter_mut() {
            area.cycle(&mut self.thal);
        }
    }

    pub fn valid_area(&self, area_name: &str) -> bool {
        self.areas.contains_key(area_name)
    }
}

impl Drop for Cortex {
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