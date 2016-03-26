
//pub use self::renderer::{Renderer};

//pub mod interactive;
//pub mod autorun;
//mod synapse_drill_down;
//pub mod input_czar;
//mod output_czar;
//mod motor_state;
//mod renderer;

mod dens_tfts;
mod automated;
mod cycle;
pub mod testbed;
pub mod testbed_vibi;
pub mod util;
pub mod hybrid;
pub mod kernels;
pub mod learning;

pub use self::testbed::{TestBed};

pub static PASS_STR: &'static str = "\x1b[1;32mpass\x1b[0m";

