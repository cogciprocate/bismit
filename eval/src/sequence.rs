//! Determine how well a layer of pyramidal cells can predict the next input
//! in a learned sequence of inputs.
//!
//!
//!

#![allow(dead_code, unused_imports, unused_variables)]

use std::mem;
use std::collections::{HashMap, BTreeMap};
use std::ops::Range;
use rand::{self, XorShiftRng};
use rand::distributions::{Range as RandRange, IndependentSample};
use qutex::QrwLock;
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
use ::{IncrResult, TrialIter, Layer, Pathway, InputSource, Sdrs, SeqCursor};
use ::spatial::{TrialData, TrialResults};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";

const ENCODE_DIMS_0: (u32, u32, u8) = (48, 48, 1);
// const ENCODE_DIMS_1: (u32, u32, u8) = (30, 255, 1);
const AREA_DIM: u32 = 48;
const SEQUENTIAL_SDR: bool = true;


fn print_stuff(samples: CorticalSamples, focus_celtfts: Vec<FocusCellTuft>,
        cycles_complete: usize, lyr_addr: LayerAddress, seq_idx: usize, seq_item_idx: usize)
        -> CorticalSamples {
    if cycles_complete % 5000 >= 5 { return samples; }

    for celtft in &focus_celtfts {
        let cel_idx = celtft.cel_coords.idx() as usize;
        let celtft_idx = celtft.celtft_idx;
        let den_idx_range = &celtft.den_idx_range;
        let syn_idx_range = &celtft.syn_idx_range;

        let axn_states = samples.sample(&SamplerKind::Axons(None)).unwrap().u8();
        let cel_states = samples.sample(&SamplerKind::SomaStates(lyr_addr)).unwrap().u8();
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

        // println!("cel_states[{:?}]: : {:03?}", 32..64, &cel_states[32..64]);

        // println!("[{}] &axn_states[..]: {:03?}", cycles_complete, &axn_states[..]);
        // return;

        if tft_states[celtft_idx] == 0 { continue; }

        println!("(seq: {}, seq_item: {})", seq_idx, seq_item_idx);
        println!("[{}] &cel_states[{}]: {:03?}", cycles_complete, cel_idx,
            &cel_states[cel_idx]);
        println!("[{}] &tft_states[{}]: {:03?}", cycles_complete, celtft_idx,
            &tft_states[celtft_idx]);
        println!("[{}] &tft_best_den_ids[{}]: {:03?}", cycles_complete, celtft_idx,
            &tft_best_den_ids[celtft_idx]);
        println!("[{}] &tft_best_den_states[{}]: {:03?}", cycles_complete, celtft_idx,
            &tft_best_den_states[celtft_idx]);
        println!("[{}] &tft_best_den_states_raw[{}]: {:03?}", cycles_complete, celtft_idx,
            &tft_best_den_states_raw[celtft_idx]);
        println!("[{}] &den_states[{:?}]: {:03?}", cycles_complete, den_idx_range,
            &den_states[den_idx_range.clone()]);
        println!("[{}] &syn_states[{:?}]: {:03?}", cycles_complete, syn_idx_range,
            &syn_states[syn_idx_range.clone()]);
        println!("[{}] &syn_strengths[{:?}]: {:03?}", cycles_complete, syn_idx_range,
            &syn_strengths[syn_idx_range.clone()]);
        // println!("[{}] &syn_src_col_v_offs[{:?}]: {:03?}", cycles_complete, syn_idx_range,
        //     &syn_src_col_v_offs[syn_idx_range.clone()]);
        // println!("[{}] &syn_src_col_u_offs[{:?}]: {:03?}", cycles_complete, syn_idx_range,
        //     &syn_src_col_u_offs[syn_idx_range.clone()]);
        println!("[{}] &syn_flag_sets[{:?}]: {:03?}", cycles_complete, syn_idx_range,
            &syn_flag_sets[syn_idx_range.clone()]);

        println!();
    }
    println!();

    samples
}


#[derive(Clone, Debug)]
struct FocusCellTuft {
    cel_coords: CelCoords,
    tft_id: usize,
    celtft_idx: usize,
    den_idx_range: Range<usize>,
    syn_idx_range: Range<usize>,
}

