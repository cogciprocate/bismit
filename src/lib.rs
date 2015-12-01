#![allow(dead_code, unused_variables, unused_assignments)]
#![feature(vec_push_all, drain)]

pub use map::proto;

extern crate num;
extern crate libc;
extern crate time;
//extern crate yaml_rust;
extern crate microcosm;
extern crate ocl;

#[macro_use]
extern crate rand;
#[macro_use] 
extern crate bitflags;
#[macro_use] 
extern crate enum_primitive;

#[macro_use]
pub mod cmn;
// mod ocl;
pub mod cortical_area;
mod axon_space;
pub mod dendrites;
mod synapses;
mod minicolumns;
mod iinn;
mod pyramidals;
pub mod spiny_stellates;
pub mod cortex;
mod thalamus;
// mod proto;
//mod energy;
pub mod encode;
mod sensory_filter;
pub mod input_source;
pub mod map;
// mod interactive;

#[cfg(test)]
pub mod tests;
