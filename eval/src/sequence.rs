//! Determine how well a layer of pyramidal cells can predict the next input
//! in a learned sequence of inputs.
//!
//!
//!

#![allow(dead_code, unused_imports, unused_variables)]

use std::mem;
use std::collections::{HashMap, BTreeMap};
use std::ops::Range;
use smallvec::SmallVec;
use rand::{self, XorShiftRng};
use rand::distributions::{Range as RandRange, IndependentSample};
use qutex::{QrwLock, ReadGuard as QrwReadGuard};
use vibi::bismit::futures::{future, Future, Poll, Async};
use vibi::bismit::ocl::{FutureReadGuard, ReadGuard};
use vibi::bismit::{map, Result as CmnResult, Error as CmnError, Cortex, CorticalAreaSettings,
    Thalamus, SubcorticalNucleus, SubcorticalNucleusLayer, WorkPool, CorticalAreas, TractReceiver,
    SamplerKind, SamplerBufferKind, ReadBuffer, FutureRecv, /*FutureReadGuardVec, ReadGuardVec,*/
    CorticalSampler, FutureCorticalSamples, CorticalSamples, CellSampleIdxs, CorticalAreaTest,
    DendritesTest, SynapsesTest, CelCoords, DenCoords, SynCoords};
use vibi::bismit::map::*;
use vibi::bismit::cmn::{TractFrameMut, TractDims};
use vibi::bismit::encode::{self, Vector2dWriter};
use ::{IncrResult, TrialIter, Layer, Pathway, InputSource, Sdrs, SeqCursor, SeqCursorPos};
use ::spatial::{TrialData, TrialResults};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";

const ENCODE_DIMS_0: (u32, u32, u8) = (48, 48, 1);
// const ENCODE_DIMS_1: (u32, u32, u8) = (30, 255, 1);
const AREA_DIM: u32 = 48;
const SEQUENTIAL_SDR: bool = true;

const PRINT_INTERVAL: usize = 5000;
const PRINT_INTERVAL_START: usize = 0;
const PRINT_INTERVAL_END: usize = 5;

const BASAL_DISTAL_TUFT_ID: usize = 1;


