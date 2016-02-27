use std::collections::{ HashMap };
use time;

use cortical_area:: { CorticalArea, CorticalAreas };
use thalamus::{ Thalamus };
use proto::{ ProtolayerMaps, ProtoareaMaps, Thalamic, };
use ocl::{ self, Context };

pub struct Cortex {
    // AREAS: CURRENTLY PUBLIC FOR DEBUG/TESTING PURPOSES - need a "disable stuff" struct to pass to it
    pub areas: CorticalAreas,
    thal: Thalamus,
    ocl_context: Context,
}

impl Cortex {
    pub fn new(plmaps: ProtolayerMaps, pamaps: ProtoareaMaps) -> Cortex {
        println!("\nInitializing Cortex... ");
        let time_start = time::get_time();
        let thal = Thalamus::new(plmaps, pamaps);
        let pamaps = thal.area_maps().clone();
        let ocl_context: Context = Context::new_by_index_and_type(None, Some(ocl::core::DEVICE_TYPE_GPU)).expect(
            "CorticalArea::new(): ocl_context creation error");
        let mut areas = HashMap::new();
        let mut device_idx = 0;        

        for (&area_name, _) in pamaps.iter().filter(|&(_, pamap)| 
                pamap.lm_kind_tmp() != &Thalamic)
        {    
            areas.insert(area_name, Box::new(CorticalArea::new(thal.area_map(area_name).clone(), 
                device_idx, &ocl_context)));

            device_idx += 1;
        }    

        // <<<<< MOVE THIS TIMING STUFF ELSEWHERE AND MAKE A FUNCTION FOR IT >>>>>
        let time_elapsed = time::get_time() - time_start;
        let t_sec = time_elapsed.num_seconds();
        let t_ms = time_elapsed.num_milliseconds() - (t_sec * 1000);
        println!("\n\n... Cortex initialized in: {}.{} seconds.", t_sec, t_ms);

        Cortex {
            areas: areas,
            thal: thal,
            ocl_context: ocl_context,
        }
    }
    
    #[inline]
    pub fn area_mut(&mut self, area_name: &str) -> &mut Box<CorticalArea> {
        let emsg = format!("cortex::Cortex::area_mut(): Area: '{}' not found. ", area_name);
        self.areas.get_mut(area_name).expect(&emsg)
    }

    #[inline]
    pub fn area(&self, area_name: &str) -> &Box<CorticalArea> {
        let emsg = format!("cortex::Cortex::area_mut(): Area: '{}' not found. ", area_name);
        self.areas.get(area_name).expect(&emsg)
    }

    pub fn cycle(&mut self) {
        self.thal.cycle_external_ganglions(&mut self.areas);

        for (area_name, area) in self.areas.iter_mut() {
            area.cycle(&mut self.thal);
        }
    }

    #[inline]
    pub fn valid_area(&self, area_name: &str) -> bool {
        self.areas.contains_key(area_name)
    }
}

impl Drop for Cortex {
    fn drop(&mut self) {
        print!("Releasing OpenCL components... ");
        print!("[ Context ]");
        // NOW DONE AUTOMATICALLY:
        // self.ocl_context.release();
        print!(" ...complete. \n");
    }
}
