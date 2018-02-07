// #![allow(dead_code)]

use std::mem;
use std::collections::{BTreeMap, HashMap};
use rand::{self, XorShiftRng};
use rand::distributions::{Range, IndependentSample};
use qutex::QrwLock;
use vibi::bismit::futures::Future;
use vibi::bismit::map::*;
use vibi::bismit::ocl::{Buffer, WriteGuard};
use vibi::bismit::{map, Cortex, CorticalAreaSettings, Thalamus, SubcorticalNucleus,
    SubcorticalNucleusLayer, WorkPool, WorkPoolRemote, TractSender, TractReceiver};
use vibi::bismit::flywheel::{Command, Request, Response};
use vibi::bismit::encode::{self};
use vibi::bismit::cmn::{MapStore, CorticalDims, CmnResult};
use vibi::bismit::{CorticalArea, SamplerKind, SamplerBufferKind};
use ::{Controls, Params};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 48;
const AREA_DIM: u32 = 16;
const SEQUENTIAL_SDR: bool = true;


type CellIdx = usize;
type ActivityCount = usize;
type ActiveCells = BTreeMap<CellIdx, ActivityCount>;
type PatternIdx = usize;
type PatternAssociations = BTreeMap<PatternIdx, ActiveCells>;

#[derive(Clone, Debug)]
struct TrialResults {
    trials: Vec<PatternAssociations>,
    trial_cycle_counts: Vec<usize>,
    pattern_watch_list: Vec<usize>,
}

impl TrialResults {
    pub fn new(pattern_watch_list: Vec<PatternIdx>) -> TrialResults {
        TrialResults {
            trials: Vec::with_capacity(16),
            trial_cycle_counts: Vec::with_capacity(16),
            pattern_watch_list,
        }
    }

    /// Adds the cell activity counts for each pattern in the watch list.
    ///
    /// Only adds those cells with activity counts above `actv_cutoff`, if
    /// specified.
    pub fn add(&mut self, trial_data: TrialData) {
        let mut pattern_assoc = PatternAssociations::new();
        let activity_counts = trial_data.activity_counts().clone().read().wait().unwrap();

        for &pattern_idx in &self.pattern_watch_list {
            let mut active_cells = ActiveCells::new();

            for (cell_idx, cell) in activity_counts.iter().enumerate() {
                let cell_actv_cnt = cell[pattern_idx];

                match trial_data.actv_cutoff {
                    Some(ac) => {
                        if cell_actv_cnt >= ac {
                            active_cells.insert(cell_idx, cell_actv_cnt);
                        }
                    },
                    None => { active_cells.insert(cell_idx, cell_actv_cnt); },
                }
            }

            pattern_assoc.insert(pattern_idx, active_cells);
        }

        self.trials.push(pattern_assoc);
        self.trial_cycle_counts.push(trial_data.ttl_cycle_count.expect("TrialResult::add: Count not set."));
    }