/// Prints stuff.
fn print_stuff(samples: CorticalSamples, focus_cels: Vec<FocusCell>,
        cycles_complete: usize, lyr_addr: LayerAddress,
        cursor_pos: SeqCursorPos) {
    //////// Only print frames we are interested in:
    // if cycles_complete % 5000 >= 5 { return samples; }
    // if seq_idx != 0 { return samples; }
    // if cursor_pos.seq_idx != 0 { return samples; }

    let tft_states = samples.sample(&SamplerKind::TuftStates(lyr_addr)).unwrap().u8();
    let tft_best_den_ids = samples.sample(&SamplerKind::TuftBestDenIds(lyr_addr)).unwrap().u8();
    let tft_best_den_states = samples.sample(&SamplerKind::TuftBestDenStates(lyr_addr)).unwrap().u8();
    let tft_best_den_states_raw = samples.sample(&SamplerKind::TuftBestDenStatesRaw(lyr_addr)).unwrap().u8();
    let den_states = samples.sample(&SamplerKind::DenStates(lyr_addr)).unwrap().u8();
    let syn_states = samples.sample(&SamplerKind::SynStates(lyr_addr)).unwrap().u8();
    let syn_strengths = samples.sample(&SamplerKind::SynStrengths(lyr_addr)).unwrap().i8();
    // let syn_src_col_v_offs = samples.sample(&SamplerKind::SynSrcColVOffs(lyr_addr)).unwrap().i8();
    // let syn_src_col_u_offs = samples.sample(&SamplerKind::SynSrcColUOffs(lyr_addr)).unwrap().i8();
    let syn_flag_sets = samples.sample(&SamplerKind::SynFlagSets(lyr_addr)).unwrap().u8();

    for cel in &focus_cels {
        let cel_idx = cel.cel_coords.idx() as usize;
        let cel_axn_idx = cel.cel_coords.axon_idx() as usize;
        let celtft = &cel.tufts[0];
        let celtft_idx = celtft.celtft_idx;
        let den_idx_range = &celtft.den_idx_range;
        let syn_idx_range = &celtft.syn_idx_range;

        let axn_states = samples.sample(&SamplerKind::Axons(None)).unwrap().u8();
        let cel_states = samples.sample(&SamplerKind::SomaStates(lyr_addr)).unwrap().u8();

        println!();
        print!("[{}] (cursor_pos: {:?}) ", cycles_complete, cursor_pos);
        println!("cel_states[{}]: <<{:03?}>>, axn_states[{}]: <<<{:03?}>>>",
            cel_idx, cel_states[cel_idx], cel_axn_idx, axn_states[cel_axn_idx]);

        println!("tft_states[{}]: {:03?}", celtft_idx,
            &tft_states[celtft_idx]);
        println!("tft_best_den_ids[{}]: {:03?}", celtft_idx,
            &tft_best_den_ids[celtft_idx]);
        println!("tft_best_den_states[{}]: {:03?}", celtft_idx,
            &tft_best_den_states[celtft_idx]);
        println!("tft_best_den_states_raw[{}]: {:03?}", celtft_idx,
            &tft_best_den_states_raw[celtft_idx]);
        println!("den_states[{:?}]: {:03?}", den_idx_range,
            &den_states[den_idx_range.clone()]);

        let mut strong_syn_count = 0;
        for (syn_idx, ((&syn_state, &syn_strength), &syn_flag_set)) in syn_states[syn_idx_range.clone()].iter()
                .zip(syn_strengths[syn_idx_range.clone()].iter())
                .zip(syn_flag_sets[syn_idx_range.clone()].iter())
                .enumerate()
                .filter(|&(_, ((_e, &syn_strength), _))| {
                    syn_strength > 21
                }) {
            print!("{{[{}]", syn_idx);
            print!("state:{:03}, ", syn_state);
            print!("strength:{:03}, ", syn_strength);
            print!("flag_set:{:03}", syn_flag_set);
            print!("}} ");
            strong_syn_count += 1;
        }
        if strong_syn_count > 0 {
            println!();
            print!("  {{{{ strong_syn_count: {} }}}}", strong_syn_count);
        }
        println!();
    }
    println!();

}


