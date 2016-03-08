//! # Bismit: Biologically Inspired Sensory Motor Interface Tool

// #![allow(dead_code, unused_variables, unused_assignments)]
// #![allow(unused_features)]
// #![feature(clone_from_slice)]

extern crate num;
extern crate libc;
extern crate time;
//extern crate yaml_rust;
extern crate microcosm;
extern crate ocl;
extern crate find_folder;
#[macro_use] extern crate rand;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate enum_primitive;

#[macro_use] pub mod cmn;
// mod ocl;
mod axon_space;
mod synapses;
mod minicolumns;
mod iinn;
mod pyramidals;
mod thalamus;
// mod proto;
//mod energy;
mod sensory_filter;
// mod interactive;

#[cfg(test)] pub mod tests;
pub mod spiny_stellates;
pub mod cortex;
pub mod input_source;
pub mod map;
pub mod encode;
pub mod dendrites;
pub mod cortical_area;

pub use map::proto;
