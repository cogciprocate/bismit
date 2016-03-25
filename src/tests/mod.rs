
//pub use self::renderer::{Renderer};

//pub mod interactive;
//pub mod autorun;
//mod synapse_drill_down;
//pub mod input_czar;
//mod output_czar;
//mod motor_state;
//mod renderer;

pub mod testbed;
pub mod util;
// pub mod hybrid;
// pub mod kernels;
// mod dens_tfts;
pub mod learning;
// mod automated;

pub use self::testbed::{TestBed};

pub static PASS_STR: &'static str = "\x1b[1;32mpass\x1b[0m";

