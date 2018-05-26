//! Determine how well a layer of pyramidal cells can predict the next input
//! in a learned sequence of inputs.
//!
//!
//!

#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::BTreeSet;
use std::sync::Arc;
use std::mem;
use std::collections::{HashMap, BTreeMap};
use std::ops::Range;
use smallvec::SmallVec;
use rand::{self, FromEntropy, rngs::SmallRng};
use rand::distributions::{Range as RandRange, Distribution};
// use ansi_term::Colour::{Blue, Red, Cyan, Green};
use qutex::{Qutex, Guard, QrwLock, ReadGuard as QrwReadGuard};
use vibi::bismit::futures::{future, Future, FutureExt, Poll, Async};
use vibi::bismit::ocl::{FutureReadGuard, ReadGuard};
use vibi::bismit::{map, Result as CmnResult, Error as CmnError, Cortex, CorticalAreaSettings,
    Thalamus, SubcorticalNucleus, SubcorticalNucleusLayer, CompletionPool, CorticalAreas, TractReceiver,
    SamplerKind, SamplerBufferKind, ReadBuffer, FutureRecv, /*FutureReadGuardVec, ReadGuardVec,*/
    CorticalSampler, FutureCorticalSamples, CorticalSamples, CellSampleIdxs,
    CorticalLayerSampler, CorticalLayerSamples,
    CorticalAreaTest, SlcId,
    DendritesTest, SynapsesTest, CelCoords, DenCoords, SynCoords, flywheel::Command};
use vibi::bismit::map::*;
use vibi::bismit::cmn::{TractFrameMut, TractDims, CorticalDims};
use vibi::bismit::encode::{self, Vector2dWriter};
use ::{IncrResult, TrialIter, Layer, Pathway, InputSource, Sdrs, SeqCursor, SeqCursorPos};
use ::spatial::{TrialData, /*TrialResults*/};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";

const ENCODE_DIMS_0: CorticalDims = CorticalDims::new(1, 48, 48);
const AREA_DIM: u32 = 48;
const SEQUENTIAL_SDR: bool = true;


#[derive(Debug)]
enum Phase {
    Init,
    Run,
    Compare,
    Void,
}


/// A trial result.
#[derive(Clone, Debug)]
struct TrialResult {

}


/// The evaluation trials.
#[derive(Debug)]
struct Trials {
    /// A list of result for each item index in an SDR sequence.
    seq_item_results: Vec<Vec<TrialResult>>,
    sdrs: Arc<Sdrs>,
    phase: Phase,
    focus: (SeqCursorPos, SeqCursorPos),
    // focus.1 cell coords:
    poss_pred_cells: Vec<(SlcId, u32, u32)>,
    // focus.1 (idx in `poss_pred_cells`, den_id, syn_id, init_strength):
    ppc_syns: Vec<(usize, u32, u32, i8)>,
    ppc_syn_strengths: Vec<i8>,
}

impl Trials {
    pub fn new(max_seq_len: usize, sdrs: Arc<Sdrs>) -> Trials {
        // let poss_pred_cells = Vec::with_capacity(sdrs.active_cell_count);

        Trials {
            seq_item_results: vec![Vec::with_capacity(10000); max_seq_len],
            sdrs,
            phase: Phase::Init,
            focus: (SeqCursorPos::default(), SeqCursorPos::default()),
            poss_pred_cells: Vec::new(),
            ppc_syns: Vec::new(),
            ppc_syn_strengths: Vec::new(),
        }
    }

