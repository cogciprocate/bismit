
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver, /*TryRecvError*/};
use rand;
use rand::distributions::{Range, IndependentSample};
use vibi::window;
use vibi::bismit::map::*;
use vibi::bismit::flywheel::Flywheel;
use vibi::bismit::ocl::{Buffer, RwVec, WriteGuard};
use vibi::bismit::{map, Cortex, Thalamus, SubcorticalNucleus, CorticalAreaSettings, Subcortex};
use vibi::bismit::flywheel::{Command, Request, Response};
use vibi::bismit::map::{/*AxonDomainRoute,*/ AreaMap};
use vibi::bismit::encode::{self, /*ScalarSdrWriter*/};


pub struct Params {
    pub cmd_tx: Sender<Command>,
    pub req_tx: Sender<Request>,
    pub res_rx: Receiver<Response>,
    pub tract_buffer: RwVec<u8>,
    pub axns: Buffer<u8>,
    pub l4_axns: Buffer<u8>,
    // pub l4_spt_den_acts: Buffer<u8>,
    pub area_map: AreaMap,
    pub encode_dim: u32,
    pub area_dim: u32,
}


pub(crate) struct Nucleus {
    area_name: String,
}

impl Nucleus {
    pub fn new<S: Into<String>>(area_name: S, _lyr_name: &'static str, _tar_area: &'static str,
            _cortex: &Cortex) -> Nucleus
    {
        let area_name = area_name.into();

        // let v0_ext_lyr_addr = *cortex.areas().by_key(area_name.as_str()).unwrap()
        //     .area_map().layer_map().layers().by_key(lyr_name).unwrap().layer_addr();
        // let v1_in_lyr_buf = cortex.areas().by_key(tar_area).unwrap()
        //     .axns().create_layer_sub_buffer(v0_ext_lyr_addr, AxonDomainRoute::Input);
        // let axns = cortex.areas().by_key(tar_area).unwrap()
        //     .axns().states().clone();
        // let area_map = cortex.areas().by_key(area_name.as_str()).unwrap()
        //     .area_map().clone();

        Nucleus {
            area_name: area_name.into()
        }
    }
}


impl SubcorticalNucleus for Nucleus {
    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }

    fn pre_cycle(&mut self, _thal: &mut Thalamus) {

    }

    fn post_cycle(&mut self, _thal: &mut Thalamus) {

    }
}

// Prints dendritic and cell activity ratings as well as a total activity
// count for a selection of cells (currently every 8th).
fn print_activity_counts(den_actvs: &Buffer<u8>, cel_actvs: &Buffer<u8>,
        activity_counts: &Vec<Vec<usize>>)
{
    let cel_count = activity_counts.len();
    let pattern_count = activity_counts[0].len();
    let mut cel_ttls = Vec::with_capacity(cel_count);
    let mut _non_zero_ptrn_ttls: Vec<(usize, usize)> = Vec::with_capacity(pattern_count);
    let mut ttl_count = 0f32;

    let mut den_activities = vec![0; den_actvs.len()];
    den_actvs.read(&mut den_activities).enq().unwrap();
    assert_eq!(den_activities.len(), activity_counts.len());

    let mut cel_activities = vec![0; cel_actvs.len()];
    cel_actvs.read(&mut cel_activities).enq().unwrap();
    assert_eq!(cel_activities.len(), activity_counts.len());

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

        // let d_act = den_activities[cel_idx];


        // `da`: dendrite activity rating (pre-inhib)
        // `ca`: cell activity rating (post-inhib)
        // `ct`: cell activity count

        if (cel_idx & 7) == 0 {
        // if cel_ttl > 0. && cel_ttl < 150. {
        // if cel_ttl > 600. {
            print!("{{[");
            printc!(dark_grey: "{}", cel_idx);
            print!("]::da:");
            printc!(green: "{}", den_activities[cel_idx]);
            print!(",ca:");
            printc!(green: "{}", cel_activities[cel_idx]);
            print!(",ct:");
            printc!(royal_blue: "{}", cel_ttl);
            print!("}} ");
        }

        // if cel_ttl > _min {
        //     println!("Cell [{}]({}): {:?}", cel_idx, cel_ttl, _non_zero_ptrn_ttls);
        // }
        cel_ttls.push(cel_ttl);
        ttl_count += cel_ttl;
    }

    print!("\n");

    // Calc stdev:
    let mean = ttl_count / cel_count as f32;
    let mut sq_diff_ttl = 0f32;
    for &cel_ttl in cel_ttls.iter() {
        sq_diff_ttl += (cel_ttl - mean).powi(2);
        // print!("<{}>", (cel_ttl - mean).powi(2));
    }
    // print!("\n");

    let stdev = (sq_diff_ttl / ttl_count).sqrt();
    println!("Standard deviation: {}", stdev);
}