    /// Calculates the normalized consistency rating for two trials.
    fn trial_consistency(&self, trial_a_idx: usize, trial_b_idx: usize, ignore_inactive: bool) -> f32 {
        fn cell_consistency(actv_count_a: ActivityCount, actv_count_a_ttl: ActivityCount, actv_count_b: ActivityCount,
                actv_count_b_ttl: ActivityCount) -> (f32, f32) {
            let actv_count_a_norm = actv_count_a as f32 / actv_count_a_ttl as f32;
            let actv_count_b_norm = actv_count_b as f32 / actv_count_b_ttl as f32;

            let consistency_rating = 1.0 - (actv_count_a_norm - actv_count_b_norm).abs();
            let influence_factor = (actv_count_a_norm + actv_count_b_norm) / 2.;
            (consistency_rating, influence_factor)
        }

        fn cell_deviation(actv_count_a: ActivityCount, actv_count_a_ttl: ActivityCount, actv_count_b: ActivityCount,
                actv_count_b_ttl: ActivityCount) -> f32 {
            let actv_count_a_norm = actv_count_a as f32 / actv_count_a_ttl as f32;
            let actv_count_b_norm = actv_count_b as f32 / actv_count_b_ttl as f32;

            // let abs_deviation = (actv_count_a_norm - actv_count_b_norm).abs();
            // let influence_factor = (actv_count_a_norm + actv_count_b_norm) / 2.;
            // (consistency_rating, influence_factor)
            (actv_count_a_norm - actv_count_b_norm).abs()
        }

        let trial_a = self.trials.get(trial_a_idx).expect("Trial A index OOR.");
        let trial_b = self.trials.get(trial_b_idx).expect("Trial B index OOR.");
        let trial_a_cycle_count = self.trial_cycle_counts[trial_a_idx];
        let trial_b_cycle_count = self.trial_cycle_counts[trial_b_idx];
        assert!(trial_a.len() == trial_b.len());

        // This could just be a `Vec`:
        let mut pattern_consistencies: HashMap<PatternIdx, f32> = HashMap::new();
        let mut pattern_deviations: HashMap<PatternIdx, f32> = HashMap::new();

        for ((&pat_idx_a, active_cells_a), (&pat_idx_b, active_cells_b)) in trial_a.iter().zip(trial_b.iter()) {
            assert!(pat_idx_a == pat_idx_b);
            // This hashmap stores the consistency rating and an influence
            // factor. Influence factor is determined by the average
            // normalized activity counts. Influence factor directly affects
            // how much the consistency rating affects the overall consistency
            // total for that pattern.
            let mut cell_consistencies: HashMap<CellIdx, (f32, f32)> = HashMap::new();
            let mut cell_deviations: HashMap<CellIdx, f32> = HashMap::new();

            for (&cell_idx, &actv_count_a) in active_cells_a.iter() {
                let actv_count_b = match active_cells_b.get(&cell_idx) {
                    Some(&count) => count,
                    None => 0,
                };

                let (consistency, influence) = cell_consistency(actv_count_a, trial_a_cycle_count,
                    actv_count_b, trial_b_cycle_count);
                let deviation = cell_deviation(actv_count_a, trial_a_cycle_count,
                    actv_count_b, trial_b_cycle_count);

                if !(ignore_inactive && (actv_count_a == 0 && actv_count_b == 0)) {
                    cell_consistencies.insert(cell_idx, (consistency, influence));
                    cell_deviations.insert(cell_idx, deviation);
                }
            }

            for (&cell_idx, &actv_count_b) in active_cells_b.iter() {
                let actv_count_a = match active_cells_a.get(&cell_idx) {
                    Some(&count) => count,
                    None => 0,
                };

                let (consistency, influence) = cell_consistency(actv_count_a, trial_a_cycle_count,
                    actv_count_b, trial_b_cycle_count);
                let deviation = cell_deviation(actv_count_a, trial_a_cycle_count,
                    actv_count_b, trial_b_cycle_count);

                if !(ignore_inactive && (actv_count_a == 0 && actv_count_b == 0)) {
                    match cell_consistencies.insert(cell_idx, (consistency, influence)) {
                        Some((cnsty, infl)) => assert!(cnsty == consistency && infl == influence),
                        None => (),
                    }
                    match cell_deviations.insert(cell_idx, deviation) {
                        Some(dev) => assert!(dev == deviation),
                        None => (),
                    }
                }
            }

            let mut pat_cnsty_ttl = 0.;
            let mut pat_infl_ttl = 0.;
            // let cel_cnt = cell_consistencies.len() as f32;
            for (_, (cel_cnsty, cel_infl)) in cell_consistencies {
                pat_cnsty_ttl += cel_cnsty * cel_infl;
                pat_infl_ttl += cel_infl;
            }
            pattern_consistencies.insert(pat_idx_a, (pat_cnsty_ttl / pat_infl_ttl) * 100.);

            let mut dev_ttl = 0.;
            let cel_cnt = cell_deviations.len() as f32;
            for (_, cel_dev) in cell_deviations {
                dev_ttl += cel_dev;
            }
            pattern_deviations.insert(pat_idx_a, (dev_ttl / cel_cnt) * 100.);
        }

        let mut consistency_total = 0.;
        let pattern_count = trial_a.len() as f32;
        for (_, pat_cnsty) in pattern_consistencies {
            consistency_total += pat_cnsty;
        }

        // let mut deviation_total = 0.;
        // for (_, pat_dev) in pattern_deviations {
        //     deviation_total += pat_dev;
        // }

        consistency_total / pattern_count
        // deviation_total / pattern_count

    }

    pub fn prior_trial_consistencies(&self, trial_a_idx: usize) -> Vec<f32> {
        let mut trial_a_cnstys = Vec::with_capacity(trial_a_idx);
        for trial_b_idx in 0..trial_a_idx {
            trial_a_cnstys.push(self.trial_consistency(trial_a_idx, trial_b_idx, false))
        }
        trial_a_cnstys
    }


    /// Calculates the consistency of the last trial with all prior trials and
    /// returns a list with one element for each previous trial.
    ///
    /// Consistency is calculated as the average over each cells closeness
    /// with it's past counterpart where the closeness is the normalized (0.0
    /// - 1.0) difference of activity counts between the floor (0) and the max
    /// (the number of cycles per pattern).
    #[allow(dead_code)]
    pub fn all_past_consistencies(&self) -> Vec<Vec<f32>> {
        let mut trial_consistencies = Vec::with_capacity(self.trials.len());
        // Compare each trial to all prior trials:
        for trial_a_idx in 0..self.trials.len() {
            trial_consistencies.push(self.prior_trial_consistencies(trial_a_idx));
        }
        trial_consistencies
    }

    /// Prints all cell activity counts over the `actv_cutoff` threshold.
    pub fn print(&self, trial_idx: usize, cycles_per_pattern: usize, actv_cutoff: usize) {
        // println!("\nTrial[{}]: {:?}", trial_idx, self.trials[trial_idx]);
        printc!(magenta_bold: "\nTrial[{}]: ", trial_idx);
        print!("[activity cutoff (min printed): {}, cycles per pattern (max possible): {}]: ",
            actv_cutoff, cycles_per_pattern);
        print!("\n");

        for (pattern_idx, patterns) in self.trials[trial_idx].iter() {
            printc!(royal_blue_bold: "Pattern {}: ", pattern_idx);
            for (cell_idx, &actv_count) in patterns.iter() {

                // if actv_count >= actv_cutoff {
                // if actv_count > actv_cutoff {
                if actv_count > 0 {
                    print!("{{[");
                    printc!(dark_grey: "{}", cell_idx);
                    print!("]:");
                    printc!(green: "{}", actv_count);
                    print!("}} ");
                }
            }
            print!("\n");
        }
        // print!("\n");

        // Calculate similarity ratios with each previous trial generation:
    }