/// Checks stuff.
fn check_stuff(samples: CorticalSamples, focus_cels: Vec<FocusCell>,
        cycles_complete: usize, lyr_addr: LayerAddress, sdrs: QrwReadGuard<Vec<Vec<u8>>>,
        cursor_pos: SeqCursorPos, next_cursor_pos: SeqCursorPos,
        focus_layer_axon_range: (usize, usize)) {
    if cursor_pos.seq_idx != 0 { return; }

    let axn_states = samples.sample(&SamplerKind::Axons(None)).unwrap().u8();
    let tft_states = samples.sample(&SamplerKind::TuftStates(lyr_addr)).unwrap().u8();
    let tft_best_den_ids = samples.sample(&SamplerKind::TuftBestDenIds(lyr_addr)).unwrap().u8();
    let tft_best_den_states = samples.sample(&SamplerKind::TuftBestDenStates(lyr_addr)).unwrap().u8();
    let tft_best_den_states_raw = samples.sample(&SamplerKind::TuftBestDenStatesRaw(lyr_addr)).unwrap().u8();
    let den_states = samples.sample(&SamplerKind::DenStates(lyr_addr)).unwrap().u8();
    let syn_states = samples.sample(&SamplerKind::SynStates(lyr_addr)).unwrap().u8();
    let syn_strengths = samples.sample(&SamplerKind::SynStrengths(lyr_addr)).unwrap().i8();
    // let syn_src_col_v_offs = samples.sample(&SamplerKind::SynSrcColVOffs(lyr_addr)).unwrap().i8();
    // let syn_src_col_u_offs = samples.sample(&SamplerKind::SynSrcColUOffs(lyr_addr)).unwrap().i8();
    let syn_flag_sets = samples.sample(&SamplerKind::SynFlagSets(lyr_addr)).unwrap().u8();

    let cur_sdr = &sdrs[cursor_pos.pattern_idx];
    let next_sdr = &sdrs[next_cursor_pos.pattern_idx];

    // Check to make sure the SDR inputs match with the axon states:
    //
    // TODO: Run this check (SDR-axon states) in its own function.
    for (sdr_idx, axn_idx) in (focus_layer_axon_range.0..focus_layer_axon_range.1).enumerate() {
        // println!("cur_sdr[sdr_idx]: {}, axn_states[axn_idx]: {}", cur_sdr[sdr_idx], axn_states[axn_idx]);
        assert!(cur_sdr[sdr_idx] != 0 && axn_states[axn_idx] != 0 ||
            cur_sdr[sdr_idx] == 0 && axn_states[axn_idx] == 0);
    }


    // TODO: Check `next_sdr` against tuft states (print accuracy %).

    // BASAL_DISTAL_TUFT_ID


    for cel in &focus_cels {
        let cel_idx = cel.cel_coords.idx() as usize;
        let cel_axn_idx = cel.cel_coords.axon_idx() as usize;
        let celtft = &cel.tufts[0];
        let celtft_idx = celtft.celtft_idx;
        let den_idx_range = &celtft.den_idx_range;
        let syn_idx_range = &celtft.syn_idx_range;

        let axn_states = samples.sample(&SamplerKind::Axons(None)).unwrap().u8();
        let cel_states = samples.sample(&SamplerKind::SomaStates(lyr_addr)).unwrap().u8();

        println!();
        print!("[{}] (cursor_pos: {:?}) ", cycles_complete, cursor_pos);
        println!("cel_states[{}]: <<{:03?}>>, axn_states[{}]: <<<{:03?}>>>",
            cel_idx, cel_states[cel_idx], cel_axn_idx, axn_states[cel_axn_idx]);

        println!("tft_states[{}]: {:03?}", celtft_idx,
            &tft_states[celtft_idx]);
        println!("tft_best_den_ids[{}]: {:03?}", celtft_idx,
            &tft_best_den_ids[celtft_idx]);
        println!("tft_best_den_states[{}]: {:03?}", celtft_idx,
            &tft_best_den_states[celtft_idx]);
        println!("tft_best_den_states_raw[{}]: {:03?}", celtft_idx,
            &tft_best_den_states_raw[celtft_idx]);
        println!("den_states[{:?}]: {:03?}", den_idx_range,
            &den_states[den_idx_range.clone()]);

        let mut strong_syn_count = 0;
        for (syn_idx, ((&syn_state, &syn_strength), &syn_flag_set)) in syn_states[syn_idx_range.clone()].iter()
                .zip(syn_strengths[syn_idx_range.clone()].iter())
                .zip(syn_flag_sets[syn_idx_range.clone()].iter())
                .enumerate()
                .filter(|&(_, ((_e, &syn_strength), _))| {
                    syn_strength > 21
                }) {
            print!("{{[{}]", syn_idx);
            print!("state:{:03}, ", syn_state);
            print!("strength:{:03}, ", syn_strength);
            print!("flag_set:{:03}", syn_flag_set);
            print!("}} ");
            strong_syn_count += 1;
        }
        if strong_syn_count > 0 {
            println!();
            print!("  {{{{ strong_syn_count: {} }}}}", strong_syn_count);
        }
        println!();
    }
    println!();

}



#[derive(Clone, Debug)]
struct FocusCell {
    cel_coords: CelCoords,
    tufts: SmallVec<[FocusCellTuft; 8]>,
}

impl FocusCell {
    fn new(area_name: &'static str, layer_name: &'static str,
            cel_coords: CelCoords, cortical_areas: &mut CorticalAreas)
            -> FocusCell {
        // let area = cortical_areas.by_key_mut(area_name).unwrap();
        // let den_idx_range = area.layer_test_mut(layer_name).unwrap().dens()
        //     .den_idx_range_celtft(&cel_coords, tft_id);
        // let syn_idx_range = area.layer_test_mut(layer_name).unwrap().dens().syns()
        //     .syn_idx_range_celtft(&cel_coords, tft_id);
        // let celtft_idx = area.layer_test_mut(layer_name).unwrap().tufts()
        //     .celtft_idx(&cel_coords, tft_id);

        let tft_count = cortical_areas.by_key_mut(area_name).unwrap()
            .layer_test_mut(layer_name).unwrap().tufts().count();

        let tufts = (0..tft_count).map(|tft_id| {
            FocusCellTuft::new(area_name, layer_name, &cel_coords, tft_id, cortical_areas)
        }).collect::<SmallVec<_>>();

        FocusCell { cel_coords, tufts }
    }

