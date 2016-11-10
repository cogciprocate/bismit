//! # Bismit: Biologically Inspired Sensory Motor Interface Tool

// #![feature(specialization)]

extern crate num;
extern crate libc;
extern crate time;
extern crate ocl;
extern crate find_folder;
extern crate twox_hash;
#[macro_use] extern crate rand;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate enum_primitive;
#[macro_use] extern crate colorify;


mod cortex;
mod subcortex;
mod tract_terminal;
pub mod encode;
pub mod flywheel;
pub mod map;
#[macro_use] pub mod cmn;
#[cfg(test)] pub mod tests;

pub use ocl::Event as OclEvent;
pub use self::cortex::{Cortex, CorticalArea, CorticalAreas, AxonSpace, Synapses, Minicolumns,
    InhibitoryInterneuronNetwork, PyramidalLayer, SpinyStellateLayer, Dendrites,
    CorticalAreaSettings};
pub use self::subcortex::{Subcortex, SubcorticalNucleus, TestScNucleus};
pub use self::subcortex::thalamus::{self, ExternalPathwayTract, ExternalPathwayEncoder};
pub use self::flywheel::Flywheel;
pub use self::map::{LayerMapSchemeList, AreaSchemeList, AreaMap};
pub use self::cmn::{TractDims, TypeId, CmnError, CmnResult};
pub use self::encode::GlyphBuckets;