//! Encode a sequence of scalar values and display their representation.

// #![allow(unused_imports, unused_variables, dead_code)]

extern crate rand;
extern crate env_logger;
extern crate clap;
#[macro_use] extern crate log;
#[macro_use] extern crate colorify;
extern crate smallvec;
extern crate vibi;
extern crate qutex;
// extern crate ansi_term;

// mod layer_sampler;
mod spatial;
mod hexdraw;
mod sequence;
mod motor;
mod sensory;
mod old_test;

use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use rand::{FromEntropy, rngs::SmallRng};
use rand::distributions::{Range, Distribution};
// use vibi::bismit::futures::executor;
// use qutex::QrwLock;
use vibi::window;
use vibi::bismit::ocl::{Buffer, RwVec};
use vibi::bismit::{encode, Cortex, SubcorticalNucleusLayer, TractSender,
    TractReceiver, CorticalDims, TractDims, Thalamus};
use vibi::bismit::flywheel::{Flywheel, Command, Request, Response};
use vibi::bismit::map::{AreaMap, AxonTopology, LayerAddress, AxonDomain};


pub struct Params {
    pub tract_buffer: RwVec<u8>,
    pub axns: Buffer<u8>,
    // pub l4_axns: Buffer<u8>,
    pub area_map: AreaMap,
    pub encode_dim: [u32; 2],
    pub area_dim: u32,
}


pub struct Controls {
    pub cmd_tx: Sender<Command>,
    pub req_tx: Sender<Request>,
    pub res_rx: Receiver<Response>,
    pub th_flywheel: thread::JoinHandle<()>,
    pub th_win: Option<thread::JoinHandle<()>>,
}


