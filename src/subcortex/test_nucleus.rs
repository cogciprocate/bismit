#![allow(unused_imports)]

use std::slice::Iter;
use std::ops::Deref;
use std::collections::HashMap;
use subcortex::{Thalamus, SubcorticalNucleus, SubcorticalNucleusLayer};
use cmn::{MapStore, CorticalDims};
use map::{AreaScheme, EncoderScheme, LayerMapScheme, LayerScheme, AxonTopology, LayerAddress,
    AxonDomain, AxonTags, AxonSignature};
use ::{WorkPool, CorticalArea};

pub struct TestScNucleus {
    area_name: String,
    layers: HashMap<LayerAddress, SubcorticalNucleusLayer>,
}

impl TestScNucleus {
    pub fn new<'a>(area_name: &'a str) -> TestScNucleus {
        TestScNucleus {
            area_name: area_name.into(),
            layers: HashMap::new(),
        }
    }
}

// impl SubcorticalNucleus for TestScNucleus {
//     fn create_pathways(&mut self, _thal: &mut Thalamus,
//             _cortical_areas: &mut MapStore<&'static str, CorticalArea>) {
//         unimplemented!();
//     }

//     fn area_name<'a>(&'a self) -> &'a str {
//         &self.area_name
//     }

//     fn pre_cycle(&mut self, _thal: &mut Thalamus, _work_pool: &mut WorkPool) {
//         println!("Pre-cycling!");
//     }

//     fn post_cycle(&mut self, _thal: &mut Thalamus, _work_pool: &mut WorkPool) {
//         println!("Post-cycling!");
//     }

//     fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer> {
//         self.layers.get(&addr)
//             // .expect(&format!("SubcorticalNucleus::layer(): Invalid addr: {:?}", addr))
//     }
// }