//! # Bismit: Biologically Inspired Sensory Motor Interface Tool

extern crate num;
extern crate libc;
extern crate time;
extern crate ocl;
extern crate find_folder;
#[macro_use] extern crate rand;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate enum_primitive;

#[macro_use] pub mod cmn;
mod thalamus;
mod sensory_filter;
mod cortex;

#[cfg(test)] pub mod tests;
pub mod input_source;
pub mod map;
pub mod encode;

pub use map::proto;
pub use ocl::Event as OclEvent;

pub use self::cortex::{ Cortex, CorticalArea, AxonSpace, Synapses, Minicolumns, InhibitoryInterneuronNetwork, PyramidalLayer, 
	SpinyStellateLayer, Dendrites };
pub use self::input_source::InputTract;
pub use self::map::SliceTractMap;