    #[allow(dead_code)]
    pub fn print_all(&self, cycles_per_pattern: usize, actv_cutoff: usize) {
        println!("\nTrial Results:");

        // TODO: Calculate similarity with previous (and others?):
        // * A 100% similarity would be exactly the same amount of activity for each cell.
        //   * ex: A 4 cell system with activities [100, 50, 50, 100] then
        //     [100, 100, 100, 100] would have 75% similarity.
        //
        // Perhaps a calculation of the similarity rating

        for trial_idx in 0..self.trials.len() {
            self.print(trial_idx, cycles_per_pattern, actv_cutoff);
        }
    }
}


/// Wait for completion.
fn finish_queues(controls: &Controls, i: u64, exiting: &mut bool) {
    loop {
        debug!("Attempting to receive...");
        match controls.res_rx.recv() {
            Ok(res) => match res {
                Response::Status(status) => {
                    debug!("Status: {:?}", status);
                    if status.cycle_counter.0 == i + 1 {
                        controls.req_tx.send(Request::FinishQueues).unwrap();
                        controls.cmd_tx.send(Command::None).unwrap();
                    }
                },
                Response::QueuesFinished(qf_i) => {
                    if qf_i == i + 1 {
                        debug!("Queues finished for cycle: {}", qf_i);
                        break;
                    }
                },
                Response::Exiting => {
                    *exiting = true;
                    break;
                },
                res @ _ => panic!("Unknown response received: {:?}", res),
            },
            Err(_) => {
                *exiting = true;
                break;
            }
        };
    }
}

// Prints dendritic and cell activity ratings as well as a total activity
// count for a selection of cells (currently every 8th).
//
// `_energy_level` can be used to make sure that all cells are being processed
// uniformly by the smoother kernel (by using the '+1 to all' debug code
// contained within).
//
// fn print_activity_counts(buffers: &Buffers, activity_counts: &[Vec<usize>], /*_energy_level: u8*/) {
fn print_activity_counts(den_activities: &[u8], cel_activities: &[u8], cel_energies: &[u8],
        activity_counts: &[Vec<usize>], /*_energy_level: u8*/) {
    let cel_count = activity_counts.len();
    let pattern_count = activity_counts[0].len();
    let mut cel_ttls = Vec::with_capacity(cel_count);
    let mut _non_zero_ptrn_ttls: Vec<(usize, usize)> = Vec::with_capacity(pattern_count);
    let mut ttl_count = 0f32;

    let mut printed = 0usize;

    for (cel_idx, counts) in activity_counts.iter().enumerate() {
        debug_assert!(counts.len() == pattern_count);
        let mut cel_ttl = 0.;
        _non_zero_ptrn_ttls.clear();

        for (pattern_idx, &ptrn_ttl) in counts.iter().enumerate() {
            if ptrn_ttl > 0 {
                _non_zero_ptrn_ttls.push((pattern_idx, ptrn_ttl));
                cel_ttl += ptrn_ttl as f32;
            }
        }

        // `da`: dendrite activity rating (pre-inhib)
        // `ca`: cell activity rating (post-inhib)
        // `ct`: cell activity count

        // if false {
        if (cel_idx & 7) == 0 {
        // if cel_ttl > 0. && cel_ttl < 150. {
        // if cel_ttl > 600. {
        // if cel_energies_vec[cel_idx] == 0 && cel_activities[cel_idx] == 0 {
        // if cel_activities[cel_idx] == 0 {
        // if den_activities[cel_idx] == 0 {
        // if den_activities[cel_idx] == 0 || cel_activities[cel_idx] == 0 {
        // if cel_energies_vec[cel_idx] != _energy_level {
        // if cel_energies_vec[cel_idx] == 0 {
        // if cel_energies_vec[cel_idx] >= 196 {
            print!("{{[");
            printc!(dark_grey: "{}", cel_idx);
            print!("]::da:");
            printc!(green: "{}", den_activities[cel_idx]);
            print!(",ca:");
            printc!(green: "{}", cel_activities[cel_idx]);
            print!(",ce:");
            printc!(green: "{}", cel_energies[cel_idx]);
            print!(",ct:");
            printc!(royal_blue: "{}", cel_ttl);
            print!("}} ");

            printed += 1;
        }

        cel_ttls.push(cel_ttl);
        ttl_count += cel_ttl;
    }

    print!("\n");
    println!("Printed: {}", printed);

    // Calc stdev:
    let mean = ttl_count / cel_count as f32;
    let mut sq_diff_ttl = 0f32;
    for &cel_ttl in cel_ttls.iter() {
        sq_diff_ttl += (cel_ttl - mean).powi(2);
        // print!("<{}>", (cel_ttl - mean).powi(2));
    }
    // print!("\n");

    // TODO: Change to: https://en.wikipedia.org/wiki/Coefficient_of_variation
    // (or just Mean +/- SD)
    let stdev = (sq_diff_ttl / ttl_count).sqrt();
    println!("Standard deviation: {}", stdev);
}

