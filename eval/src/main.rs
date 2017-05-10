//! Encode a sequence of scalar values and display their representation.

#![allow(unused_imports, unused_variables, dead_code)]

extern crate rand;
extern crate vibi;
extern crate env_logger;
extern crate clap;
#[macro_use] extern crate log;

mod spatial;
mod hexdraw;

use vibi::window;
use vibi::bismit::{map, Cortex, CorticalAreaSettings, Subcortex};
use vibi::bismit::map::*;
use vibi::bismit::flywheel::Flywheel;
use spatial::Params;



fn main() {
    use clap::{Arg, App, SubCommand};

    env_logger::init().unwrap();

    let matches = App::new("Bismit Evaluator")
        .version("0.1")
        .author("Nick Sanders <cogciprocate@gmail.com>")
        .about("Runs a variety of evaluations and tests using Bismit")
        .arg(Arg::with_name("EVALUATION")
            .help("Specifies the evaluation to run")
            .required(true)
            .index(1))
        .get_matches();

    match matches.value_of("EVALUATION").unwrap() {
        "spatial" => spatial::eval(),
        "hexdraw" => hexdraw::eval(),
        e @ _ => println!("Unknown evaluation specified: {}", e),
    }
}

