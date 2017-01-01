mod cortex;
mod area;

pub use self::cortex::Cortex;
pub use self::area::{CorticalArea, CorticalAreas, AxonSpace, Synapses, Minicolumns,
    InhibitoryInterneuronNetwork, PyramidalLayer, SpinyStellateLayer, Dendrites,
    CorticalAreaSettings, SensoryFilter, TuftDims};
#[cfg(test)] pub use self::area::{CorticalAreaTest, SynCoords, SynapsesTest,
    AxonSpaceTest, AxnCoords, DenCoords, DendritesTest, MinicolumnsTest, den_idx, syn_idx};