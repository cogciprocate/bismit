
mod cortex;
mod cortical_area;
mod axon_space;
mod synapses;
// mod minicolumns;
mod iinn;
mod smoother;
mod pyramidals;
mod spiny_stellates;
mod tufts;
mod dendrites;
mod sensory_filter;
mod data_cell_layer;
mod control_cell_layer;
mod pyr_outputter;
mod intra_column_inhib;
#[cfg(any(test, feature = "eval"))]
mod sampler;
#[cfg(any(test, feature = "eval"))]
mod layer_sampler;
// mod completion_pool;

// pub use ocl_extras::completion_pool::{self, CompletionPool, /*CompletionPoolRemote,*/ CompletionPoolError};
pub use cmn::completion_pool::{CompletionPool, /*CompletionPoolRemote,*/ CompletionPoolError};

pub use self::cortex::{Cortex, CorticalAreas};
pub use self::cortical_area::{CorticalArea, CorticalAreaSettings, SamplerKind, SamplerBufferKind};
pub use self::axon_space::AxonSpace;
pub use self::synapses::{Synapses, TuftDims};
pub use self::iinn::InhibitoryInterneuronNetwork;
pub use self::smoother::ActivitySmoother;
pub use self::pyramidals::PyramidalLayer;
pub use self::spiny_stellates::SpinyStellateLayer;
pub use self::tufts::Tufts;
pub use self::dendrites::Dendrites;
pub use self::sensory_filter::SensoryFilter;
pub use self::data_cell_layer::DataCellLayer;
pub use self::control_cell_layer::{ControlCellLayer, ControlCellLayers};
pub use self::pyr_outputter::PyrOutputter;
pub use self::intra_column_inhib::IntraColumnInhib;
#[cfg(any(test, feature = "eval"))]
pub use self::sampler::{CorticalSampler, FutureCorticalSamples, CorticalSamples, CellSampleIdxs};
#[cfg(any(test, feature = "eval"))]
pub use self::layer_sampler::{CorticalLayerSampler, FutureCorticalLayerSamples, CorticalLayerSamples};

#[cfg(any(test, feature = "eval"))]
pub use self::cortical_area::CorticalAreaTest;
#[cfg(any(test, feature = "eval"))]
pub use self::synapses::{SynCoords, SynapsesTest, syn_idx};
#[cfg(any(test, feature = "eval"))]
pub use self::axon_space::{AxonSpaceTest, AxnCoords};
#[cfg(any(test, feature = "eval"))]
pub use self::dendrites::{DenCoords, DendritesTest, den_idx};
#[cfg(any(test, feature = "eval"))]
pub use self::data_cell_layer::tests::{CelCoords, DataCellLayerTest, DataCellLayerMap, Cell, Tuft, Dendrite};