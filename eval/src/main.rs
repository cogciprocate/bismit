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


use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};

use vibi::window;
use vibi::bismit::ocl::{Buffer, RwVec};
use vibi::bismit::{Cortex, SubcorticalNucleusLayer, TractSender, TractReceiver,
    CorticalDims};
use vibi::bismit::flywheel::{Flywheel, Command, Request, Response};
use vibi::bismit::map::{AreaMap, AxonTopology, LayerAddress};


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
        -> Controls {
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




/// A result of incrementing a `CycleCounter`.
#[derive(Clone, Copy, Debug)]
pub enum IncrResult {
    Training,
    TrainingComplete,
    Collecting,
    CollectingComplete,
    TrialComplete { scheme_idx: usize, train: usize, collect: usize },
}


/// An iterator over the cycles of a currently running trial.
#[derive(Clone, Copy, Debug)]
pub struct CycleCounter {
    train_total: usize,
    train_complete: usize,
    collect_total: usize,
    collect_complete: usize,
}

impl CycleCounter {
    /// Returns a new cycle counter.
    pub fn new(train_total: usize, collect_total: usize) -> CycleCounter {
        CycleCounter {
            train_total,
            train_complete: 0,
            collect_total,
            collect_complete: 0,
        }
    }

    /// Returns true if the trial is currently on a training cycle.
    pub fn is_training(&self) -> bool {
        self.train_complete < self.train_total
    }

    /// Returns true if the trial is currently on a collecting cycle.
    pub fn is_collecting(&self) -> bool {
        self.collect_complete < self.collect_total
    }

    /// Returns true if the collect complete counter is 1 away from completion.
    pub fn is_last_cycle(&self) -> bool {
        self.collect_complete + 1 == self.collect_total
    }

    /// Returns true if all training and collecting cycles are complete.
    pub fn all_complete(&self) -> bool {
        self.train_complete >= self.train_total &&
            self.collect_complete >= self.collect_total
    }

    /// Increments the currently running trial run iterator and returns `true`
    /// if all trial runs have completed (both training and collecting).
    pub fn incr(&mut self) -> IncrResult {
        if self.is_training() {
            self.train_complete += 1;
            if self.is_training() {
                IncrResult::Training
            } else {
                IncrResult::TrainingComplete
            }
        } else {
            if self.is_collecting() {
                self.collect_complete += 1;
                if self.is_collecting() {
                    IncrResult::Collecting
                } else {
                    IncrResult::CollectingComplete
                }
            } else {
                IncrResult::CollectingComplete
            }
        }
    }
}

impl From<(usize, usize)> for CycleCounter {
    fn from(totals: (usize, usize)) -> CycleCounter {
        CycleCounter::new(totals.0, totals.1)
    }
}


/// The set of all trial iterators.
#[derive(Clone, Debug)]
struct TrialIter {
    schemes: Vec<(usize, usize)>,
    current_counter: CycleCounter,
    current_scheme_idx: usize,
    global_cycle_idx: usize,
}

impl TrialIter {
    // Defines the number of cycles to first train then collect for each
    // sample period (trial).
    pub fn new(schemes: Vec<(usize, usize)>) -> TrialIter {
        assert!(schemes.len() > 0, "TrialIter::new: Empty scheme list.");
        let first_counter = schemes[0].into();

        TrialIter {
            schemes,
            current_counter: first_counter,
            current_scheme_idx: 0,
            global_cycle_idx: 0,
        }
    }

    /// Increments the current scheme index and returns true if the
    /// incrementation resets the counter.
    fn next_scheme(&mut self) -> bool {
        if self.current_scheme_idx < self.schemes.len() - 1 {
            self.current_scheme_idx += 1;
            false
        } else {
            self.current_scheme_idx = 0;
            true
        }
    }

    /// Increment the cycle counters.
    pub fn incr(&mut self) -> IncrResult {
        self.global_cycle_idx = self.global_cycle_idx.wrapping_add(1);

        match self.current_counter.incr() {
            IncrResult::CollectingComplete => {
                let completed_scheme = self.schemes[self.current_scheme_idx];
                let completed_scheme_idx = self.current_scheme_idx;
                self.next_scheme();
                self.current_counter = self.schemes[self.current_scheme_idx].into();
                IncrResult::TrialComplete {
                    scheme_idx: completed_scheme_idx,
                    train: completed_scheme.0,
                    collect: completed_scheme.1
                }
            },
            r @ _ => r,
        }
    }

    pub fn current_counter(&self) -> &CycleCounter {
        &self.current_counter
    }
}


#[derive(Debug)]
pub struct InputSource {
    addr: LayerAddress,
    dims: CorticalDims,
    rx: TractReceiver,
}


#[derive(Debug)]
pub enum PathwayDir {
    Output { tx: TractSender },
    Input { srcs: Vec<InputSource> },
    None,
}


/// A subcortical nucleus layer with a pathway.
#[derive(Debug)]
pub struct Layer {
    sub: SubcorticalNucleusLayer,
    pathway: PathwayDir,
}

impl Layer {
    pub fn set_dims(&mut self, dims: CorticalDims) {
        self.sub.set_dims(dims);
    }

    pub fn axn_topology(&self) -> AxonTopology {
        self.sub.axon_topology().clone()
    }

    pub fn sub(&self) -> &SubcorticalNucleusLayer {
        &self.sub
    }

    pub fn sub_mut(&mut self) -> &mut SubcorticalNucleusLayer {
        &mut self.sub
    }

    pub fn pathway(&self) -> &PathwayDir {
        &self.pathway
    }
}
