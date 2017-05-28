//! Encode a sequence of scalar values and display their representation.

#![allow(unused_imports, unused_variables, dead_code)]

extern crate rand;
extern crate vibi;
extern crate env_logger;
extern crate clap;
#[macro_use] extern crate log;
#[macro_use] extern crate colorify;

mod spatial;
mod hexdraw;

use vibi::window;
use vibi::bismit::{map, Cortex, CorticalAreaSettings, Subcortex};
use vibi::bismit::map::*;
use vibi::bismit::flywheel::Flywheel;
use spatial::Params;



fn main() {
    use clap::{Arg, ArgGroup, App, SubCommand};

    env_logger::init().unwrap();

    let matches = App::new("Bismit Evaluator")
        .version("0.1")
        .author("Nick Sanders <cogciprocate@gmail.com>")
        .about("Runs a variety of evaluations and tests using Bismit")
        .arg(Arg::with_name("EVALUATION")
            .help("Specifies the evaluation to run")
            .required(true)
            .index(1)
            .value_name("spatial, hexdraw"))
        .get_matches();

    match matches.value_of("EVALUATION").unwrap() {
        "spatial" => spatial::eval(),
        "hexdraw" => hexdraw::eval(),
        e @ _ => println!("Unknown evaluation specified: {}", e),
    }

    // let matches = App::new("Bismit Evaluator")
    //     .version("0.1")
    //     .author("Nick Sanders <cogciprocate@gmail.com>")
    //     .about("Runs a variety of evaluations and tests using Bismit")
    //     .arg(Arg::with_name("spatial")
    //         .help("Spatial activity frequencies") )
    //     .arg(Arg::with_name("hexdraw")
    //         .help("Draw arbitrary patterns") )
    //     .group(ArgGroup::with_name("evaluation")
    //         .args(&["spatial", "hexdraw"])
    //         .required(true))
    //     .get_matches();

    // if matches.is_present("spatial") {
    //     // spatial::eval()
    //     println!("")
    // } else if matches.is_present("hexdraw") {
    //     hexdraw::eval()
    // } else {
    //     println!("No evaluation specified");
    // }
}

