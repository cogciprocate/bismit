
mod cortex;
mod cortical_area;
mod axon_space;
mod synapses;
mod minicolumns;
mod iinn;
mod pyramidals;
mod spiny_stellates;
mod dendrites;

pub use self::cortex::Cortex;
pub use self::cortical_area::{CorticalArea, CorticalAreas};
pub use self::axon_space::AxonSpace;
pub use self::synapses::Synapses;
pub use self::minicolumns::Minicolumns;
pub use self::iinn::InhibitoryInterneuronNetwork;
pub use self::pyramidals::PyramidalLayer;
pub use self::spiny_stellates::SpinyStellateLayer;
pub use self::dendrites::Dendrites;

#[cfg(test)] pub use self::cortical_area::CorticalAreaTest;
#[cfg(test)] pub use self::synapses::{SynCoords, SynapsesTest};
#[cfg(test)] pub use self::axon_space::{AxonSpaceTest, AxnCoords};
#[cfg(test)] pub use self::dendrites::{DenCoords, DendritesTest, den_idx};