    /// Initialize stuff.
    fn init(&mut self, samples: &CorticalLayerSamples) {
        // Create sets for lookup efficiency (could be hash or btree):
        let focus_0_set: BTreeSet<u32> = self.sdrs.indices[self.focus.0.pattern_idx].iter()
            .map(|&idx| idx).collect();
        let focus_1_set: BTreeSet<u32> = self.sdrs.indices[self.focus.1.pattern_idx].iter()
            .map(|&idx| idx).collect();

        // Allocate:
        self.poss_pred_cells = Vec::with_capacity(samples.map().dims().depth() as usize *
            self.sdrs.active_cell_count);
        self.ppc_syns = Vec::with_capacity(samples.map().dims().depth() as usize *
            self.sdrs.active_cell_count * 4);
        self.ppc_syns = Vec::with_capacity(self.ppc_syns.capacity());

        let col_count = samples.map().dims().columns();

        println!("Possible predictive cells:");

        // Determine the set of cells which could eventually learn to become
        // 'predictive' based on step[0] input. Only a fraction (1/depth) of
        // the cells *should* become predictors over time.
        for cell in samples.cells(.., .., ..).filter(|c| focus_1_set.contains(&c.map().col_id())) {
            let pp_cel_idx = self.poss_pred_cells.len();
            self.poss_pred_cells.push((cell.map().slc_id_lyr(), cell.map().v_id(),
                cell.map().u_id()));

            let mut pred_syn_count = 0;

            // Determine which distal synapses are targeted by step[1]
            // input cells.
            for den in cell.tuft_distal().unwrap().dendrites(..) {
                // Filter synapses with source axons belonging to cells within
                // one of the step[0] columns. Those synapses will have just
                // been active when step[1] cells become active.
                for syn in den.synapses(..).filter(|s| {
                        match s.src_axon_idx() {
                            Ok(idx) => {
                                let col_id = s.src_axon_idx().unwrap() % col_count;
                                focus_0_set.contains(&col_id)
                            },
                            // Out of range axon:
                            Err(_) => false,
                        } }) {
                    let syn_info = (pp_cel_idx, den.map().den_id(), syn.map().syn_id(), syn.strength());
                    self.ppc_syns.push(syn_info);
                    pred_syn_count += 1;

                    let dst_syn_str = samples.cell(cell.map().slc_id_lyr(),
                        cell.map().v_id(), cell.map().u_id()).tuft_distal().unwrap()
                        .dendrite(den.map().den_id()).synapse(syn.map().syn_id()).strength();
                    // print!("[{}]", dst_syn_str);
                    self.ppc_syn_strengths.push(dst_syn_str);
                }
            }

            ////// KEEPME (CELL INFO):
            // print!("[s:{}, v:{}, u:{} | col:{}, syns: {}]", cell.map().slc_id_lyr(),
            //     cell.map().v_id(), cell.map().u_id(), cell.map().col_id(), pred_syn_count);

            // TODO: Check the strength of the PROXIMAL focus_1 synapses (only
            // one per cell) and possibly set it > 0 for the purposes of this
            // evaluation (as a shortcut). Also make sure that it increases
            // naturally (below). UPDATE: strengths start at zero -- probably
            // don't worry about it for now.
        }

        println!("\nPredictor cell count: {} / {}", self.poss_pred_cells.len(),
            self.poss_pred_cells.capacity());
        println!("\nPredictor cell synapse count: {} / {}", self.ppc_syns.len(),
            self.ppc_syns.capacity());
    }

    /// Compares stuff.
    fn compare(&mut self, samples: &CorticalLayerSamples) {
        // Check the strength of the PROXIMAL focus_1 synapses (only one per
        // cell) to make sure that they have become solid.
        println!("Proximal synapse strengths:");
        for (&(ppc_idx, den_id, syn_id, str_init), &orig_syn_str) in self.ppc_syns.iter()
                .zip(self.ppc_syn_strengths.iter()) {
            let (slc, v, u) = self.poss_pred_cells[ppc_idx];
            // let prx_syn_str = samples.cell(slc, v, u).tuft_proximal().unwrap()
            //     .dendrite(0).synapse(0).strength();
            // print!("[{}]", prx_syn_str);
            // assert!(prx_syn_str >= 0);

            // TODO: Verify that the src_axn_idx has not changed.

            let new_syn_str = samples.cell(slc, v, u).tuft_distal().unwrap()
                .dendrite(den_id).synapse(syn_id).strength();
            // if new_syn_str != orig_syn_str {
            //     print!("[{}->{}]", orig_syn_str, Cyan.bold().paint(new_syn_str.to_string()));
            // }

            if new_syn_str != orig_syn_str {
                printc!(default: "[{}->", orig_syn_str);
                printc!(cyan_bold: "{}", new_syn_str);
                printc!(default: "]");
            }

            // println!("This is in red: {}", Red.paint("a red string"));
        }

        // TODO: Check that `ppc_syns` have increased in strength.
    }

    /// Checks stuff.
    fn cycle(&mut self, cycle_counter: usize, samples: &CorticalLayerSamples,
            cursor_pos: &SeqCursorPos, cursor_pos_next: &SeqCursorPos) {
        if cycle_counter % 200 == 0 { println!("{} cycles enqueued.", cycle_counter); }

        match cycle_counter {
            0 => self.phase = Phase::Init,
            1 => self.phase = Phase::Run,
            200 | 400 | 600 | 800 | 1000 => self.phase = Phase::Compare,
            _ => self.phase = Phase::Void,
        }

        match self.phase {
            Phase::Init => {
                self.focus = (cursor_pos.clone(), cursor_pos_next.clone());
                self.init(samples)
            },
            Phase::Run => (),
            Phase::Compare => self.compare(samples),
            Phase::Void => (),
        }
    }
}


/// A `SubcorticalNucleus`.
#[derive(Debug)]
struct EvalSequence {
    area_name: String,
    area_id: usize,
    layers: HashMap<LayerAddress, Layer>,
    cycle_counter: usize,
    sdrs: Arc<Sdrs>,
    sdr_cursor: SeqCursor,
    sampler: Option<CorticalLayerSampler>,
    trials: Qutex<Trials>,
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