pub fn spawn_threads(cortex: Cortex, pri_area_name: &'static str, vibi: bool)
        -> Controls {
    let (command_tx, command_rx) = mpsc::channel();
    let (vibi_request_tx, vibi_request_rx) = mpsc::channel();
    let (vibi_response_tx, vibi_response_rx) = mpsc::channel();
    let vibi_command_tx = command_tx.clone();

    let (control_request_tx, control_request_rx) = mpsc::channel();
    let (control_response_tx, control_response_rx) = mpsc::channel();
    let control_command_tx = command_tx;

    let mut flywheel = Flywheel::new(cortex, command_rx, pri_area_name);
    flywheel.add_req_res_pair(vibi_request_rx, vibi_response_tx);
    flywheel.add_req_res_pair(control_request_rx, control_response_tx);

    // Flywheel thread:
    let th_flywheel = thread::Builder::new().name("flywheel".to_string())
            .spawn(move || {
        println!("Controls::spawn_threads: Spinning flywheel...");
        flywheel.spin();
    }).expect("Error creating 'flywheel' thread");

    // Vibi thread:
    let th_win = if vibi {
        Some(thread::Builder::new().name("win".to_string()).spawn(move || {
            println!("Opening vibi window...");
            window::Window::open(vibi_command_tx, vibi_request_tx, vibi_response_rx);
        }).expect("Error creating 'win' thread"))
    } else {
        None
    };

    Controls {
        cmd_tx: control_command_tx,
        req_tx: control_request_tx,
        res_rx: control_response_rx,
        th_flywheel,
        th_win,
    }
}

pub fn join_threads(mut controls: Controls) {
    // if let Err(e) = controls.th_win.take().join() {
    //     println!("th_win.join(): Error: '{:?}'", e);
    // }
    if let Some(th_win) = controls.th_win.take() {
        if let Err(e) = th_win.join() {
            println!("th_win.join(): Error: '{:?}'", e);
        }
        println!("Vibi window closed.");
    }
    if let Err(e) = controls.th_flywheel.join() {
        println!("th_flywheel.join(): Error: '{:?}'", e);
    }
    println!("Flywheel stopped.");
}


fn main() {
    use clap::{Arg, /*ArgGroup,*/ App, /*SubCommand*/};

    env_logger::init();
    // ansi_term::enable_ansi_support().ok();

    let matches = App::new("Bismit Evaluator")
        .version("0.1")
        .author("Nick Sanders <cogciprocate@gmail.com>")
        .about("Runs a variety of evaluations and tests using Bismit")
        .arg(Arg::with_name("EVALUATION")
            .help("Specifies the evaluation to run")
            .required(true)
            .index(1)
            .value_name("spatial, hexdraw, motor, sensory"))
        .arg(Arg::with_name("SUBEVALUATION")
            .help("Specifies an evaluation argument")
            .required(false)
            .index(2))
        .get_matches();

    let sub = matches.value_of("SUBEVALUATION");

    match matches.value_of("EVALUATION").unwrap() {
        "spatial" => spatial::eval(),
        "hexdraw" => hexdraw::eval(sub),
        "motor" => motor::eval(),
        "sensory" => sensory::eval(),
        "sequence" => sequence::eval(),
        "old_test" => old_test::eval(),
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
pub enum Pathway {
    Output { tx: TractSender },
    Input { srcs: Vec<InputSource> },
    None,
}

impl Pathway {
    pub fn output(thal: &mut Thalamus, layer: &SubcorticalNucleusLayer,
            wait_for_frame: bool) -> Pathway {
        let tx = thal.input_pathway(layer.addr(), wait_for_frame);
        Pathway::Output { tx }
    }

    pub fn input(thal: &mut Thalamus, layer: &SubcorticalNucleusLayer)
            -> Pathway {
        let src_lyr_infos: Vec<_> =
            thal.area_maps().by_index(layer.addr().area_id())
                    .unwrap()
                    .layer(layer.addr().layer_id()).unwrap()
                    .sources().iter().map(|src_lyr| {
                (src_lyr.layer_addr(), src_lyr.dims().clone())
        }).collect();

        let srcs: Vec<_> = src_lyr_infos.into_iter().map(|(addr, dims)| {
            InputSource {
                addr,
                dims,
                rx: thal.output_pathway(addr)
            }
        }).collect();

        Pathway::Input { srcs }
    }

    /// Creates the appropriate pathway with backpressure if the layer is an
    /// output layer.
    pub fn new(thal: &mut Thalamus, layer: &SubcorticalNucleusLayer) -> Pathway {
        match *layer.axon_domain() {
            AxonDomain::Output(_) => Pathway::output(thal, layer, true),
            AxonDomain::Input(_) => Pathway::input(thal, layer),
            _ => Pathway::None,
        }
    }
}


/// A subcortical nucleus layer with a pathway.
#[derive(Debug)]
pub struct Layer {
    sub: SubcorticalNucleusLayer,
    pathway: Pathway,
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

    pub fn pathway(&self) -> &Pathway {
        &self.pathway
    }
}


#[derive(Debug)]
pub struct Sdrs {
    pub pattern_count: usize,
    pub active_cell_count: usize,
    pub dims: TractDims,
    pub indices: Vec<Vec<u32>>,
    pub sdrs: Vec<Vec<u8>>,
    pub rng: SmallRng,
}

impl Sdrs {
    pub fn new<D>(pattern_count: usize, dims: D) -> Sdrs
            where D: Into<TractDims> {
        let dims = dims.into();
        const SPARSITY: usize = 48;
        // let pattern_count = 300;
        let cell_count = dims.to_len();
        let active_cell_count = cell_count / SPARSITY;

        let mut rng = SmallRng::from_entropy();

        // Produce randomized indexes:
        let pattern_indices: Vec<_> = (0..pattern_count).map(|_| {
            encode::gen_axn_idxs(&mut rng, active_cell_count, cell_count)
        }).collect();

        // Create sdr from randomized indexes:
        let sdrs: Vec<_> = pattern_indices.iter().map(|axn_idxs| {
            let mut sdr = vec![0u8; cell_count];
            for &axn_idx in axn_idxs.iter() {
                sdr[axn_idx as usize] = Range::new(1, 256).sample(&mut rng) as u8;
            }
            sdr
        }).collect();

        Sdrs {
            pattern_count,
            active_cell_count,
            dims,
            indices: pattern_indices,
            sdrs,
            rng,
        }
    }

    /// Returns the (v, u) coords. of the next active cell with an index
    /// greater than (the next active cell after) `pattern_idx`.
    pub fn a_middle_active_cell(&self, pattern_idx: usize) -> (u32, u32) {
        // let patterns = executor::block_on(self.sdrs.clone().read()).unwrap();
        let pattern = &self.sdrs[pattern_idx];

        let mut mid_active_cel_idx = None;
        let mid_cel_idx = pattern.len() / 2;

        for cel_idx in mid_cel_idx..pattern.len() {
            let cel_state = pattern[cel_idx];
            if cel_state > 0 {
                mid_active_cel_idx = Some(cel_idx);
            }
        }

        let cel_idx = mid_active_cel_idx.expect(&format!("Sdrs::middle_active_cell: \
            No active cells in the pattern with an index greater than [{}].",
            mid_cel_idx)) as u32;
        let v = cel_idx / self.dims.u_size();
        let u = cel_idx - (v * self.dims.u_size());
        (v, u)
    }

    pub fn len(&self) -> usize { self.pattern_count }
}


/// A sequence cursor position.
#[derive(Clone, Debug)]
pub struct SeqCursorPos {
    seq_idx: usize,
    seq_item_idx: usize,
    pattern_idx: usize,
}

impl Default for SeqCursorPos {
    fn default() -> SeqCursorPos {
        SeqCursorPos { seq_idx: 0, seq_item_idx: 0, pattern_idx: 0 }
    }
}


/// A cursor for iterating over sequences at random.
///
/// A sequence is always returned sequentially. When the end of the sequence
/// is reached the next sequence is selected randomly.
//
// TODO: Come up with less confusing names for struct members/methods.
//
#[derive(Debug)]
pub struct SeqCursor {
    sequences: Vec<Vec<usize>>,
    cur_seq_idx: usize,
    next_seq_idx: usize,
    cur_seq_item_idx: usize,
    next_seq_item_idx: usize,
    rng: SmallRng,
}

impl SeqCursor {
    /// Returns a new `SeqCursor`.
    ///
    /// * seq_lens: (min, max) sequence lengths.
    /// * seq_count: number of sequences to generate.
    /// * src_idx_count: Length of the source pool.
    ///
    pub fn new(seq_lens: (usize, usize), seq_count: usize, src_idx_count: usize)
            -> SeqCursor {
        assert!(seq_lens.1 >= seq_lens.0, "SeqCursor::new(): Sequence length range \
            ('seq_lens') invalid. High end must at least be equal to low end: '{:?}'.",
                seq_lens);

        let mut rng = SmallRng::from_entropy();
        let mut sequences = Vec::with_capacity(seq_count);

        // Build sequences of source indexes:
        for _ in 0..seq_count {
            let mut seq_len = 0;
            while seq_len == 0 {
                seq_len = Range::new(seq_lens.0, seq_lens.1 + 1).sample(&mut rng);
            }
            let mut seq = Vec::<usize>::with_capacity(seq_len);

            for _ in 0..seq_len {
                let src_idx = Range::new(0, src_idx_count).sample(&mut rng);
                seq.push(src_idx);
            }

            sequences.push(seq);
        }

        // Initial sequence index (randomized for fun):
        let cur_seq_idx = Range::new(0, sequences.len()).sample(&mut rng);

        // Next sequence index:
        let next_seq_idx = Range::new(0, sequences.len()).sample(&mut rng);

        SeqCursor {
            sequences: sequences,
            cur_seq_idx,
            next_seq_idx,
            cur_seq_item_idx: 0,
            next_seq_item_idx: 0,
            rng
        }
    }

    /// Returns a 'middle' source index from the lot.
    ///
    /// Could be used to select an active cell from sdr lists for
    /// testing/eval. purposes.
    pub fn a_middle_src_idx(&self) -> usize {
        let mid_seq_idx = self.sequences.len() / 2;
        let mid_seq_item_idx = self.sequences[mid_seq_idx].len() / 2;
        self.sequences[mid_seq_idx][mid_seq_item_idx]
    }

    /// Increments all indexes then returns the resulting source index.
    ///
    // If if the next source index is the final index in the sequence, a new
    // random sequence is selected.
    pub fn incr_src_idx(&mut self) -> usize {
        self.cur_seq_idx = self.next_seq_idx;
        self.cur_seq_item_idx = self.next_seq_item_idx;
        self.next_seq_item_idx += 1;

        if self.next_seq_item_idx >= self.sequences[self.cur_seq_idx].len() {
            self.next_seq_idx = Range::new(0, self.sequences.len())
                .sample(&mut self.rng);
            self.next_seq_item_idx = 0;
        }
        self.sequences[self.cur_seq_idx][self.cur_seq_item_idx]
    }

    /// Returns the current position of the cursor.
    pub fn curr_pos(&self) -> SeqCursorPos {
        SeqCursorPos {
            seq_idx: self.cur_seq_idx,
            seq_item_idx: self.cur_seq_item_idx,
            pattern_idx: self.sequences[self.cur_seq_idx][self.cur_seq_item_idx],
        }
    }

    /// Returns the next position of the cursor.
    pub fn next_pos(&self) -> SeqCursorPos {
        SeqCursorPos {
            seq_idx: self.next_seq_idx,
            seq_item_idx: self.next_seq_item_idx,
            pattern_idx: self.sequences[self.next_seq_idx][self.next_seq_item_idx],
        }
    }
}

