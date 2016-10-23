use ocl::Context;
use ::{CorticalArea, AreaMap};

pub struct Cerebellum {
    cortex: CorticalArea,
}

impl Cerebellum {
    pub fn new(area_map: AreaMap, device_idx: usize, ocl_context: &Context) -> Cerebellum {
        Cerebellum {
            cortex: CorticalArea::new(area_map, device_idx, ocl_context, None).unwrap(),
        }
    }
}

