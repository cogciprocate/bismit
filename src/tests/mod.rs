
//pub use self::renderer::{ Renderer };

//pub mod interactive;
//pub mod autorun;
//mod synapse_drill_down;
//pub mod input_czar;
//mod output_czar;
//mod motor_state;
//mod renderer;

#[cfg(test)]
pub use self::testbed::{ TestBed };
#[cfg(test)]
pub mod hybrid;
#[cfg(test)]
pub mod testbed;
#[cfg(test)]
pub mod kernels;
#[cfg(test)]
mod learning;
#[cfg(test)]
mod automated;

pub static PASS_STR: &'static str = "\x1b[1;32mpass\x1b[0m";