impl FocusCellTuft {
    fn new(area_name: &'static str, layer_name: &'static str,
            cel_coords: CelCoords, tft_id: usize, cortical_areas: &mut CorticalAreas)
            -> FocusCellTuft {
        let area = cortical_areas.by_key_mut(area_name).unwrap();
        let den_idx_range = area.layer_test_mut(layer_name).unwrap().dens()
            .den_idx_range_celtft(&cel_coords, tft_id);
        let syn_idx_range = area.layer_test_mut(layer_name).unwrap().dens().syns()
            .syn_idx_range_celtft(&cel_coords, tft_id);
        let celtft_idx = area.layer_test_mut(layer_name).unwrap().tufts()
            .celtft_idx(&cel_coords, tft_id);

        FocusCellTuft { cel_coords, tft_id, celtft_idx, den_idx_range, syn_idx_range }
    }

    fn random(area_name: &'static str, layer_name: &'static str, tft_id: usize,
            cortical_areas: &mut CorticalAreas) -> FocusCellTuft {
        let cel_coords = cortical_areas.by_key_mut(area_name).unwrap()
            .layer_test_mut(layer_name).unwrap().rand_cel_coords();
        FocusCellTuft::new(area_name, layer_name, cel_coords, tft_id, cortical_areas)
    }
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
    focus_celtfts: Vec<FocusCellTuft>,
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

        let sdrs = Sdrs::new(25, ENCODE_DIMS_0);
        // let sdr_cursor = SeqCursor::new((4, 8), 25, sdrs.len());
        let sdr_cursor = SeqCursor::new((5, 5), 1, sdrs.len());
        let a_valid_sdr_idx = sdr_cursor.a_middle_src_idx();
        let an_active_cell = sdrs.a_middle_active_cell(a_valid_sdr_idx);
        println!("###### an_active_cell: {:?}", an_active_cell);

        // Define the number of iters to first train then collect for each
        // sample period. All learning and other cell parameters (activity,
        // energy, etc.) persist between sample periods. Only collection
        // iters are recorded and evaluated.
        let trial_iter = TrialIter::new(vec![
            (5000, 5000), (5000, 5000), (5000, 5000), (5000, 5000), (5000, 5000),
        ]);

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
            focus_celtfts: Vec::with_capacity(16),
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

        self.sampler = Some(CorticalSampler::new(PRI_AREA, sampler_kinds, CellSampleIdxs::All,
            thal, cortical_areas));

        self.pri_iii_layer_addr = Some(lyr_addr);

        // Choose a random focus cell tuft:
        let layer_depth = cortical_areas.by_key_mut(PRI_AREA).unwrap()
            .layer_test_mut("iii").unwrap().dims().depth();

        let lyr_axn_slc_idz = cortical_areas.by_key_mut(PRI_AREA).unwrap()
            .layer_test_mut("iii").unwrap().base_axon_slc();

        let cel_coords = cortical_areas.by_key_mut(PRI_AREA).unwrap()
            .layer_test_mut("iii").unwrap().rand_cel_coords();

        for slc_id_lyr in 0..layer_depth {
            let ccs = CelCoords::new(lyr_axn_slc_idz + slc_id_lyr, slc_id_lyr,
                cel_coords.v_id, cel_coords.u_id, cel_coords.lyr_dims.clone());
            // self.focus_celtfts.push(FocusCellTuft::new(PRI_AREA, "iii", ccs.clone(), 0, cortical_areas));
            self.focus_celtfts.push(FocusCellTuft::new(PRI_AREA, "iii", ccs, 1, cortical_areas));
        }

        Ok(())
    }

    /// Pre-cycle:
    ///
    /// * Writes output SDR to thalamic tract
    /// *
    ///
    fn pre_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            work_pool: &mut WorkPool) -> CmnResult<()> {
        let pattern_idx = self.sdr_cursor.next_src_idx();

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

        let focus_celtfts = self.focus_celtfts.clone();
        let cycles_complete = self.cycles_complete;
        let lyr_addr = self.pri_iii_layer_addr.clone().unwrap();
        let seq_idx = self.sdr_cursor.seq_idx();
        let seq_item_idx = self.sdr_cursor.seq_item_idx();

        let future_recv = self.sampler.as_ref().unwrap().recv()
            .map(move |samples| {
                print_stuff(samples, focus_celtfts, cycles_complete, lyr_addr,
                    seq_idx, seq_item_idx);
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
                // .depth(5)
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
                        .dens_per_tft(8)
                        .syns_per_den(32)
                        .max_active_dens_l2(0)
                        .thresh_init(0)
                        .src_lyr(TuftSourceLayer::define("iii")
                            .syn_reach(16)
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