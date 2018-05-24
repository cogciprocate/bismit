#![allow(dead_code)]

use std::mem;
use std::collections::{BTreeMap, HashMap};
use rand::{FromEntropy, rngs::SmallRng};
use rand::distributions::{Range, Distribution};
use qutex::QrwLock;
use vibi::bismit::futures::{executor, FutureExt};
use vibi::bismit::{map, encode, Result as CmnResult, Cortex, CorticalAreaSettings, Thalamus,
    SubcorticalNucleus, SubcorticalNucleusLayer, CompletionPool, /*CompletionPoolRemote,*/ TractReceiver,
    SamplerKind, SamplerBufferKind, CorticalAreas};
use vibi::bismit::map::*;
use ::{IncrResult, TrialIter, Layer, Pathway};

static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 48;
const AREA_DIM: u32 = 16;
const SEQUENTIAL_SDR: bool = true;

pub type CellIdx = usize;
pub type ActivityCount = usize;
pub type ActiveCells = BTreeMap<CellIdx, ActivityCount>;
pub type PatternIdx = usize;
pub type PatternAssociations = BTreeMap<PatternIdx, ActiveCells>;


#[derive(Clone, Debug)]
pub struct TrialResults {
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
        let activity_counts = executor::block_on(trial_data.activity_counts().clone().read()).unwrap();

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
    pub fn trial_consistency(&self, trial_a_idx: usize, trial_b_idx: usize, ignore_inactive: bool) -> f32 {
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

    pub fn trials(&self) -> &[PatternAssociations] {
        &self.trials
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
pub fn print_activity_counts(den_activities: &[u8], cel_activities: &[u8], cel_energies: &[u8],
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

        if (cel_idx & 7) == 0 {
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

    // TODO: Change to: https://en.wikipedia.org/wiki/Coefficient_of_variation
    // (or just Mean +/- SD)
    let stdev = (sq_diff_ttl / ttl_count).sqrt();
    println!("Standard deviation: {}", stdev);
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



struct Samplers {
    l4_axns: TractReceiver,
    l4_den_actvs: TractReceiver,
    l4_cel_actvs: TractReceiver,
    l4_cel_enrgs: TractReceiver,
}


/// A `SubcorticalNucleus` which runs several evaluations of a spiny stellate
/// cell layer and its accompanying control cells (smoother).
struct EvalSpatial {
    area_name: String,
    area_id: usize,
    layers: HashMap<LayerAddress, Layer>,
    pattern_count: usize,
    area_cell_count: usize,
    input_sdrs: QrwLock<Vec<Vec<u8>>>,
    trial_iter: TrialIter,
    cycles_complete: usize,
    current_trial_data: TrialData,
    current_pattern_idx: usize,
    trial_results: TrialResults,
    // completion_pool_remote: CompletionPoolRemote,
    rng: SmallRng,
    samplers: Option<Samplers>,
}

impl EvalSpatial {
    pub fn new<S: Into<String>>(layer_map_schemes: &LayerMapSchemeList,
            area_schemes: &AreaSchemeList, area_name: S, /*completion_pool_remote: CompletionPoolRemote*/)
            -> EvalSpatial {
        let area_name = area_name.into();
        let area_scheme = &area_schemes[&area_name];
        let layer_map_scheme = &layer_map_schemes[area_scheme.layer_map_name()];
        let mut layers = HashMap::with_capacity(4);

        for layer_scheme in layer_map_scheme.layers() {
            let sub_layer = SubcorticalNucleusLayer::from_schemes(layer_scheme, area_scheme, None);

            let layer = Layer {
                sub: sub_layer,
                pathway: Pathway::None,
            };

            layers.insert(layer.sub().addr().clone(), layer);
        }

        const SPARSITY: usize = 48;
        let pattern_count = 300;
        let cell_count = (ENCODE_DIM * ENCODE_DIM) as usize;
        let sdr_active_count = cell_count / SPARSITY;

        let mut rng = SmallRng::from_entropy();

        // Produce randomized indexes:
        let pattern_indices: Vec<_> = (0..pattern_count).map(|_| {
            encode::gen_axn_idxs(&mut rng, sdr_active_count, cell_count)
        }).collect();

        // Create sdr from randomized indexes:
        let input_sdrs: Vec<_> = pattern_indices.iter().map(|axn_idxs| {
            let mut sdr = vec![0u8; cell_count];
            for &axn_idx in axn_idxs.iter() {
                sdr[axn_idx as usize] = Range::new(96, 160).sample(&mut rng);
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
            area_name: area_name,
            area_id: area_scheme.area_id(),
            layers,
            pattern_count,
            area_cell_count,
            input_sdrs: QrwLock::new(input_sdrs),
            trial_iter,
            cycles_complete: 0,
            current_trial_data: TrialData::new(pattern_count, area_cell_count),
            current_pattern_idx: 0,
            trial_results,
            // completion_pool_remote,
            rng,
            samplers: None,
        }
    }
}

impl SubcorticalNucleus for EvalSpatial {
    fn create_pathways(&mut self, thal: &mut Thalamus,
            cortical_areas: &mut CorticalAreas) -> CmnResult<()> {
        // Wire up I/O pathways.
        for layer in self.layers.values_mut() {
            layer.pathway = Pathway::new(thal, layer.sub());
        }

        // let v1_l4_lyr_addr = thal.area_maps().by_key(PRI_AREA).expect("invalid area")
        //     .layer_map().layers().by_key(SPT_LYR)
        //     .expect("bad lyr").layer_addr();
        let v1_l4_lyr_addr = thal.layer_addr(PRI_AREA, SPT_LYR);

        let pri_area = cortical_areas.by_key_mut(PRI_AREA).unwrap();

        let l4_axns = pri_area.sampler(SamplerKind::Axons(Some(v1_l4_lyr_addr)),
                SamplerBufferKind::Single, true);

        // Layer 4 spatial dendrite activity ratings (pre-inhib):
        let l4_den_actvs = pri_area.sampler(SamplerKind::DenActivities(v1_l4_lyr_addr),
                SamplerBufferKind::Single, false);

        // Layer 4 spatial cell activity ratings (axon activity, post-inhib):
        let l4_cel_actvs = pri_area.sampler(SamplerKind::SomaActivities(v1_l4_lyr_addr),
                SamplerBufferKind::Single, false);

        // Layer 4 spatial cell energies (restlessness):
        let l4_cel_enrgs = pri_area.sampler(SamplerKind::SomaEnergies(v1_l4_lyr_addr),
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
    fn pre_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            completion_pool: &mut CompletionPool) -> CmnResult<()> {
        self.current_pattern_idx = if SEQUENTIAL_SDR {
            // Write a non-random SDR:
            self.trial_iter.global_cycle_idx % self.pattern_count
        } else {
            // Write a random SDR:
            Range::new(0, self.pattern_count).sample(&mut self.rng)
        };

        let pattern_idx = self.current_pattern_idx;

        // Write sdr to pathway:
        for layer in self.layers.values() {
            if let Pathway::Output { ref tx } = layer.pathway {
                let future_sdrs = self.input_sdrs.clone().read()
                    .map_err(|err| err.into());

                let future_write_guard = tx.send()
                    .map(|buf_opt| buf_opt.map(|buf| buf.write_u8()))
                    .map_err(|err| err.into())
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

                completion_pool.complete_work(Box::new(future_write))?;
            }
        }

        Ok(())
    }

    /// Post-cycle:
    ///
    /// * Blocks to wait for sampler channels
    /// * Increments the cell activity counts
    ///
    fn post_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            completion_pool: &mut CompletionPool) -> CmnResult<()> {
        if self.trial_iter.current_counter().is_collecting() {
            let pattern_idx = self.current_pattern_idx;

            let future_axns = self.samplers.as_ref().unwrap().l4_axns.recv(true)
                .wait()?.unwrap().read_u8();

            let future_activity_counts = self.current_trial_data.activity_counts().clone().write()
                .err_into();

            let future_increment = future_axns.join(future_activity_counts)
                .map(move |(axns, mut actv_counts)| {
                    for (&axn, counts) in axns.iter().zip(actv_counts.iter_mut()) {
                        counts[pattern_idx] += (axn > 0) as usize;
                    }
                })
                .map_err(|err| panic!("{}", err));

            completion_pool.complete_work(Box::new(future_increment))?;
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
                    .err_into();

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

                executor::block_on(future_print_activity)?;

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
                    let _future_den_activities = self.samplers.as_ref().unwrap().l4_den_actvs.recv(true)
                        .wait()?.unwrap().read_u8();
                    let _future_cel_activities = self.samplers.as_ref().unwrap().l4_cel_actvs.recv(true)
                        .wait()?.unwrap().read_u8();
                    let _future_cel_energies = self.samplers.as_ref().unwrap().l4_cel_enrgs.recv(true)
                        .wait()?.unwrap().read_u8();
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

    fn area_id(&self) -> usize {
        self.area_id
    }
}


pub fn eval() {
    let layer_map_schemes = define_lm_schemes();
    let area_schemes = define_a_schemes();

    let cortex_builder = Cortex::builder(layer_map_schemes, area_schemes)
        .ca_settings(ca_settings());

    // let completion_pool_remote = cortex_builder.get_completion_pool_remote();

    let eval_nucl = EvalSpatial::new(cortex_builder.get_layer_map_schemes(),
        cortex_builder.get_area_schemes(), IN_AREA, /*completion_pool_remote*/);
    let cortex_builder = cortex_builder.subcortical_nucleus(eval_nucl);

    let cortex = cortex_builder.build().unwrap();

    let controls = ::spawn_threads(cortex, PRI_AREA, true);

    ::join_threads(controls)
}

fn define_lm_schemes() -> LayerMapSchemeList {
    let at0 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            .layer(LayerScheme::define("aff_in")
                .axonal(AxonTopology::Spatial)
                .axon_domain(AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]))
            )
            .layer(LayerScheme::define(SPT_LYR)
                .depth(3)
                .tags(LayerTags::PSAL)
                .axon_domain(AxonDomain::output(&[map::THAL_SP]))
                .cellular(CellScheme::spiny_stellate()
                    .tft(TuftScheme::basal().proximal()
                        .syns_per_den(32)
                        .src_lyr(TuftSourceLayer::define("aff_in")
                            .syn_reach(7)
                            .prevalence(1)
                        )
                    )
                )
            )
            .layer(LayerScheme::define("iv_inhib")
                .cellular(CellScheme::control(
                        ControlCellKind::InhibitoryBasketSurround {
                            host_lyr_name: SPT_LYR.into(),
                            field_radius: 4,
                        },
                        0
                    )
                )
            )
            .layer(LayerScheme::define("iv_smooth")
                .cellular(CellScheme::control(
                        ControlCellKind::ActivitySmoother {
                            host_lyr_name: SPT_LYR.into(),
                            field_radius: 4,
                        },
                        1
                    )
                )
            )
            .layer(LayerScheme::define("iii")
                .depth(3)
                .tags(LayerTags::PTAL)
                .axon_domain(AxonDomain::output(&[AxonTag::unique()]))
                .cellular(CellScheme::pyramidal()
                    .tft(TuftScheme::basal().proximal()
                        .syns_per_den(3)
                        .src_lyr(TuftSourceLayer::define(SPT_LYR)
                            .syn_reach(0)
                            .prevalence(1)
                        )
                    )
                    .tft(TuftScheme::basal().distal()
                        .dens_per_tft(16)
                        .syns_per_den(32)
                        .max_active_dens_l2(0)
                        .thresh_init(0)
                        .src_lyr(TuftSourceLayer::define("iii")
                            .syn_reach(8)
                            .prevalence(1)
                        )
                    )
                )
            )
            .layer(LayerScheme::define("iii_inhib_col")
                .cellular(CellScheme::control(
                        ControlCellKind::IntraColumnInhib {
                            host_lyr_name: "iii".into(),
                        },
                        0
                    )
                )
            )
        )
        .lmap(LayerMapScheme::new("v0_lm", LayerMapKind::Subcortical)
            .layer(LayerScheme::define(EXT_LYR)
                .depth(1)
                .axonal(AxonTopology::Spatial)
                .axon_domain(AxonDomain::output(&[map::THAL_SP, at0]))
            )
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