    fn random(area_name: &'static str, layer_name: &'static str, tft_id: usize,
            cortical_areas: &mut CorticalAreas) -> FocusCell {
        let cel_coords = cortical_areas.by_key_mut(area_name).unwrap()
            .layer_test_mut(layer_name).unwrap().rand_cel_coords();
        FocusCell::new(area_name, layer_name, cel_coords, cortical_areas)
    }
}


#[derive(Clone, Debug)]
struct FocusCellTuft {
    // cel_coords: CelCoords,
    tft_id: usize,
    celtft_idx: usize,
    den_idx_range: Range<usize>,
    syn_idx_range: Range<usize>,
}

impl FocusCellTuft {
    fn new(area_name: &'static str, layer_name: &'static str,
            cel_coords: &CelCoords, tft_id: usize, cortical_areas: &mut CorticalAreas)
            -> FocusCellTuft {
        let area = cortical_areas.by_key_mut(area_name).unwrap();
        let den_idx_range = area.layer_test_mut(layer_name).unwrap().dens()
            .den_idx_range_celtft(&cel_coords, tft_id);
        let syn_idx_range = area.layer_test_mut(layer_name).unwrap().dens().syns()
            .syn_idx_range_celtft(&cel_coords, tft_id);
        let celtft_idx = area.layer_test_mut(layer_name).unwrap().tufts()
            .celtft_idx(&cel_coords, tft_id);

        FocusCellTuft { /*cel_coords,*/ tft_id, celtft_idx, den_idx_range, syn_idx_range }
    }

    fn random(area_name: &'static str, layer_name: &'static str, tft_id: usize,
            cortical_areas: &mut CorticalAreas) -> FocusCellTuft {
        let cel_coords = cortical_areas.by_key_mut(area_name).unwrap()
            .layer_test_mut(layer_name).unwrap().rand_cel_coords();
        FocusCellTuft::new(area_name, layer_name, &cel_coords, tft_id, cortical_areas)
    }
}


/// Tracks the activity of a set of cells over time, storing usefully relevant
/// details about synaptic activity, etc.
pub struct CellTracker {
    focus_cels: Vec<FocusCell>,

}


/// A `SubcorticalNucleus`.
struct EvalSequence {
    area_name: String,
    area_id: usize,
    layers: HashMap<LayerAddress, Layer>,
    cycles_complete: usize,
    sdrs: Sdrs,
    sdr_cursor: SeqCursor,
    trial_iter: TrialIter,
    sampler: Option<CorticalSampler>,
    pri_iii_layer_addr: Option<LayerAddress>,
    focus_layer_axon_range: Option<(usize, usize)>,
    // input_layer_axon_range: Option<(usize, usize)>,
    focus_cels: Vec<FocusCell>,
    // last_pattern: SeqCursorPos,
}

