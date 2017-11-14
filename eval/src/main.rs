//! Encode a sequence of scalar values and display their representation.

// #![allow(unused_imports, unused_variables, dead_code)]

extern crate rand;
extern crate vibi;
extern crate env_logger;
extern crate clap;
#[macro_use] extern crate log;
#[macro_use] extern crate colorify;
extern crate qutex;

mod spatial;
mod hexdraw;
mod motor;

use vibi::window;
use vibi::bismit::Cortex;
use vibi::bismit::flywheel::{Flywheel, Command, Request, Response};
use vibi::bismit::map::AreaMap;
use vibi::bismit::ocl::{Buffer, RwVec};

use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};


pub struct Params {
    pub tract_buffer: RwVec<u8>,
    pub axns: Buffer<u8>,
    pub l4_axns: Buffer<u8>,
    pub area_map: AreaMap,
    pub encode_dim: u32,
    pub area_dim: u32,
}


pub struct Controls {
    pub cmd_tx: Sender<Command>,
    pub req_tx: Sender<Request>,
    pub res_rx: Receiver<Response>,
    pub th_flywheel: thread::JoinHandle<()>,
    pub th_win: thread::JoinHandle<()>,
}


pub fn spawn_threads(cortex: Cortex, pri_area_name: &'static str)
        -> Controls
{
    let (command_tx, command_rx) = mpsc::channel();
    let (vibi_request_tx, vibi_request_rx) = mpsc::channel();
    let (vibi_response_tx, vibi_response_rx) = mpsc::channel();
    let vibi_command_tx = command_tx.clone();

    let (spatial_request_tx, spatial_request_rx) = mpsc::channel();
    let (spatial_response_tx, spatial_response_rx) = mpsc::channel();
    let spatial_command_tx = command_tx;

    let mut flywheel = Flywheel::new(cortex, command_rx, pri_area_name);
    flywheel.add_req_res_pair(vibi_request_rx, vibi_response_tx);
    flywheel.add_req_res_pair(spatial_request_rx, spatial_response_tx);

    // Flywheel thread:
    let th_flywheel = thread::Builder::new().name("flywheel".to_string())
            .spawn(move || {
        flywheel.spin();
    }).expect("Error creating 'flywheel' thread");

    // Vibi thread:
    let th_win = thread::Builder::new().name("win".to_string()).spawn(move || {
        println!("Opening vibi window...");
        window::Window::open(vibi_command_tx, vibi_request_tx, vibi_response_rx);
    }).expect("Error creating 'win' thread");

    Controls {
        cmd_tx: spatial_command_tx,
        req_tx: spatial_request_tx,
        res_rx: spatial_response_rx,
        th_flywheel,
        th_win,
    }
}

pub fn join_threads(controls: Controls) {
    if let Err(e) = controls.th_win.join() {
        println!("th_win.join(): Error: '{:?}'", e);
    }
    println!("Vibi window closed.");
    if let Err(e) = controls.th_flywheel.join() {
        println!("th_flywheel.join(): Error: '{:?}'", e);
    }
    println!("Flywheel stopped.");
}


fn main() {
    use clap::{Arg, /*ArgGroup,*/ App, /*SubCommand*/};

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
        "motor" => motor::eval(),
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