        assert!(ENCODE_DIMS_0.v_size() == AREA_DIM && ENCODE_DIMS_0.u_size() == AREA_DIM,
            "For this evaluation, the encoding dims must equal the area dims. \
            The encoding is representative of layer IV output.");

        let sdrs = Arc::new(Sdrs::new(200, ENCODE_DIMS_0));
        // let sdr_cursor = SeqCursor::new((4, 8), 25, sdrs.len());
        let max_seq_len = 5;
        let sdr_cursor = SeqCursor::new((5, 5), 1, sdrs.len());
        let trials = Qutex::new(Trials::new(max_seq_len, sdrs.clone()));

        EvalSequence {
            area_name: area_name,
            area_id: area_scheme.area_id(),
            layers,
            cycle_counter: 0,
            sdrs,
            sdr_cursor,
            sampler: None,
            trials,
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

        self.sampler = Some(CorticalLayerSampler::builder(PRI_AREA, "iii", thal, cortical_areas)
            .axons()
            .soma_states()
            .tuft_states()
            .tuft_best_den_ids()
            .tuft_best_den_states_raw()
            .tuft_best_den_states()
            .den_states()
            .syn_states()
            .syn_strengths()
            .syn_flag_sets()
            .syn_src_slc_ids()
            .syn_src_col_v_offs()
            .syn_src_col_u_offs()
            .build());

        Ok(())
    }

    /// Pre-cycle:
    ///
    /// * Writes output SDR to thalamic tract
    /// *
    ///
    fn pre_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            completion_pool: &mut CompletionPool) -> CmnResult<()> {
        let pattern_idx = self.sdr_cursor.incr_src_idx();

        // // Turn off backpressure for frames we are not interested in:
        // if self.cycle_counter % PRINT_INTERVAL == PRINT_INTERVAL_START {
        //     self.sampler.as_ref().unwrap().set_backpressure(true);
        // }
        // if self.cycle_counter % PRINT_INTERVAL == PRINT_INTERVAL_END {
        //     self.sampler.as_ref().unwrap().set_backpressure(false);
        // }

        // Write sdr to pathway:
        for layer in self.layers.values() {
            if let Pathway::Output { ref tx } = layer.pathway {
                debug_assert!(layer.sub().axon_domain().is_output());

                match layer.sub().name() {
                    "external_0" => {
                        let sdrs = self.sdrs.clone();

                        let future_write_guard = tx.send()
                            .map(|buf_opt| buf_opt.map(|buf| buf.write_u8()))
                            .err_into()
                            .flatten();

                        let future_write = future_write_guard
                            // .join(future_sdrs)
                            .map(move |tract_opt| {
                                tract_opt.map(|mut t| {
                                    debug_assert!(t.len() == sdrs.sdrs[pattern_idx].len());
                                    t.copy_from_slice(&sdrs.sdrs[pattern_idx]);
                                });
                            })
                            .map_err(|err| panic!("{}", err));

                        completion_pool.complete_work(Box::new(future_write))?;
                    },
                    _ => (),
                }
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
        for layer in self.layers.values() {
            if let Pathway::Input { srcs: _ } = layer.pathway {
                debug_assert!(layer.sub().axon_domain().is_input());
            }
        }

        let cycle_counter = self.cycle_counter;
        let cursor_pos = self.sdr_cursor.curr_pos();
        let next_cursor_pos = self.sdr_cursor.next_pos();

        let future_recv = self.sampler.as_ref().unwrap().recv()
            .join(self.trials.clone().lock().err_into())
            .map(move |(samples, mut trials)| {
                trials.cycle(cycle_counter, &samples, &cursor_pos,
                    &next_cursor_pos);
            })
            .map_err(|err| panic!("{}", err));

        completion_pool.complete_work(Box::new(future_recv))?;

        self.cycle_counter += 1;
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
            //     .depth(5)
            //     .tags(LayerTags::PSAL)
            //     .axon_domain(AxonDomain::output(&[at1]))
            //     .cellular(CellScheme::spiny_stellate()
            //         .tft(TuftScheme::basal().proximal()
            //             .syns_per_den(32)
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
                .depth(5)
                // .depth(1)
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
                        .dens_per_tft(2)
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
        .area(AreaScheme::new(IN_AREA, "v0_lm", ENCODE_DIMS_0.v_size())
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

    let controls = ::spawn_threads(cortex, PRI_AREA, false);

    controls.cmd_tx.send(Command::Iterate(1000)).unwrap();
    controls.cmd_tx.send(Command::ExitAfterCycling).unwrap();

    ::join_threads(controls)
}