impl EvalSequence {
    pub fn new<S: Into<String>>(layer_map_schemes: &LayerMapSchemeList,
            area_schemes: &AreaSchemeList, area_name: S)
            -> EvalSequence {
        let area_name = area_name.into();
        let area_scheme = &area_schemes[&area_name];
        let layer_map_scheme = &layer_map_schemes[area_scheme.layer_map_name()];
        let mut layers = HashMap::with_capacity(4);

        for layer_scheme in layer_map_scheme.layers() {
            let lyr_dims = match layer_scheme.name() {
                "external_0" => None,
                // "external_1" => Some(ENCODE_DIMS_1.into()),
                ln @ _ => panic!("EvalSequence::new: Unknown layer name: {}.", ln),
            };

            let sub_layer = SubcorticalNucleusLayer::from_schemes(layer_scheme, area_scheme,
                lyr_dims);

            let layer = Layer {
                sub: sub_layer,
                pathway: Pathway::None,
            };

            layers.insert(layer.sub().addr().clone(), layer);
        }

        assert!(ENCODE_DIMS_0.0 == AREA_DIM && ENCODE_DIMS_0.1 == AREA_DIM,
            "For this evaluation, the encoding dims must equal the area dims. \
            The encoding is representative of layer IV output.");

        let sdrs = Sdrs::new(200, ENCODE_DIMS_0);
        // let sdr_cursor = SeqCursor::new((4, 8), 25, sdrs.len());
        let sdr_cursor = SeqCursor::new((5, 5), 1, sdrs.len());

        // Define the number of iters to first train then collect for each
        // sample period. All learning and other cell parameters (activity,
        // energy, etc.) persist between sample periods. Only collection
        // iters are recorded and evaluated.
        let trial_iter = TrialIter::new(vec![
            (5000, 5000), (5000, 5000), (5000, 5000), (5000, 5000), (5000, 5000),
        ]);

        // let last_pattern = sdr_cursor.pos();

        EvalSequence {
            area_name: area_name,
            area_id: area_scheme.area_id(),
            layers,
            cycles_complete: 0,
            sdrs,
            sdr_cursor,
            trial_iter,
            sampler: None,
            pri_iii_layer_addr: None,
            focus_layer_axon_range: None,
            // input_layer_axon_range: None,
            focus_cels: Vec::with_capacity(16),
            // last_pattern,
        }
    }

    /*
        Plan:
        - Load up a length 4:25 glyph sequence.
        - 8 slices of pyrs.
        - Cycle
        - Check to see who's winning
            - Print the tft and cel state/best values
    */

}

impl SubcorticalNucleus for EvalSequence {
    fn create_pathways(&mut self, thal: &mut Thalamus, cortical_areas: &mut CorticalAreas)
            -> CmnResult<()> {
        // Wire up I/O pathways:
        for layer in self.layers.values_mut() {
            layer.pathway = Pathway::new(thal, layer.sub());
        }

        let lyr_addr = thal.area_maps().by_key(PRI_AREA).expect("invalid area name")
            .layer_map().layers().by_key("iii").expect("invalid lyr name")
            .layer_addr();

        let sampler_kinds = vec![
            SamplerKind::Axons(None),
            SamplerKind::SomaStates(lyr_addr),
            // SamplerKind::SomaEnergies(lyr_addr),
            // SamplerKind::SomaActivities(lyr_addr),
            // SamplerKind::SomaFlagSets(lyr_addr),
            SamplerKind::TuftStates(lyr_addr),
            SamplerKind::TuftBestDenIds(lyr_addr),
            SamplerKind::TuftBestDenStatesRaw(lyr_addr),
            SamplerKind::TuftBestDenStates(lyr_addr),
            // SamplerKind::TuftPrevStates(lyr_addr),
            // SamplerKind::TuftPrevBestDenIds(lyr_addr),
            // SamplerKind::TuftPrevBestDenStatesRaw(lyr_addr),
            // SamplerKind::TuftPrevBestDenStates(lyr_addr),
            SamplerKind::DenStates(lyr_addr),
            // SamplerKind::DenStatesRaw(lyr_addr),
            // SamplerKind::DenEnergies(lyr_addr),
            // SamplerKind::DenActivities(lyr_addr),
            // SamplerKind::DenThresholds(lyr_addr),
            SamplerKind::SynStates(lyr_addr),
            SamplerKind::SynStrengths(lyr_addr),
            // SamplerKind::SynSrcColVOffs(lyr_addr),
            // SamplerKind::SynSrcColUOffs(lyr_addr),
            SamplerKind::SynFlagSets(lyr_addr),
        ];

        // TODO: Determine the tuft id of the basal distal tuft instead:
        assert!(cortical_areas.by_key_mut(PRI_AREA).unwrap().layer_test_mut("iii").unwrap()
            .cell_scheme().tft_schemes()[BASAL_DISTAL_TUFT_ID].den_class() == DendriteClass::Basal);
        assert!(cortical_areas.by_key_mut(PRI_AREA).unwrap().layer_test_mut("iii").unwrap()
            .cell_scheme().tft_schemes()[BASAL_DISTAL_TUFT_ID].den_kind() == DendriteKind::Distal);

        self.sampler = Some(CorticalSampler::new(PRI_AREA, sampler_kinds, CellSampleIdxs::All,
            thal, cortical_areas));

        self.pri_iii_layer_addr = Some(lyr_addr);

        let layer_dims = cortical_areas.by_key_mut(PRI_AREA).unwrap()
            .layer_test_mut("iii").unwrap().dims().clone();

        let lyr_axn_slc_idz = cortical_areas.by_key_mut(PRI_AREA).unwrap()
            .layer_test_mut("iii").unwrap().base_axon_slc();

        // Choose a focus cell-tuft which will be active:
        let a_valid_sdr_idx = self.sdr_cursor.a_middle_src_idx();
        let an_active_cell = self.sdrs.a_middle_active_cell(a_valid_sdr_idx);
        println!("###### an_active_cell: {:?}", an_active_cell);

        for slc_id_lyr in 0..layer_dims.depth() {
            let mut ccs = CelCoords::new(lyr_axn_slc_idz + slc_id_lyr, slc_id_lyr,
                an_active_cell.0, an_active_cell.1, layer_dims.clone());
            ccs.set_axon_idx(thal.area_maps().by_key(PRI_AREA).unwrap());
            self.focus_cels.push(FocusCell::new(PRI_AREA, "iii", ccs, cortical_areas));
        }

        self.focus_layer_axon_range = Some(cortical_areas.by_key_mut(PRI_AREA).unwrap()
            .layer_test_mut("iii").unwrap().axon_range());
        // self.input_layer_axon_range = Some(cortical_areas.by_key_mut(PRI_AREA).unwrap()
        //     .layer_test_mut("iii").unwrap().axon_range());

        Ok(())
    }

