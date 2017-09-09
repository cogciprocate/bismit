
mod cortex;
mod cortical_area;
mod axon_space;
mod synapses;
// mod minicolumns;
mod iinn;
mod smoother;
mod pyramidals;
mod spiny_stellates;
mod dendrites;
mod sensory_filter;
mod data_cell_layer;
mod control_cell_layer;
mod cortical_sampler;
mod pyr_outputter;

pub use self::cortex::Cortex;
pub use self::cortical_area::{CorticalArea, CorticalAreaSettings, SamplerKind, SamplerBufferKind};
pub use self::axon_space::AxonSpace;
pub use self::synapses::{Synapses, TuftDims};
// pub use self::minicolumns::Minicolumns;
pub use self::iinn::InhibitoryInterneuronNetwork;
pub use self::smoother::ActivitySmoother;
pub use self::pyramidals::PyramidalLayer;
pub use self::spiny_stellates::SpinyStellateLayer;
pub use self::dendrites::Dendrites;
pub use self::sensory_filter::SensoryFilter;
pub use self::data_cell_layer::DataCellLayer;
pub use self::control_cell_layer::ControlCellLayer;
pub use self::pyr_outputter::PyrOutputter;

#[cfg(test)] pub use self::cortical_area::CorticalAreaTest;
#[cfg(test)] pub use self::synapses::{SynCoords, SynapsesTest, syn_idx};
#[cfg(test)] pub use self::axon_space::{AxonSpaceTest, AxnCoords};
#[cfg(test)] pub use self::dendrites::{DenCoords, DendritesTest, den_idx};
// #[cfg(test)] pub use self::minicolumns::MinicolumnsTest;
#[cfg(test)] pub use self::data_cell_layer::tests::{CelCoords, DataCellLayerTest};