// [FIXME]: Remove:
#![allow(dead_code)]

use ocl::Context;
use ::{CorticalArea, AreaMap, Thalamus};

pub struct Cerebellum {
    cortex: CorticalArea,

    inverso: bool,

}

impl Cerebellum {
    pub fn new(area_map: AreaMap, device_idx: usize, ocl_context: &Context, thal: &mut Thalamus) -> Cerebellum {
        Cerebellum {
            cortex: CorticalArea::new(area_map, device_idx, ocl_context, None, thal).unwrap(),
            inverso: true,
        }
    }
}