fn finish_queues(params: &Params, i: usize, exiting: &mut bool) {
    params.req_tx.send(Request::FinishQueues(i)).unwrap();
    params.cmd_tx.send(Command::None).unwrap();

    // Wait for completion.
    loop {
        debug!("Attempting to receive...");
        match params.res_rx.recv() {
            Ok(res) => match res {
                Response::Status(status) => {
                    debug!("Status: {:?}", status);
                    // if status.prev_cycles > cycle_count {
                    //     params.req_tx.send(Request::FinishQueues(i)).unwrap();
                    //     params.cmd_tx.send(Command::None).unwrap();
                    // }
                },
                // Response::QueuesFinished(prev_cycles) => {
                Response::QueuesFinished(qf_i) => {
                    // if prev_cycles > cycle_count {
                    //     debug!("Queues finished for: {}", prev_cycles);
                    //     cycle_count = cycle_count.wrapping_add(1);
                    //     break;=
                    // }
                    if qf_i == i {
                        debug!("Queues finished for iteration: {}", i);
                        // cycle_count = cycle_count.wrapping_add(1);
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


fn cycle(params: &Params, training_iters: usize, collect_iters: usize,
        pattern_count: usize, sdrs: &Vec<Vec<u8>>, activity_counts: &mut Vec<Vec<usize>>)
{
    let mut rng = rand::weak_rng();
    let mut exiting = false;
    // let mut cycle_count = 0u32;

    // Main loop:
    for i in 0..training_iters + collect_iters {
        // Write a random SDR.
        let pattern_idx = Range::new(0, pattern_count).ind_sample(&mut rng);
        debug!("Locking tract buffer...");
        let mut guard = params.tract_buffer.clone().write().wait().unwrap();
        debug_assert!(guard.len() == sdrs[pattern_idx].len());
        for (src, dst) in sdrs[pattern_idx].iter().zip(guard.iter_mut()) {
            *dst = *src;
        }
        WriteGuard::release(guard);

        // Cycle.
        params.cmd_tx.send(Command::Iterate(1)).unwrap();

        // Wait for completion.
        finish_queues(params, i, &mut exiting);

        if i >= training_iters {
            // Increment the cell activity counts.
            let l4_axns = params.l4_axns.map().read().enq().unwrap();
            for (counts, &axn) in activity_counts.iter_mut().zip(l4_axns.iter()) {
                counts[pattern_idx] += (axn > 0) as usize;
            }
        }

        if exiting { break; }
    }
}

static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 64;
const AREA_DIM: u32 = 16;

pub fn eval(/*params: Params*/) {
    let (command_tx, command_rx) = mpsc::channel();
    let (vibi_request_tx, vibi_request_rx) = mpsc::channel();
    let (vibi_response_tx, vibi_response_rx) = mpsc::channel();
    let vibi_command_tx = command_tx.clone();

    let (spatial_request_tx, spatial_request_rx) = mpsc::channel();
    let (spatial_response_tx, spatial_response_rx) = mpsc::channel();
    let spatial_command_tx = command_tx;

    let mut cortex = Cortex::new(define_lm_schemes(), define_a_schemes(), Some(ca_settings()));

    let v0_ext_lyr_addr = *cortex.thal().area_maps().by_key(IN_AREA).expect("bad area")
        .layer_map().layers().by_key(EXT_LYR).expect("bad lyr").layer_addr();

    let v1_spt_lyr_buf = {
        let pri_area_map = cortex.thal().area_maps().by_key(PRI_AREA).expect("bad area");
        let v1_spt_lyr_addr = *pri_area_map.layer_map().layers().by_key(SPT_LYR)
            .expect("bad lyr").layer_addr();
        let v1_spt_lyr_axn_range = pri_area_map.lyr_axn_range(&v1_spt_lyr_addr, None).unwrap();
        cortex.areas().by_key(PRI_AREA).unwrap().axns()
            .create_sub_buffer(&v1_spt_lyr_axn_range).unwrap()
    };

    // Layer 4 spatial dendrite activity ratings (pre-inhib):
    let l4_spt_den_actvs = cortex.areas().by_key(PRI_AREA).unwrap()
        .psal_TEMP().dens().activities().clone();

    // Layer 4 spatial cell activity ratings (axon activity, post-inhib):
    let l4_spt_cel_actvs = cortex.areas().by_key(PRI_AREA).unwrap()
        .psal_TEMP().activities().clone();

    let in_tract_idx = cortex.thal().tract().index_of(v0_ext_lyr_addr).unwrap();
    let in_tract_buffer = cortex.thal().tract().buffer(in_tract_idx).unwrap().clone();
    let axns = cortex.areas().by_key(PRI_AREA).unwrap().axns().states().clone();
    let area_map = cortex.areas().by_key(PRI_AREA).unwrap().area_map().clone();

    let nucl = Nucleus::new(IN_AREA, EXT_LYR, PRI_AREA, &cortex);
    cortex.add_subcortex(Subcortex::new().nucl(nucl));

    let mut flywheel = Flywheel::new(cortex, command_rx, PRI_AREA);
    flywheel.add_req_res_pair(vibi_request_rx, vibi_response_tx);
    flywheel.add_req_res_pair(spatial_request_rx, spatial_response_tx);

    // Flywheel thread:
    let th_flywheel = thread::Builder::new().name("flywheel".to_string()).spawn(move || {
        flywheel.spin();
    }).expect("Error creating 'flywheel' thread");

    // Vibi thread:
    let th_win = thread::Builder::new().name("win".to_string()).spawn(move || {
        println!("Opening vibi window...");
        window::Window::open(vibi_command_tx, vibi_request_tx, vibi_response_rx);
    }).expect("Error creating 'win' thread");

    let params = Params { cmd_tx: spatial_command_tx, req_tx: spatial_request_tx,
        res_rx: spatial_response_rx, tract_buffer: in_tract_buffer, axns,
        l4_axns: v1_spt_lyr_buf, /*l4_spt_den_acts: l4_spt_den_acts,*/
        area_map, encode_dim: ENCODE_DIM, area_dim: AREA_DIM };

    { // Inner (refactorable)
        const SPARSITY: usize = 48;
        let pattern_count = 64;
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
        params.cmd_tx.send(Command::None).unwrap();

        // Define the number of iters to first train then collect for each
        // sample period. All learning and other cell parameters (activity,
        // energy, etc.) persist between sample periods. Only collection
        // iters are recorded and evaluated.
        let training_collect_iters = vec![
            // (0, 10000),
            // (40000, 10000),
            // (40000, 10000),
            // (40000, 10000),
            // (40000, 10000),
            // (40000, 10000),
            // (80000, 10000),
            // (80000, 10000),
            // (80000, 10000),
            // (80000, 10000),
            // (80000, 10000),

            // (0, 10000),
            // (0, 10000),
            // (40000, 10000),
            // (0, 10000),

            (0, 10),
        ];

        for (t, (training_iters, collect_iters)) in training_collect_iters.into_iter().enumerate() {
            let mut activity_counts = vec![vec![0; pattern_count]; cell_count];
            cycle(&params, training_iters, collect_iters, pattern_count,
                &sdrs, &mut activity_counts);
            println!("\nActivity Counts [{}] (train: {}, collect: {}):",
                t, training_iters, collect_iters);
            print_activity_counts(&l4_spt_den_actvs, &l4_spt_cel_actvs, &activity_counts);
        }

        params.cmd_tx.send(Command::Exit).unwrap();
        params.cmd_tx.send(Command::None).unwrap();

        println!("Spatial evaluation complete.\n");
        // params.res_rx.recv().unwrap();
    }

    if let Err(e) = th_win.join() { println!("th_win.join(): Error: '{:?}'", e); }
    if let Err(e) = th_flywheel.join() { println!("th_flywheel.join(): Error: '{:?}'", e); }
}

fn define_lm_schemes() -> LayerMapSchemeList {
    let at0 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            .input_layer("aff_in", map::DEFAULT,
                AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]),
                AxonTopology::Spatial
            )
            .layer("mcols", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
                CellScheme::minicolumn("iv", "iii", 9999)
            )
            .layer(SPT_LYR, 1, map::PSAL, AxonDomain::Local,
                CellScheme::spiny_stellate(&[("aff_in", 4, 1)], 7, 600)
            )
            .layer("iv_inhib", 0, map::DEFAULT, AxonDomain::Local, CellScheme::inhib("iv", 4, 0))
            .layer("iv_smooth", 0, map::DEFAULT, AxonDomain::Local, CellScheme::smooth("iv", 4, 1))
            .layer("iii", 1, map::PTAL, AxonDomain::Local,
                CellScheme::pyramidal(&[("iii", 20, 1)], 1, 6, 500)
                    // .apical(&[("eff_in", 22)], 1, 5, 500)
            )
        )
        .lmap(LayerMapScheme::new("v0_lm", LayerMapKind::Subcortical)
            .layer(EXT_LYR, 1, map::DEFAULT,
                AxonDomain::output(&[map::THAL_SP, at0]),
                LayerKind::Axonal(AxonTopology::Spatial))
        )
}


fn define_a_schemes() -> AreaSchemeList {
    AreaSchemeList::new()
        .area(AreaScheme::new("v0", "v0_lm", ENCODE_DIM)
            // .input(InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 10,
            //    scale: 1.4, hrz_dims: (16, 16) }),
            // .input(InputScheme::ScalarSdrGradiant { range: (-8.0, 8.0), way_span: 16.0, incr: 0.1 }),
            // .input(InputScheme::None),
            // .input(InputScheme::Custom { layer_count: 1 }),
            // .custom_layer_count(1)
            .subcortex()
        )
        .area(AreaScheme::new(PRI_AREA, "visual", AREA_DIM)
            .eff_areas(vec!["v0"])
            // .filter_chain(map::FF_IN, vec![FilterScheme::new("retina", None)])
        )
}

// #########################
// ##### DISABLE STUFF #####
// #########################
#[allow(unused_mut)]
pub fn ca_settings() -> CorticalAreaSettings {
    let mut settings = CorticalAreaSettings::new();

    // settings.bypass_inhib = true;
    settings.bypass_filters = true;
    settings.disable_pyrs = true;
    // settings.disable_ssts = true;
    settings.disable_mcols = true;
    // settings.disable_regrowth = true;
    // settings.disable_learning = true;

    settings
}
