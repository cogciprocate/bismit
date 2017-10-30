
use std::collections::{BTreeMap, HashMap};
use rand;
use rand::distributions::{Range, IndependentSample};
use vibi::bismit::map::*;
use vibi::bismit::ocl::{Buffer, WriteGuard};
use vibi::bismit::{map, Cortex, CorticalAreaSettings, Subcortex, InputGenerator};
use vibi::bismit::flywheel::{Command, Request, Response};
use vibi::bismit::encode::{self};
use ::{Controls, Params};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 48;
const AREA_DIM: u32 = 16;
const SEQUENTIAL_SDR: bool = true;
// const DEBUG_SMOOTHER_OVERLAP: bool = true;

struct Buffers {
    pub l4_spt_den_actvs: Buffer<u8>,
    pub l4_spt_cel_actvs: Buffer<u8>,
    pub l4_spt_cel_enrgs: Buffer<u8>,
}


// // TODO: DO SOMETHING WITH ME
// pub(crate) struct Nucleus {
//     area_name: String,
//     layers: HashMap<LayerAddress, SubcorticalNucleusLayer>,
// }

// impl Nucleus {
//     pub fn new<S: Into<String>>(area_name: S, _lyr_name: &'static str, _tar_area: &'static str)
//             -> Nucleus {
//         let area_name = area_name.into();

//         // let v0_ext_lyr_addr = *cortex.areas().by_key(area_name.as_str()).unwrap()
//         //     .area_map().layer_map().layers().by_key(lyr_name).unwrap().layer_addr();
//         // let v1_in_lyr_buf = cortex.areas().by_key(tar_area).unwrap()
//         //     .axns().create_layer_sub_buffer(v0_ext_lyr_addr, AxonDomainRoute::Input);
//         // let axns = cortex.areas().by_key(tar_area).unwrap()
//         //     .axns().states().clone();
//         // let area_map = cortex.areas().by_key(area_name.as_str()).unwrap()
//         //     .area_map().clone();

//         Nucleus {
//             area_name: area_name.into(),
//             layers: HashMap::new(),
//         }
//     }
// }

// // TODO: DO SOMETHING WITH ME
// impl SubcorticalNucleus for Nucleus {
//     fn area_name<'a>(&'a self) -> &'a str { &self.area_name }
//     fn pre_cycle(&mut self, _thal: &mut Thalamus) {}
//     fn post_cycle(&mut self, _thal: &mut Thalamus) {}

//     fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer> {
//         self.layers.get(&addr)
//             // .expect(&format!("SubcorticalNucleus::layer(): Invalid addr: {:?}", addr))
//     }
// }



type CellIdx = usize;
type ActivityCount = usize;
type ActiveCells = BTreeMap<CellIdx, ActivityCount>;
type PatternIdx = usize;
type PatternAssociations = BTreeMap<PatternIdx, ActiveCells>;

struct Trials {
    trials: Vec<PatternAssociations>,
    trial_cycle_counts: Vec<usize>,
    pattern_watch_list: Vec<usize>,
}

impl Trials {
    pub fn new(pattern_watch_list: Vec<PatternIdx>) -> Trials {
        Trials {
            trials: Vec::with_capacity(16),
            trial_cycle_counts: Vec::with_capacity(16),
            pattern_watch_list,
        }
    }