fn track_pattern_activity(controls: &Controls, params: Params, buffers: Buffers) {
    const SPARSITY: usize = 48;
    let pattern_count = 300;
    let cell_count = (params.encode_dim * params.encode_dim) as usize;
    let sdr_active_count = cell_count / SPARSITY;

    let mut rng = rand::weak_rng();

    // Produce randomized indexes:
    let pattern_indices: Vec<_> = (0..pattern_count).map(|_| {
        encode::gen_axn_idxs(&mut rng, sdr_active_count, cell_count)
    }).collect();

    // Create sdr from randomized indexes:
    let sdrs: Vec<_> = pattern_indices.iter().map(|axn_idxs| {
        let mut sdr = vec![0u8; cell_count];
        for &axn_idx in axn_idxs.iter() {
            sdr[axn_idx] = Range::new(96, 160).ind_sample(&mut rng);
        }
        sdr
    }).collect();

    let area_cell_count = (params.area_dim * params.area_dim) as usize;

    // Get the flywheel moving:
    controls.cmd_tx.send(Command::None).unwrap();

    // Define the number of iters to first train then collect for each
    // sample period. All learning and other cell parameters (activity,
    // energy, etc.) persist between sample periods. Only collection
    // iters are recorded and evaluated.
    let training_collect_iters = vec![
        // (0, 5), (0, 5), (0, 5), (0, 5),
        // (0, 5), (0, 5), (0, 5), (0, 5),
        // (0, 5), (0, 5), (0, 5), (0, 5),

        (5000, 5000), (5000, 5000), (5000, 5000), (5000, 5000), (5000, 5000),
        (0, 10000), (0, 10000), (0, 10000), (0, 10000), (0, 10000),

        // (0, 40000), (0, 40000), (0, 40000), (0, 40000), (0, 40000),
        // (0, 40000), (0, 40000), (0, 40000), (0, 40000), (0, 40000),

        (0, 100000), (0, 100000), (0, 100000), (0, 100000), (0, 100000),
        (0, 100000), (0, 100000), (0, 100000), (0, 100000), (0, 100000),

        // (40000, 10000), (80000, 10000), (80000, 10000), (80000, 10000),
        // (80000, 10000), (80000, 10000),
    ];

    let pattern_watch_list = vec![0, 1, 2, 3, 4];
    // let pattern_watch_list = vec![1, 7, 15];
    let mut trials = TrialResults::new(pattern_watch_list);

    let mut rng = rand::weak_rng();
    let mut exiting = false;
    let mut cycle_count_running_ttl = 0usize;

    for (t, (training_iters, collect_iters)) in training_collect_iters.into_iter().enumerate() {
        // let mut activity_counts = vec![vec![0; pattern_count]; area_cell_count];
        let mut trial_data = TrialData::new(pattern_count, area_cell_count);
        {
            let mut activity_counts = trial_data.activity_counts().clone().write().wait().unwrap();

            // cycle(&controls, &params, training_iters, collect_iters, pattern_count,
            //     &sdrs, &mut activity_counts, cycle_count_running_ttl);

            // Main loop:
            for i in 0..training_iters + collect_iters {
                let pattern_idx = if SEQUENTIAL_SDR {
                    // Write a non-random SDR:
                    i % pattern_count
                } else {
                    // Write a random SDR:
                    Range::new(0, pattern_count).ind_sample(&mut rng)
                };

                let mut guard = params.tract_buffer.clone().write().wait().unwrap();
                debug_assert!(guard.len() == sdrs[pattern_idx].len());

                for (src, dst) in sdrs[pattern_idx].iter().zip(guard.iter_mut()) {
                    *dst = *src;
                }

                WriteGuard::release(guard);

                // Cycle.
                controls.cmd_tx.send(Command::Iterate(1)).unwrap();

                // Wait for completion.
                finish_queues(controls, (cycle_count_running_ttl + i) as u64, &mut exiting);

                if i >= training_iters {
                    // Increment the cell activity counts.
                    let l4_axns = unsafe { params.l4_axns.map().read().enq().unwrap() };
                    for (counts, &axn) in activity_counts.iter_mut().zip(l4_axns.iter()) {
                        counts[pattern_idx] += (axn > 0) as usize;
                    }
                }

                if exiting { break; }
            }

            println!("\nActivity Counts [{}] (train: {}, collect: {}, running total: {}):",
                t, training_iters, collect_iters, cycle_count_running_ttl);

            // let _smoother_layers = 6;
            // let _energy_level_raw = _smoother_layers * cycle_count_running_ttl;
            // let _energy_level = if _energy_level_raw > 255 { 255 } else { _energy_level_raw as u8 };

            let mut den_activities = vec![0; buffers.l4_spt_den_actvs.len()];
            buffers.l4_spt_den_actvs.read(&mut den_activities).enq().unwrap();
            assert_eq!(den_activities.len(), activity_counts.len());

            let mut cel_activities = vec![0; buffers.l4_spt_cel_actvs.len()];
            buffers.l4_spt_cel_actvs.read(&mut cel_activities).enq().unwrap();
            assert_eq!(cel_activities.len(), activity_counts.len());

            let mut cel_energies = vec![0; buffers.l4_spt_cel_enrgs.len()];
            buffers.l4_spt_cel_enrgs.read(&mut cel_energies).enq().unwrap();
            assert_eq!(cel_energies.len(), activity_counts.len());


            print_activity_counts(&den_activities, &cel_activities, &cel_energies,
                activity_counts.as_slice(), /*_energy_level*/);

        }

        let trial_cycle_count = training_iters + collect_iters;
        cycle_count_running_ttl += trial_cycle_count;

        let cycles_per_pattern = collect_iters / pattern_count;
        const CUTOFF_QUOTIENT: usize = 16;
        let actv_cutoff = cycles_per_pattern / CUTOFF_QUOTIENT;

        trial_data.set_ttl_cycle_count(trial_cycle_count);
        trial_data.set_actv_cutoff(actv_cutoff);

        trials.add(trial_data);

        trials.print(trials.trials.len() - 1, cycles_per_pattern, actv_cutoff);
        println!("Prior Trial Consistencies: {:?}", trials.prior_trial_consistencies(t));
    }

    // println!("\nAll Trial Consistencies: {:?}", trials.all_past_consistencies());
    // trials.print_all();

    controls.cmd_tx.send(Command::Exit).unwrap();
    controls.cmd_tx.send(Command::None).unwrap();

    println!("\nSpatial evaluation complete.\n");
    // controls.cmd_tx.recv().unwrap();
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


/// The data collected from a trial.
#[derive(Debug, Clone)]
pub struct TrialData {
    activity_counts: QrwLock<Vec<Vec<ActivityCount>>>,
    ttl_cycle_count: Option<usize>,
    actv_cutoff: Option<usize>,
}

impl TrialData {
    pub fn new(pattern_count: usize, cell_count: usize) -> TrialData {
        let activity_counts = QrwLock::new(vec![vec![0; pattern_count]; cell_count]);

        TrialData {
            activity_counts,
            ttl_cycle_count: None,
            actv_cutoff: None,
        }
    }

    // pub fn activity_counts_mut(&mut self) -> &mut Vec<Vec<ActivityCount>> {
    //     &mut self.activity_counts
    // }

    pub fn activity_counts(&self) -> &QrwLock<Vec<Vec<ActivityCount>>> {
        &self.activity_counts
    }

    pub fn set_ttl_cycle_count(&mut self, ttl_cycle_count: usize) {
        self.ttl_cycle_count = Some(ttl_cycle_count);
    }

    pub fn set_actv_cutoff(&mut self, actv_cutoff: usize) {
        self.actv_cutoff = Some(actv_cutoff);
    }
}


/// A subcortical nucleus layer with a pathway.
#[derive(Debug)]
pub struct Layer {
    sub: SubcorticalNucleusLayer,
    pathway: Option<TractSender>,
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

    pub fn pathway(&self) -> Option<&TractSender> {
        self.pathway.as_ref()
    }
}


struct Samplers {
    l4_axns: TractReceiver,
    l4_den_actvs: TractReceiver,
    l4_cel_actvs: TractReceiver,
    l4_cel_enrgs: TractReceiver,
}

impl Samplers {

}


/// A `SubcorticalNucleus`.
struct EvalSpatial {
    area_name: String,
    // layers: HashMap<LayerAddress, SubcorticalNucleusLayer>,
    layers: HashMap<LayerAddress, Layer>,
    pattern_count: usize,
    area_cell_count: usize,
    input_sdrs: QrwLock<Vec<Vec<u8>>>,
    trial_iter: TrialIter,
    cycles_complete: usize,
    current_trial_data: TrialData,
    trial_results: TrialResults,
    work_pool_remote: WorkPoolRemote,
    rng: XorShiftRng,
    samplers: Option<Samplers>,
    current_pattern_idx: usize,
}

impl EvalSpatial {
    pub fn new<S: Into<String>>(layer_map_schemes: &LayerMapSchemeList,
            area_schemes: &AreaSchemeList, area_name: S, work_pool_remote: WorkPoolRemote)
            -> EvalSpatial {
        let area_name = area_name.into();
        let area_scheme = &area_schemes[&area_name];
        let layer_map_scheme = &layer_map_schemes[area_scheme.layer_map_name()];
        let layer_schemes: Vec<&LayerScheme> = layer_map_scheme.layers().iter().map(|ls| ls).collect();

        let mut layers = HashMap::with_capacity(4);

        for layer_scheme in layer_schemes.into_iter() {
            let lyr_name = layer_scheme.name();
            let lyr_addr = LayerAddress::new(area_scheme.area_id(), layer_scheme.layer_id());
            let axn_topology = layer_scheme.kind().axn_topology();
            let lyr_depth = layer_scheme.depth().expect("EvalSpatial::new: No layer depth set.");

            let dims = match axn_topology {
                AxonTopology::Spatial | AxonTopology::Nonspatial =>
                    area_scheme.dims().clone_with_depth(lyr_depth),
                AxonTopology::None => panic!("EvalSpatial::new: Invalid axon topology."),
            };

            let sub_layer = SubcorticalNucleusLayer::new(lyr_name, lyr_addr,
                layer_scheme.axn_domain().clone(), axn_topology, dims);

            let layer = Layer {
                sub: sub_layer,
                pathway: None,
            };

            layers.insert(lyr_addr.clone(), layer);
        }

        const SPARSITY: usize = 48;
        let pattern_count = 300;
        let cell_count = (ENCODE_DIM * ENCODE_DIM) as usize;
        let sdr_active_count = cell_count / SPARSITY;

        let mut rng = rand::weak_rng();

        // Produce randomized indexes:
        let pattern_indices: Vec<_> = (0..pattern_count).map(|_| {
            encode::gen_axn_idxs(&mut rng, sdr_active_count, cell_count)
        }).collect();

        // Create sdr from randomized indexes:
        let input_sdrs: Vec<_> = pattern_indices.iter().map(|axn_idxs| {
            let mut sdr = vec![0u8; cell_count];
            for &axn_idx in axn_idxs.iter() {
                sdr[axn_idx] = Range::new(96, 160).ind_sample(&mut rng);
            }
            sdr
        }).collect();

        let area_cell_count = (AREA_DIM * AREA_DIM) as usize;

        // Define the number of iters to first train then collect for each
        // sample period. All learning and other cell parameters (activity,
        // energy, etc.) persist between sample periods. Only collection
        // iters are recorded and evaluated.
        let trial_iter = TrialIter::new(vec![
            // (100, 100), (200, 200), (300, 300), (400, 400), (500, 500),
            (5000, 5000), (5000, 5000), (5000, 5000), (5000, 5000), (5000, 5000),

            // (40000, 10000), (80000, 10000), (80000, 10000), (80000, 10000),
            // (80000, 10000), (80000, 10000),
        ]);

        let pattern_watch_list = vec![0, 1, 2, 3, 4];
        let trial_results = TrialResults::new(pattern_watch_list);

        EvalSpatial {
            area_name,
            layers,
            pattern_count,
            area_cell_count,
            input_sdrs: QrwLock::new(input_sdrs),
            trial_iter,
            cycles_complete: 0,
            current_trial_data: TrialData::new(pattern_count, area_cell_count),
            trial_results,
            work_pool_remote,
            rng,
            samplers: None,
            current_pattern_idx: 0,
        }
    }
}

impl SubcorticalNucleus for EvalSpatial {
    fn create_pathways(&mut self, thal: &mut Thalamus,
            cortical_areas: &mut CorticalAreas) -> CmnResult<()> {
        return Ok(());

        // Wire up output (sdr) pathways.
        for layer in self.layers.values_mut() {
            let tx = thal.input_pathway(*layer.sub().addr(), true);
            layer.pathway = Some(tx);
        }

        let v1_l4_lyr_addr = *thal.area_maps().by_key(PRI_AREA).expect("invalid area")
            .layer_map().layers().by_key(SPT_LYR)
            .expect("bad lyr").layer_addr();

        let pri_area = cortical_areas.by_key_mut(PRI_AREA).unwrap();

        let l4_axns = pri_area.sampler(SamplerKind::Axons(Some(v1_l4_lyr_addr)),
                SamplerBufferKind::Single, true);

        // Layer 4 spatial dendrite activity ratings (pre-inhib):
        let l4_den_actvs = pri_area.sampler(SamplerKind::SscDenActivities(v1_l4_lyr_addr),
                SamplerBufferKind::Single, false);

        // Layer 4 spatial cell activity ratings (axon activity, post-inhib):
        let l4_cel_actvs = pri_area.sampler(SamplerKind::SscSomaActivities(v1_l4_lyr_addr),
                SamplerBufferKind::Single, false);

        // Layer 4 spatial cell energies (restlessness):
        let l4_cel_enrgs = pri_area.sampler(SamplerKind::SscSomaEnergies(v1_l4_lyr_addr),
                SamplerBufferKind::Single, false);

        self.samplers = Some(Samplers { l4_axns, l4_den_actvs, l4_cel_actvs,
            l4_cel_enrgs });

        Ok(())
    }

    /// Pre-cycle:
    ///
    /// * Writes output SDR to thalamic tract
    /// *
    ///
    fn pre_cycle(&mut self, _thal: &mut Thalamus, work_pool: &mut WorkPool) -> CmnResult<()> {
        return Ok(());

        // let pattern_idx = if SEQUENTIAL_SDR {
        self.current_pattern_idx = if SEQUENTIAL_SDR {
            // Write a non-random SDR:
            self.trial_iter.global_cycle_idx % self.pattern_count
        } else {
            // Write a random SDR:
            Range::new(0, self.pattern_count).ind_sample(&mut self.rng)
        };

        let pattern_idx = self.current_pattern_idx;

        // Write sdr to pathway:
        for layer in self.layers.values() {
            let pathway = layer.pathway.as_ref().expect("no pathway set");

            let future_sdrs = self.input_sdrs.clone().read().from_err();

            let future_write_guard = pathway.send()
                .map(|buf_opt| buf_opt.map(|buf| buf.write_u8()))
                .flatten();

            let future_write = future_write_guard
                .join(future_sdrs)
                .map(move |(tract_opt, sdrs)| {
                    tract_opt.map(|mut t| {
                        debug_assert!(t.len() == sdrs[pattern_idx].len());
                        t.copy_from_slice(&sdrs[pattern_idx]);
                    });
                })
                .map_err(|err| panic!("{}", err));

            work_pool.complete_work(future_write)?;
        }

        Ok(())
    }

    /// Post-cycle:
    ///
    /// * Blocks to wait for sampler channels
    /// * Increments the cell activity counts
    ///
    fn post_cycle(&mut self, _thal: &mut Thalamus, work_pool: &mut WorkPool) -> CmnResult<()> {
        return Ok(());

        if self.trial_iter.current_counter().is_collecting() {
            let pattern_idx = self.current_pattern_idx;

            let future_axns = self.samplers.as_ref().unwrap().l4_axns.recv(true)
                .wait()?.unwrap().read_u8();

            let future_activity_counts = self.current_trial_data.activity_counts().clone().write()
                .from_err();

                // let l4_axns = unsafe { params.l4_axns.map().read().enq().unwrap() };
                // for (counts, &axn) in activity_counts.iter_mut().zip(l4_axns.iter()) {
                //     counts[pattern_idx] += (axn > 0) as usize;
                // }

            let future_increment = future_axns.join(future_activity_counts)
                .map(move |(axns, mut actv_counts)| {
                    for (&axn, counts) in axns.iter().zip(actv_counts.iter_mut()) {
                        // if axn > 0 {
                        //     print!("{{COUNT:{}}}", counts[pattern_idx]);
                        // }
                        counts[pattern_idx] += (axn > 0) as usize;
                    }
                })
                .map_err(|err| panic!("{}", err));

            work_pool.complete_work(future_increment)?;
        }

        match self.trial_iter.incr() {
            IncrResult::TrialComplete { scheme_idx, train, collect } => {
                println!("\nActivity Counts [{}] (train: {}, collect: {}, running total: {}):",
                    scheme_idx, train, collect, self.trial_iter.global_cycle_idx);

                let future_den_activities = self.samplers.as_ref().unwrap().l4_den_actvs.recv(true)
                    .wait()?.unwrap().read_u8();
                let future_cel_activities = self.samplers.as_ref().unwrap().l4_cel_actvs.recv(true)
                    .wait()?.unwrap().read_u8();
                let future_cel_energies = self.samplers.as_ref().unwrap().l4_cel_enrgs.recv(true)
                    .wait()?.unwrap().read_u8();

                let future_activity_counts = self.current_trial_data.activity_counts().clone().read()
                    .from_err();

                // let _smoother_layers = 6;
                // let _energy_level_raw = _smoother_layers * cycle_count_running_ttl;
                // let _energy_level = if _energy_level_raw > 255 { 255 } else { _energy_level_raw as u8 };

                let future_print_activity = future_den_activities
                    .join4(future_cel_activities, future_cel_energies, future_activity_counts)
                    .map(|(den_actvs, cel_actvs, cel_enrgs, activity_counts)| {
                        print_activity_counts(den_actvs.as_slice(), cel_actvs.as_slice(), cel_enrgs.as_slice(),
                            activity_counts.as_slice(), /*_energy_level*/);
                    })
                    .map_err(|err| println!("{:?}", err));

                future_print_activity.wait()?;

                // work_pool.complete_work(future_print_activity)?;

                let trial_cycle_count = train + collect;

                let cycles_per_pattern = collect / self.pattern_count;
                const CUTOFF_QUOTIENT: usize = 16;
                let actv_cutoff = cycles_per_pattern / CUTOFF_QUOTIENT;

                self.current_trial_data.set_ttl_cycle_count(trial_cycle_count);
                self.current_trial_data.set_actv_cutoff(actv_cutoff);
                let completed_trial_data = mem::replace(&mut self.current_trial_data,
                    TrialData::new(self.pattern_count, self.area_cell_count));
                self.trial_results.add(completed_trial_data);

                self.trial_results.print(self.trial_results.trials.len() - 1, cycles_per_pattern, actv_cutoff);
                println!("Prior Trial Consistencies: {:?}", self.trial_results
                    .prior_trial_consistencies(self.trial_results.trials.len() - 1));
            },
            _ir @ _ => {
                if self.trial_iter.current_counter.is_last_cycle() {
                    // Clear all of the sampler tract buffers in prep. for final cycle.
                    let future_den_activities = self.samplers.as_ref().unwrap().l4_den_actvs.recv(true)
                        .wait()?.unwrap().read_u8();
                    let future_cel_activities = self.samplers.as_ref().unwrap().l4_cel_actvs.recv(true)
                        .wait()?.unwrap().read_u8();
                    let future_cel_energies = self.samplers.as_ref().unwrap().l4_cel_enrgs.recv(true)
                        .wait()?.unwrap().read_u8();
                    let joined = future_den_activities.join3(future_cel_activities, future_cel_energies)
                        .map(|_| ())
                        .map_err(|err| panic!("{}", err));
                    work_pool.complete(joined)?;
                }
            },
        }

        Ok(())
    }

    fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer> {
        self.layers.get(&addr).map(|l| l.sub())
    }

    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }
}


