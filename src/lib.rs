//! # Bismit: Biologically Inspired Sensory Motor Interface Tool

// #![feature(discriminant_value)]
// #![feature(conservative_impl_trait)]

extern crate num;
extern crate libc;
extern crate time;
extern crate find_folder;
// extern crate twox_hash;
extern crate rand;

extern crate futures_cpupool;
extern crate tokio_core;
extern crate crossbeam;
extern crate ocl_extras;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate colorify;
#[macro_use]
extern crate failure;
#[cfg(feature = "profile")]
extern crate cpuprofiler;
pub extern crate futures;
pub extern crate ocl;

mod cortex;
mod subcortex;
mod tract_terminal;
pub mod encode;
pub mod flywheel;
pub mod map;
#[macro_use] pub mod cmn;
#[cfg(test)]
pub mod tests;

pub use ocl::Event as OclEvent;
pub use self::cortex::{Cortex, CorticalArea, AxonSpace, Synapses,
    InhibitoryInterneuronNetwork, ActivitySmoother, PyramidalLayer,
    SpinyStellateLayer, Tufts, Dendrites, CorticalAreaSettings, DataCellLayer,
    SamplerKind, SamplerBufferKind, WorkPool, WorkPoolRemote, CorticalAreas,
    CorticalSampler, FutureCorticalSamples, CorticalSamples, CellSampleIdxs};
#[cfg(any(test, feature = "eval"))]
pub use self::cortex::{CorticalAreaTest, SynCoords, SynapsesTest, syn_idx,
    AxonSpaceTest, AxnCoords, DenCoords, DendritesTest, den_idx,
    CelCoords, DataCellLayerTest};
pub use self::subcortex::{Thalamus, Subcortex, SubcorticalNucleus,
    SubcorticalNucleusLayer, TestScNucleus, InputGenerator, InputGeneratorTract,
    InputGeneratorEncoder, InputGeneratorFrame, TractBuffer, TractSender,
    TractReceiver, WriteBuffer, ReadBuffer, FutureSend, FutureRecv,
    /*FutureWriteGuardVec,*/ FutureReadGuardVec,
    /*WriteGuardVec,*/ ReadGuardVec};
pub use self::flywheel::Flywheel;
pub use self::map::{LayerMapSchemeList, AreaSchemeList, AreaMap, AxonTopology,
    LayerAddress};
pub use self::cmn::{util, TractDims, TypeId, CmnError as Error,
    CmnResult as Result, CorticalDims, MapStore};
pub use self::encode::GlyphBuckets;