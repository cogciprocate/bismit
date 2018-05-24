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
}


/// A trial result.
#[derive(Clone, Debug)]
struct TrialResult {

}


/// The evaluation state.
#[derive(Debug)]
struct State {
    /// A list of result for each item index in an SDR sequence.
    seq_item_results: Vec<Vec<TrialResult>>,
    sdrs: Arc<Sdrs>,
    phase: Phase,
    focus: (SeqCursorPos, SeqCursorPos),
    // focus.1 cell coords:
    predictor_cells: Vec<(SlcId, u32, u32)>,
}

impl State {
    pub fn new(max_seq_len: usize, sdrs: Arc<Sdrs>) -> State {
        // let predictor_cells = Vec::with_capacity(sdrs.active_cell_count);

        State {
            seq_item_results: vec![Vec::with_capacity(10000); max_seq_len],
            sdrs,
            phase: Phase::Init,
            focus: (SeqCursorPos::default(), SeqCursorPos::default()),
            predictor_cells: Vec::new(),
        }
    }

    fn init(&mut self, samples: &CorticalLayerSamples) {
        // Create a set for efficiency (could be hash or btree):
        let focus_1_set: BTreeSet<u32> = self.sdrs.indices[self.focus.1.pattern_idx].iter()
            .map(|&idx| idx).collect();

        // Allocate:
        self.predictor_cells = Vec::with_capacity(samples.map().dims().depth() as usize *
            self.sdrs.active_cell_count);

        for cell in samples.cells(.., .., ..) {
            let col_id = cell.map().col_id();
            // Determine the set of cells which could eventually learn to
            // become 'predictive' based on step[0] input. Only a fraction
            // (1/depth) of the cells *should* become predictors over time.
            if focus_1_set.contains(&col_id) {
                print!("[s:{}, v:{}, u:{} / col:{}]", cell.map().slc_id_lyr(), cell.map().v_id(),
                    cell.map().u_id(), col_id);
                self.predictor_cells.push((cell.map().slc_id_lyr(), cell.map().v_id(),
                    cell.map().u_id()));
            }
        }
        println!("\nPredictor cell count: {} / {}", self.predictor_cells.len(),
            self.predictor_cells.capacity());


        // Check the strength of the PROXIMAL focus_1 synapses (only one per cell);
    }

    /// Checks stuff.
    fn cycle(&mut self, samples: &CorticalLayerSamples, cycle_counter: usize,
            cursor_pos: &SeqCursorPos, cursor_pos_next: &SeqCursorPos) {
        if cycle_counter % 200 == 0 { println!("{} cycles enqueued.", cycle_counter); }

        match cycle_counter {
            0 => self.phase = Phase::Init,
            1 => self.phase = Phase::Run,
            200 => self.phase = Phase::Compare,
            _ => (),
        }

        match self.phase {
            Phase::Init => {
                self.focus = (cursor_pos.clone(), cursor_pos_next.clone());
                self.init(samples)
            },
            Phase::Run => (),
            Phase::Compare => (),
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
    state: Qutex<State>,
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
        let state = Qutex::new(State::new(max_seq_len, sdrs.clone()));

        EvalSequence {
            area_name: area_name,
            area_id: area_scheme.area_id(),
            layers,
            cycle_counter: 0,
            sdrs,
            sdr_cursor,
            sampler: None,
            state,
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
            .join(self.state.clone().lock().err_into())
            .map(move |(samples, mut state)| {
                state.cycle(&samples, cycle_counter, &cursor_pos,
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
                        .dens_per_tft(4)
                        // .syns_per_den(64)
                        .syns_per_den(16)
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