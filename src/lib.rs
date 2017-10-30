//! # Bismit: Biologically Inspired Sensory Motor Interface Tool

// #![feature(discriminant_value)]
// #![feature(conservative_impl_trait)]

extern crate num;
extern crate libc;
extern crate time;
extern crate find_folder;
// extern crate twox_hash;
extern crate rand;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;
// extern crate cpuprofiler;
extern crate crossbeam;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate enum_primitive;
#[macro_use] extern crate colorify;
pub extern crate ocl;

mod cortex;
mod subcortex;
mod tract_terminal;
pub mod encode;
pub mod flywheel;
pub mod map;
#[macro_use] pub mod cmn;
#[cfg(test)] pub mod tests;

pub use ocl::Event as OclEvent;
pub use self::cortex::{Cortex, CorticalArea, AxonSpace, Synapses,
    InhibitoryInterneuronNetwork, ActivitySmoother, PyramidalLayer, SpinyStellateLayer,
    Dendrites, CorticalAreaSettings, DataCellLayer, SamplerKind, SamplerBufferKind,
    WorkPool,};
pub use self::subcortex::{Thalamus, Subcortex, SubcorticalNucleus,
    SubcorticalNucleusLayer, TestScNucleus, InputGenerator, InputGeneratorTract,
    InputGeneratorEncoder, InputGeneratorFrame, TractBuffer, TractSender,
    TractReceiver};
pub use self::flywheel::Flywheel;
pub use self::map::{LayerMapSchemeList, AreaSchemeList, AreaMap};
pub use self::cmn::{util, TractDims, TypeId, CmnError as Error, CmnResult as Result};
pub use self::encode::GlyphBuckets;