    /// Adds the cell activity counts for each pattern in the watch list.
    ///
    /// Only adds those cells with activity counts above `actv_cutoff`, if
    /// specified.
    pub fn add(&mut self, counts: &Vec<Vec<ActivityCount>>, ttl_cycle_count: usize, actv_cutoff: Option<usize>) {
        // let mut active_cells = ActiveCells::new();
        let mut pattern_assoc = PatternAssociations::new();

        for &pattern_idx in &self.pattern_watch_list {
            let mut active_cells = ActiveCells::new();

            for (cell_idx, cell) in counts.iter().enumerate() {
                let cell_actv_cnt = cell[pattern_idx];

                match actv_cutoff {
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
        self.trial_cycle_counts.push(ttl_cycle_count);
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
        // for ((&pat_idx_a, active_cells_a), (&pat_idx_b, active_cells_b)) in trial_a.iter().zip(trial_b.iter().rev()) {
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


fn finish_queues(controls: &Controls, i: u64, exiting: &mut bool) {
    // controls.req_tx.send(Request::FinishQueues).unwrap();
    // controls.cmd_tx.send(Command::None).unwrap();

    // Wait for completion.
    loop {
        // println!(">>>>>> Attempting to receive...");
        debug!("Attempting to receive...");
        match controls.res_rx.recv() {
            Ok(res) => match res {
                Response::Status(status) => {
                    debug!("Status: {:?}", status);
                    // println!(">>>>>> Response::Status({:?})", status);
                    if status.cycle_counter.0 == i + 1 {
                        // println!(">>>>> Waiting for completion for cycle: {}", i + 1);
                        controls.req_tx.send(Request::FinishQueues).unwrap();
                        controls.cmd_tx.send(Command::None).unwrap();
                    }
                },
                Response::QueuesFinished(qf_i) => {
                    if qf_i == i + 1 {
                        // println!(">>>>> Queues finished for cycle: {}", qf_i);
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

fn cycle(controls: &Controls, params: &Params, training_iters: usize, collect_iters: usize,
        pattern_count: usize, sdrs: &Vec<Vec<u8>>, activity_counts: &mut Vec<Vec<usize>>,
        prev_elapsed_iters: usize)
{
    let mut rng = rand::weak_rng();
    let mut exiting = false;

    // Main loop:
    for i in 0..training_iters + collect_iters {
        let pattern_idx = if SEQUENTIAL_SDR {
            // Write a non-random SDR:
            i % pattern_count
        } else {
            // Write a random SDR:
            Range::new(0, pattern_count).ind_sample(&mut rng)
        };

        // println!(" (0.0-WriteStart...) ");

        // debug!("Locking tract buffer...");
        let mut guard = params.tract_buffer.clone().write().wait().unwrap();
        debug_assert!(guard.len() == sdrs[pattern_idx].len());

        // println!(" (1.0-WriteLocked) ");

        for (src, dst) in sdrs[pattern_idx].iter().zip(guard.iter_mut()) {
            *dst = *src;
        }

        // println!(" (1.1-WriteComplete) ");

        WriteGuard::release(guard);

        // ::std::thread::sleep(::std::time::Duration::from_millis(10));

        // println!(" (1.2-WriteReleased) ");

        // Cycle.
        controls.cmd_tx.send(Command::Iterate(1)).unwrap();

        // println!(" (1.3-FinishingQueues...) ");

        // Wait for completion.
        finish_queues(controls, (prev_elapsed_iters + i) as u64, &mut exiting);

        // ::std::thread::sleep(::std::time::Duration::from_millis(50));
        // println!(" (3.0-QueuesFinished) ");

        if i >= training_iters {
            // Increment the cell activity counts.
            let l4_axns = unsafe { params.l4_axns.map().read().enq().unwrap() };
            for (counts, &axn) in activity_counts.iter_mut().zip(l4_axns.iter()) {
                counts[pattern_idx] += (axn > 0) as usize;
            }
        }

        if exiting { break; }
    }

    // print!("\n");
}

// Prints dendritic and cell activity ratings as well as a total activity
// count for a selection of cells (currently every 8th).
//
// `_energy_level` can be used to make sure that all cells are being processed
// uniformly by the smoother kernel (by using the '+1 to all' debug code
// contained within).
fn print_activity_counts(buffers: &Buffers, activity_counts: &Vec<Vec<usize>>, _energy_level: u8) {
    let cel_count = activity_counts.len();
    let pattern_count = activity_counts[0].len();
    let mut cel_ttls = Vec::with_capacity(cel_count);
    let mut _non_zero_ptrn_ttls: Vec<(usize, usize)> = Vec::with_capacity(pattern_count);
    let mut ttl_count = 0f32;

    let mut den_activities = vec![0; buffers.l4_spt_den_actvs.len()];
    buffers.l4_spt_den_actvs.read(&mut den_activities).enq().unwrap();
    assert_eq!(den_activities.len(), activity_counts.len());

    let mut cel_activities = vec![0; buffers.l4_spt_cel_actvs.len()];
    buffers.l4_spt_cel_actvs.read(&mut cel_activities).enq().unwrap();
    assert_eq!(cel_activities.len(), activity_counts.len());

    let mut cel_energies_vec = vec![0; buffers.l4_spt_cel_enrgs.len()];
    buffers.l4_spt_cel_enrgs.read(&mut cel_energies_vec).enq().unwrap();
    assert_eq!(cel_energies_vec.len(), activity_counts.len());

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
            printc!(green: "{}", cel_energies_vec[cel_idx]);
            print!(",ct:");
            printc!(royal_blue: "{}", cel_ttl);
            print!("}} ");

            printed += 1;
        }

        // if cel_ttl > _min {
        //     println!("Cell [{}]({}): {:?}", cel_idx, cel_ttl, _non_zero_ptrn_ttls);
        // }

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

    let cell_count = (params.area_dim * params.area_dim) as usize;

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

        (0, 10000), (0, 10000), (0, 10000), (0, 10000), (0, 10000),
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
    let mut trials = Trials::new(pattern_watch_list);

    let mut cycle_count_running_ttl = 0usize;

    for (t, (training_iters, collect_iters)) in training_collect_iters.into_iter().enumerate() {
        let mut activity_counts = vec![vec![0; pattern_count]; cell_count];

        cycle(&controls, &params, training_iters, collect_iters, pattern_count,
            &sdrs, &mut activity_counts, cycle_count_running_ttl);

        let trial_cycle_count = training_iters + collect_iters;
        cycle_count_running_ttl += trial_cycle_count;
        println!("\nActivity Counts [{}] (train: {}, collect: {}, running total: {}):",
            t, training_iters, collect_iters, cycle_count_running_ttl);

        let _smoother_layers = 6;
        let _energy_level_raw = _smoother_layers * cycle_count_running_ttl;
        let _energy_level = if _energy_level_raw > 255 { 255 } else { _energy_level_raw as u8 };

        print_activity_counts(&buffers, &activity_counts, _energy_level);
        let cycles_per_pattern = collect_iters / pattern_count;
        const CUTOFF_QUOTIENT: usize = 16;
        let actv_cutoff = cycles_per_pattern / CUTOFF_QUOTIENT;
        trials.add(&activity_counts, trial_cycle_count, Some(actv_cutoff));
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

pub fn eval() {
    let layer_map_schemes = define_lm_schemes();
    let area_schemes = define_a_schemes();

    let input_gen = InputGenerator::new(&layer_map_schemes[&area_schemes[IN_AREA].layer_map_name()],
        &area_schemes[IN_AREA]).unwrap();
    let subcortex = Subcortex::new().nucleus(input_gen);

    let cortex = Cortex::builder(layer_map_schemes, area_schemes)
        .ca_settings(ca_settings())
        .sub(subcortex)
        .build().unwrap();

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

    // // TODO: DO SOMETHING WITH ME
    // let nucl = Nucleus::new(IN_AREA, EXT_LYR, PRI_AREA, &cortex);
    // cortex.add_subcortex(Subcortex::new().nucl(nucl));

    let controls = ::spawn_threads(cortex, PRI_AREA);

    let params = Params {
        tract_buffer: in_tract_buffer,
        axns,
        l4_axns: v1_spt_lyr_buf,
        area_map,
        encode_dim: ENCODE_DIM,
        area_dim: AREA_DIM,
    };

    let buffers = Buffers {
        l4_spt_den_actvs,
        l4_spt_cel_actvs,
        l4_spt_cel_enrgs,
    };

    track_pattern_activity(&controls, params, buffers);

    // if let Err(e) = controls.th_win.join() { println!("th_win.join(): Error: '{:?}'", e); }
    // if let Err(e) = controls.th_flywheel.join() { println!("th_flywheel.join(): Error: '{:?}'", e); }
    ::join_threads(controls)
}

fn define_lm_schemes() -> LayerMapSchemeList {
    let at0 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            .input_layer("aff_in", LayerTags::DEFAULT,
                AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]),
                AxonTopology::Spatial
                // AxonTopology::Horizontal
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
        .disable_mcols()
        // .disable_regrowth()
        // .disable_learning()
        // .build_opt(BuildOpt::cmplr_def("DEBUG_SMOOTHER_OVERLAP", 1))
}