
mod cortex;
mod cortical_area;
mod axon_space;
mod synapses;
mod minicolumns;
mod iinn;
mod pyramidals;
mod spiny_stellates;
mod dendrites;
mod sensory_filter;

pub use self::cortex::Cortex;
pub use self::cortical_area::{CorticalArea, CorticalAreas, CorticalAreaSettings};
pub use self::axon_space::AxonSpace;
pub use self::synapses::{Synapses, TuftDims};
pub use self::minicolumns::Minicolumns;
pub use self::iinn::InhibitoryInterneuronNetwork;
pub use self::pyramidals::PyramidalLayer;
pub use self::spiny_stellates::SpinyStellateLayer;
pub use self::dendrites::Dendrites;
pub use self::sensory_filter::SensoryFilter;

#[cfg(test)] pub use self::cortical_area::CorticalAreaTest;
#[cfg(test)] pub use self::synapses::{SynCoords, SynapsesTest, syn_idx};
#[cfg(test)] pub use self::axon_space::{AxonSpaceTest, AxnCoords};
#[cfg(test)] pub use self::dendrites::{DenCoords, DendritesTest, den_idx};
#[cfg(test)] pub use self::minicolumns::MinicolumnsTest;