
//pub use self::renderer::{ Renderer };

//pub mod interactive;
//pub mod autorun;
//mod synapse_drill_down;
//pub mod input_czar;
//mod output_czar;
//mod motor_state;
//mod renderer;

pub use self::testbed::{ TestBed };
pub mod hybrid;
pub mod testbed;
pub mod kernels;
mod learning;
mod automated;

pub static PASS_STR: &'static str = "\x1b[1;32mpass\x1b[0m";
