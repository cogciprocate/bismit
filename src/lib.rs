//! # Bismit: Biologically Inspired Sensory Motor Interface Tool

// #![feature(discriminant_value)]

extern crate num;
extern crate libc;
extern crate time;
extern crate find_folder;
extern crate twox_hash;
extern crate rand;
extern crate futures;
extern crate tokio_core;
// extern crate cpuprofiler;
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
pub use self::cortex::{Cortex, CorticalArea, AxonSpace, Synapses, /*Minicolumns,*/
    InhibitoryInterneuronNetwork, ActivitySmoother, PyramidalLayer, SpinyStellateLayer, Dendrites,
    CorticalAreaSettings, DataCellLayer};
pub use self::subcortex::{Thalamus, Subcortex, SubcorticalNucleus, TestScNucleus, ExternalPathway,
    ExternalPathwayTract, ExternalPathwayEncoder, ExternalPathwayFrame, ExternalPathwayLayer};
// pub use self::subcortex::thalamus::{};
pub use self::flywheel::Flywheel;
pub use self::map::{LayerMapSchemeList, AreaSchemeList, AreaMap};
pub use self::cmn::{util, TractDims, TypeId, CmnError as Error, CmnResult as Result};
pub use self::encode::GlyphBuckets;