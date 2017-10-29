
#![allow(unused_imports)]

use std::slice::Iter;
use std::ops::Deref;
use std::collections::HashMap;
use subcortex::Thalamus;
use cmn::{MapStore, CorticalDims};
use map::{AreaScheme, EncoderScheme, LayerMapScheme, LayerScheme, AxonTopology, LayerAddress,
    AxonDomain, AxonTags, AxonSignature};

// [NOTES]:
//
// VentralLateralNucleus -- Inputs from the basal nuclei which includes the
// substantia nigra and the globus pallidus (via the thalamic fasciculus). It
// also has inputs from the cerebellum (dentate nucleus, via the
// dentatothalamic tract). It sends neuronal output to the primary motor
// cortex and premotor cortex
//
// The function of the ventral lateral nucleus is to target efferents
// including the motor cortex, premotor cortex, and supplementary motor
// cortex. Therefore, its function helps the coordination and planning of
// movement. It also plays a role in the learning of movement.

// VentralAnteriorNucleus -- Receives neuronal inputs from the basal ganglia.
// Its main afferent fibres are from the globus pallidus. The efferent fibres
// from this nucleus pass into the premotor cortex for initiation and planning
// of movement.
//
// It helps to function in movement by providing feedback for the outputs of the basal ganglia.



/// A subcortical nucleus.
pub trait SubcorticalNucleus: 'static + Send {
    fn area_name<'a>(&'a self) -> &'a str;
    fn pre_cycle(&mut self, thal: &mut Thalamus);
    fn post_cycle(&mut self, thal: &mut Thalamus);
    fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer>;
}


pub struct SubcorticalNucleusLayer {
    name: &'static str,
    addr: LayerAddress,
    axn_sig: AxonSignature,
    axn_topology: AxonTopology,
    dims: Option<CorticalDims>,
}

impl SubcorticalNucleusLayer {
    pub fn new(name: &'static str, addr: LayerAddress, axn_sig: AxonSignature,
            axn_topology: AxonTopology, dims: Option<CorticalDims>,) -> SubcorticalNucleusLayer {
        SubcorticalNucleusLayer {
            name,
            addr,
            axn_sig,
            axn_topology,
            dims,
        }
    }
    pub fn set_dims(&mut self, dims: Option<CorticalDims>) {
        self.dims = dims;
    }

    pub fn name(&self) -> &'static str { self.name }
    pub fn addr(&self) -> &LayerAddress { &self.addr }
    pub fn axn_sig(&self) -> &AxonSignature { &self.axn_sig }
    pub fn axn_tags(&self) -> &AxonTags { &self.axn_sig.tags() }
    pub fn axn_topology(&self) -> AxonTopology { self.axn_topology.clone() }
    pub fn dims(&self) -> Option<&CorticalDims> { self.dims.as_ref() }
}



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

impl SubcorticalNucleus for TestScNucleus {
    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }

    fn pre_cycle(&mut self, _thal: &mut Thalamus) {
        println!("Pre-cycling!");
    }

    fn post_cycle(&mut self, _thal: &mut Thalamus) {
        println!("Post-cycling!");
    }

    fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer> {
        self.layers.get(&addr)
            // .expect(&format!("SubcorticalNucleus::layer(): Invalid addr: {:?}", addr))
    }
}



pub struct Subcortex {
    nuclei: MapStore<String, Box<SubcorticalNucleus>>,
}

impl Subcortex {
    pub fn new() -> Subcortex {
        Subcortex {
            nuclei: MapStore::with_capacity(8),
        }
    }

    pub fn nucleus<N>(mut self, nucleus: N) -> Subcortex
            where N: SubcorticalNucleus {
        let area_name = nucleus.area_name().to_owned();
        self.add_nucleus(area_name, nucleus);
        self
    }

    pub fn add_nucleus<S, N>(&mut self, area_name: S, nucleus: N)
            where S: Into<String>, N: SubcorticalNucleus {
        self.nuclei.insert(area_name.into(), Box::new(nucleus));

    }

    pub fn pre_cycle(&mut self, thal: &mut Thalamus) {
        for nucleus in self.nuclei.iter_mut() {
            thal.area_maps();
            let _ = nucleus;
        }
    }

    pub fn post_cycle(&mut self, thal: &mut Thalamus) {
        for nucleus in self.nuclei.iter_mut() {
            thal.area_maps();
            let _ = nucleus;
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Box<SubcorticalNucleus>> {
        self.nuclei.iter()
    }
}

impl Deref for Subcortex {
    type Target = MapStore<String, Box<SubcorticalNucleus>>;

    fn deref(&self) -> &Self::Target {
        &self.nuclei
    }
}