pub fn eval() {
    let layer_map_schemes = define_lm_schemes();
    let area_schemes = define_a_schemes();

    let cortex_builder = Cortex::builder(layer_map_schemes, area_schemes)
        .ca_settings(ca_settings());

    let work_pool_remote = cortex_builder.get_work_pool_remote();

    // let input_gen = InputGenerator::new(cortex_builder.get_layer_map_schemes(),
    //     cortex_builder.get_area_schemes(), IN_AREA).unwrap();
    // let cortex_builder = cortex_builder.subcortical_nucleus(input_gen);

    let eval_nucl = EvalSpatial::new(cortex_builder.get_layer_map_schemes(),
        cortex_builder.get_area_schemes(), IN_AREA, work_pool_remote);
    let cortex_builder = cortex_builder.subcortical_nucleus(eval_nucl);

    let cortex = cortex_builder.build().unwrap();

    let controls = ::spawn_threads(cortex, PRI_AREA);

    //////////////

    let v0_ext_lyr_addr = *cortex.thal().area_maps().by_key(IN_AREA).expect("bad area")
        .layer_map().layers().by_key(EXT_LYR).expect("bad lyr").layer_addr();

    let v1_spt_lyr_buf = {
        let pri_area_map = cortex.thal().area_maps().by_key(PRI_AREA).expect("bad area");
        let v1_spt_lyr_addr = *pri_area_map.layer_map().layers().by_key(SPT_LYR)
            .expect("bad lyr").layer_addr();
        let v1_spt_lyr_axn_range = pri_area_map.lyr_axn_range(&v1_spt_lyr_addr, None).unwrap();
        // println!("######## v1_spt_lyr_axn_range: {:?}", v1_spt_lyr_axn_range);
        cortex.areas().by_key(PRI_AREA).unwrap().axns()
            .create_sub_buffer(&v1_spt_lyr_axn_range).unwrap()
    };

    // Layer 4 spatial dendrite activity ratings (pre-inhib):
    let l4_spt_den_actvs = cortex.areas().by_key(PRI_AREA).unwrap()
        .ssc_layer(SPT_LYR).unwrap().dens().activities().clone();

    // Layer 4 spatial cell activity ratings (axon activity, post-inhib):
    let l4_spt_cel_actvs = cortex.areas().by_key(PRI_AREA).unwrap()
        .ssc_layer(SPT_LYR).unwrap().activities().clone();

    // Layer 4 spatial cell energies (restlessness):
    let l4_spt_cel_enrgs = cortex.areas().by_key(PRI_AREA).unwrap()
        .ssc_layer(SPT_LYR).unwrap().energies().clone();

    let in_tract_idx = cortex.thal().tract().index_of(v0_ext_lyr_addr).unwrap();
    let in_tract_buffer = cortex.thal().tract().buffer_rwvec(in_tract_idx).unwrap().clone();
    let axns = cortex.areas().by_key(PRI_AREA).unwrap().axns().states().clone();
    let area_map = cortex.areas().by_key(PRI_AREA).unwrap().area_map().clone();

    let _params = Params {
        tract_buffer: in_tract_buffer,
        axns,
        l4_axns: v1_spt_lyr_buf,
        area_map,
        encode_dim: ENCODE_DIM,
        area_dim: AREA_DIM,
    };

    let _buffers = Buffers {
        l4_spt_den_actvs,
        l4_spt_cel_actvs,
        l4_spt_cel_enrgs,
    };

    // Get the flywheel moving:
    controls.cmd_tx.send(Command::None).unwrap();
    // Sleep until vibi window opens (need a better mech. for this):
    ::std::thread::sleep(::std::time::Duration::new(1, 0));

    track_pattern_activity(&controls, _params, _buffers);

    controls.req_tx.send(Request::FinishQueues).unwrap();
    controls.cmd_tx.send(Command::None).unwrap();

    /////

    ::join_threads(controls)
}