    /// Pre-cycle:
    ///
    /// * Writes output SDR to thalamic tract
    /// *
    ///
    fn pre_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            work_pool: &mut WorkPool) -> CmnResult<()> {
        let pattern_idx = self.sdr_cursor.incr_src_idx();

        // // Turn off backpressure for frames we are not interested in:
        // if self.cycles_complete % PRINT_INTERVAL == PRINT_INTERVAL_START {
        //     self.sampler.as_ref().unwrap().set_backpressure(true);
        // }
        // if self.cycles_complete % PRINT_INTERVAL == PRINT_INTERVAL_END {
        //     self.sampler.as_ref().unwrap().set_backpressure(false);
        // }

        // Write sdr to pathway:
        for layer in self.layers.values() {
            if let Pathway::Output { ref tx } = layer.pathway {
                debug_assert!(layer.sub().axon_domain().is_output());

                match layer.sub().name() {
                    "external_0" => {
                        let future_sdrs = self.sdrs.lock.clone().read().from_err();

                        let future_write_guard = tx.send()
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
                    },
                    _ => (),
                }
            }
        }

        self.cycles_complete += 1;
        Ok(())
    }

    /// Post-cycle:
    ///
    /// * Blocks to wait for sampler channels
    /// * Increments the cell activity counts
    ///
    fn post_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            work_pool: &mut WorkPool) -> CmnResult<()> {
        for layer in self.layers.values() {
            if let Pathway::Input { srcs: _ } = layer.pathway {
                debug_assert!(layer.sub().axon_domain().is_input());
            }
        }

        match self.trial_iter.incr() {
            IncrResult::TrialComplete { scheme_idx: _, train: _, collect: _ } => {},
            _ir @ _ => {},
        }

        ///////////////////////////////////////////////////////////////////////

        let focus_cels = self.focus_cels.clone();
        let cycles_complete = self.cycles_complete;
        let lyr_addr = self.pri_iii_layer_addr.clone().unwrap();
        let cursor_pos = self.sdr_cursor.cur_pos();
        let next_cursor_pos = self.sdr_cursor.cur_pos();
        let focus_layer_axon_range = self.focus_layer_axon_range.unwrap();
        // let input_layer_axon_range = self.input_layer_axon_range.unwrap();

        let future_recv = self.sampler.as_ref().unwrap().recv()
            .join(self.sdrs.lock.clone().read().from_err())
            .map(move |(samples, sdrs)| {
                // print_stuff(samples, focus_cels, cycles_complete,
                //     lyr_addr, cursor_pos);
                check_stuff(samples, focus_cels, cycles_complete,
                    lyr_addr, sdrs, cursor_pos, next_cursor_pos,
                    focus_layer_axon_range, /*input_layer_axon_range*/);
            })
            .map_err(|err| panic!("{}", err));

        work_pool.complete_work(future_recv)?;


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


fn define_lm_schemes() -> LayerMapSchemeList {
    let at_el0 = AxonTag::unique();
    let at_el1 = AxonTag::unique();
    let at1 = AxonTag::unique();
    let at2 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            .layer(LayerScheme::define("aff_in_0")
                .axonal(AxonTopology::Spatial)
                .axon_domain(AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at_el0])]))
            )
            .layer(LayerScheme::define("aff_in_1")
                .axonal(AxonTopology::Nonspatial)
                .axon_domain(AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at_el1])]))
            )
            // .layer(LayerScheme::define("iv")
            //     .depth(1)
            //     .tags(LayerTags::PSAL)
            //     .axon_domain(AxonDomain::output(&[at1]))
            //     .cellular(CellScheme::spiny_stellate()
            //         .tft(TuftScheme::basal().proximal()
            //             .syns_per_den_l2(5)
            //             .src_lyr(TuftSourceLayer::define("aff_in_0")
            //                 .syn_reach(7)
            //                 .prevalence(1)
            //             )
            //         )
            //     )
            // )
            // .layer(LayerScheme::define("iv_inhib")
            //     .cellular(CellScheme::control(
            //             ControlCellKind::InhibitoryBasketSurround {
            //                 host_lyr_name: "iv".into(),
            //                 field_radius: 4,
            //             },
            //             0
            //         )
            //     )
            // )
            // .layer(LayerScheme::define("iv_smooth")
            //     .cellular(CellScheme::control(
            //             ControlCellKind::ActivitySmoother {
            //                 host_lyr_name: "iv".into(),
            //                 field_radius: 4,
            //             },
            //             1
            //         )
            //     )
            // )
            .layer(LayerScheme::define("iii")
                // .depth(9)
                .depth(1)
                .tags(LayerTags::PTAL)
                .axon_domain(AxonDomain::output(&[at2]))
                .cellular(CellScheme::pyramidal()
                    .tft(TuftScheme::basal().proximal()
                        .syns_per_den(1)
                        // .src_lyr(TuftSourceLayer::define("iv")
                        .src_lyr(TuftSourceLayer::define("aff_in_0")
                            .syn_reach(0)
                            .prevalence(1)
                        )
                    )
                    .tft(TuftScheme::basal().distal()
                        // .dens_per_tft(16)
                        .dens_per_tft(4)
                        .syns_per_den(64)
                        .max_active_dens_l2(0)
                        .thresh_init(0)
                        .src_lyr(TuftSourceLayer::define("iii")
                            .syn_reach(32)
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
            .layer(LayerScheme::define("external_0")
                .depth(1)
                .axonal(AxonTopology::Spatial)
                .axon_domain(AxonDomain::output(&[map::THAL_SP, at_el0]))
            )
            // .layer(LayerScheme::define("external_1")
            //     .depth(1)
            //     .axonal(AxonTopology::Nonspatial)
            //     .axon_domain(AxonDomain::output(&[map::THAL_SP, at_el1]))
            // )
        )
}

fn define_a_schemes() -> AreaSchemeList {
    AreaSchemeList::new()
        .area(AreaScheme::new(IN_AREA, "v0_lm", ENCODE_DIMS_0.0)
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


pub fn eval() {
    let layer_map_schemes = define_lm_schemes();
    let area_schemes = define_a_schemes();

    let eval_nucl = EvalSequence::new(&layer_map_schemes,
        &area_schemes, IN_AREA);

    let cortex_builder = Cortex::builder(layer_map_schemes, area_schemes)
        .ca_settings(ca_settings())
        .subcortical_nucleus(eval_nucl);

    let cortex = cortex_builder.build().unwrap();

    let controls = ::spawn_threads(cortex, PRI_AREA);

    ::join_threads(controls)
}