
#![allow(unused_imports)]

use std::slice::{Iter, IterMut};
use std::vec::IntoIter;
use std::ops::Deref;
use std::collections::HashMap;
use subcortex::Thalamus;
use cmn::{MapStore, CorticalDims, CmnResult};
use map::{AreaScheme, EncoderScheme, LayerMapScheme, LayerScheme, AxonTopology, LayerAddress,
    AxonDomain, AxonTags, AxonSignature};
use cortex::{WorkPool, CorticalArea};

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


/// A subcortical nucleus layer.
///
/// Used primarily when constructing the area/layer maps.
///
#[derive(Clone, Debug)]
pub struct SubcorticalNucleusLayer {
    name: &'static str,
    addr: LayerAddress,
    axon_domain: AxonDomain,
    axon_topology: AxonTopology,
    dims: CorticalDims,
}

impl SubcorticalNucleusLayer {
    /// Returns a new `SubcorticalNucleusLayer`.
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

    /// Sets the dimensions of a layer.
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
    /// Creates thalamic pathways for communication with the thalamus and other
    /// subcortices.
    fn create_pathways(&mut self, thal: &mut Thalamus,
        cortical_areas: &mut MapStore<&'static str, CorticalArea>) -> CmnResult<()>;

    // /// Creates thalamic pathways for communication with the thalamus and other
    // /// subcortices.
    // fn create_pathways<Sn>(s: Box<Sn>, thal: &mut Thalamus,
    //     cortical_areas: &mut MapStore<&'static str, CorticalArea>)
    //     -> Box<SubcorticalNucleus + Send + 'static>;

    /// Is called before the cortex cycles.
    ///
    /// This is where most subcortical processing should typically be
    /// enqueued.
    ///
    /// This must never block the current thread. All work must be sent to the
    /// work pool.
    fn pre_cycle(&mut self, thal: &mut Thalamus, work_pool: &mut WorkPool) -> CmnResult<()>;

    /// Is called after the cortex cycles.
    ///
    /// This must never block the current thread. All work must be sent to the
    /// work pool.
    fn post_cycle(&mut self, thal: &mut Thalamus, work_pool: &mut WorkPool) -> CmnResult<()>;

    /// Returns the layer specified by `addr`.
    fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer>;

    /// Returns the area name.
    fn area_name<'a>(&'a self) -> &'a str;
}


/// A sub-cortex.
pub struct Subcortex {
    nuclei: MapStore<String, Box<SubcorticalNucleus>>,
}

impl Subcortex {
    /// Returns a new `Subcortex`.
    pub fn new() -> Subcortex {
        Subcortex {
            nuclei: MapStore::with_capacity(8),
        }
    }

    pub fn nucleus<N>(mut self, nucleus: N) -> Subcortex
            where N: SubcorticalNucleus {
        // let area_name = nucleus.area_name().to_owned();
        self.add_nucleus(nucleus);
        self
    }

    pub fn add_nucleus<N>(&mut self, nucleus: N)
            where N: SubcorticalNucleus {
        let area_name = nucleus.area_name().to_owned();
        self.nuclei.insert(area_name, Box::new(nucleus));

    }

    pub fn add_boxed_nucleus(&mut self, nucleus: Box<SubcorticalNucleus + 'static>) {
        let area_name = nucleus.area_name().to_owned();
        self.nuclei.insert(area_name, nucleus);

    }

    /// Pre-cycles all subcortical layers (see `SubcorticalNucleusLayer::pre_cycle`).
    pub fn pre_cycle(&mut self, thal: &mut Thalamus, work_pool: &mut WorkPool) -> CmnResult<()> {
        for nucleus in self.nuclei.iter_mut() {
            nucleus.pre_cycle(thal, work_pool)?;
        }
        Ok(())
    }

    /// Post-cycles all subcortical layers (see `SubcorticalNucleusLayer::post_cycle`).
    pub fn post_cycle(&mut self, thal: &mut Thalamus, work_pool: &mut WorkPool) -> CmnResult<()> {
        for nucleus in self.nuclei.iter_mut() {
            nucleus.post_cycle(thal, work_pool)?;
        }
        Ok(())
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, Box<SubcorticalNucleus>> {
        self.nuclei.iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, Box<SubcorticalNucleus>> {
        self.nuclei.iter_mut()
    }

    pub fn into_iter(self) -> IntoIter<Box<SubcorticalNucleus>> {
        self.nuclei.into_iter()
    }
}

impl Deref for Subcortex {
    type Target = MapStore<String, Box<SubcorticalNucleus>>;

    fn deref(&self) -> &Self::Target {
        &self.nuclei
    }
}