fn define_lm_schemes() -> LayerMapSchemeList {
    let at0 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            // .input_layer("aff_in", LayerTags::DEFAULT,
            //     AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]),
            //     AxonTopology::Spatial
            //     // AxonTopology::Nonspatial
            // )
            .layer(LayerScheme::define("aff_in")
                .axonal(AxonTopology::Spatial)
                .axon_domain(AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]))
            )
            .layer("dummy_out", 1, LayerTags::DEFAULT, AxonDomain::output(&[AxonTag::unique()]),
                LayerKind::Axonal(AxonTopology::Spatial)
            )
            .layer(SPT_LYR, 1, LayerTags::PSAL, AxonDomain::Local,
            // .layer(SPT_LYR, 1, LayerTags::PSAL, AxonDomain::output(&[map::THAL_SP]),
                CellScheme::spiny_stellate(&[("aff_in", 7, 1)], 5, 000)
            )
            .layer("iv_inhib", 0, LayerTags::DEFAULT, AxonDomain::Local, CellScheme::inhib(SPT_LYR, 4, 0))
            .layer("iv_smooth", 0, LayerTags::DEFAULT, AxonDomain::Local, CellScheme::smooth(SPT_LYR, 4, 1))
            // .layer("iii", 1, LayerTags::PTAL, AxonDomain::Local,
            .layer("iii", 1, LayerTags::PTAL, AxonDomain::output(&[AxonTag::unique()]),
                CellScheme::pyramidal(&[("iii", 5, 1)], 1, 2, 500)
            )
            .layer("iii_output", 0, LayerTags::DEFAULT, AxonDomain::Local,
                CellScheme::pyr_outputter("iii", 0)
            )
            // .layer("mcols", 1, LayerTags::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
            //     CellScheme::minicolumn(9999)
            // )
        )
        .lmap(LayerMapScheme::new("v0_lm", LayerMapKind::Subcortical)
            .layer(EXT_LYR, 1, LayerTags::DEFAULT,
                AxonDomain::output(&[map::THAL_SP, at0]),
                LayerKind::Axonal(AxonTopology::Spatial))
        )
}

fn define_a_schemes() -> AreaSchemeList {
    AreaSchemeList::new()
        .area(AreaScheme::new(IN_AREA, "v0_lm", ENCODE_DIM)
            .subcortex()
        )
        .area(AreaScheme::new(PRI_AREA, "visual", AREA_DIM)
            .eff_areas(vec![IN_AREA])
        )
}

pub fn ca_settings() -> CorticalAreaSettings {
    #[allow(unused_imports)]
    use vibi::bismit::ocl::builders::BuildOpt;

    CorticalAreaSettings::new()
        // .bypass_inhib()
        // .bypass_filters()
        // .disable_pyrs()
        // .disable_ssts()
        // .disable_mcols()
        // .disable_regrowth()
        // .disable_learning()
        // .build_opt(BuildOpt::cmplr_def("DEBUG_SMOOTHER_OVERLAP", 1))
}