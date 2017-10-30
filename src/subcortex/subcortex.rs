
#![allow(unused_imports)]

use std::slice::{Iter, IterMut};
use std::ops::Deref;
use std::collections::HashMap;
use subcortex::Thalamus;
use cmn::{MapStore, CorticalDims};
use map::{AreaScheme, EncoderScheme, LayerMapScheme, LayerScheme, AxonTopology, LayerAddress,
    AxonDomain, AxonTags, AxonSignature};
use cortex::WorkPool;

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


pub struct SubcorticalNucleusLayer {
    name: &'static str,
    addr: LayerAddress,
    axon_domain: AxonDomain,
    axon_topology: AxonTopology,
    dims: CorticalDims,
}

impl SubcorticalNucleusLayer {
    pub fn new(name: &'static str, addr: LayerAddress, axon_domain: AxonDomain,
            axon_topology: AxonTopology, dims: CorticalDims)
            -> SubcorticalNucleusLayer {
        SubcorticalNucleusLayer {
            name,
            addr,
            axon_domain,
            axon_topology,
            dims,
        }
    }

    pub fn set_dims(&mut self, dims: CorticalDims) {
        self.dims = dims;
    }

    pub fn name(&self) -> &'static str { self.name }
    pub fn addr(&self) -> &LayerAddress { &self.addr }
    pub fn axon_domain(&self) -> &AxonDomain { &self.axon_domain }
    pub fn axon_topology(&self) -> AxonTopology { self.axon_topology.clone() }
    pub fn dims(&self) -> &CorticalDims { &self.dims }
}



/// A subcortical nucleus.
pub trait SubcorticalNucleus: 'static + Send {
    fn create_pathways(&mut self, thal: &mut Thalamus);
    fn pre_cycle(&mut self, thal: &mut Thalamus, work_pool: &mut WorkPool);
    fn post_cycle(&mut self, thal: &mut Thalamus, work_pool: &mut WorkPool);
    fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer>;
    fn area_name<'a>(&'a self) -> &'a str;
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

    pub fn pre_cycle(&mut self, thal: &mut Thalamus, work_pool: &mut WorkPool) {
        for nucleus in self.nuclei.iter_mut() {
            nucleus.pre_cycle(thal, work_pool);
        }
    }

    pub fn post_cycle(&mut self, thal: &mut Thalamus, work_pool: &mut WorkPool) {
        for nucleus in self.nuclei.iter_mut() {
            nucleus.post_cycle(thal, work_pool);
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Box<SubcorticalNucleus>> {
        self.nuclei.iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, Box<SubcorticalNucleus>> {
        self.nuclei.iter_mut()
    }
}

impl Deref for Subcortex {
    type Target = MapStore<String, Box<SubcorticalNucleus>>;

    fn deref(&self) -> &Self::Target {
        &self.nuclei
    }
}