//! Determine how well a layer of pyramidal cells can predict the next input
//! in a learned sequence of inputs.
//!
//!
//!

#![allow(dead_code, unused_imports, unused_variables)]

// use std::mem;
use std::collections::{HashMap};
use rand::{self, FromEntropy, rngs::SmallRng};
use rand::distributions::{Range, Distribution};
use qutex::QrwLock;
use vibi::bismit::futures::Future;
use vibi::bismit::{map, Result as CmnResult, Cortex, CorticalAreaSettings, Thalamus,
    SubcorticalNucleus, SubcorticalNucleusLayer, CompletionPool, CorticalAreas};
use vibi::bismit::map::*;
use vibi::bismit::cmn::{TractFrameMut, TractDims};
use vibi::bismit::encode::{self, Vector2dWriter};
use ::{IncrResult, TrialIter, Layer, Pathway, InputSource, Sdrs};
use ::spatial::{TrialData, TrialResults};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIMS_0: (u32, u32, u8) = (48, 48, 1);
const ENCODE_DIMS_1: (u32, u32, u8) = (30, 255, 1);
const AREA_DIM: u32 = 16;
const SEQUENTIAL_SDR: bool = true;


/// A `SubcorticalNucleus`.
struct EvalSequence {
    area_name: String,
    area_id: usize,
    layers: HashMap<LayerAddress, Layer>,
    cycles_complete: usize,
    sdrs: Sdrs,
    trial_iter: TrialIter,
    // current_pattern_idx: usize,
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
                // "external_1" => Some(ENCODE_DIMS.into()),
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

        let sdrs = Sdrs::new(100, ENCODE_DIMS_0);

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
            trial_iter,
            // current_pattern_idx: 0,
        }
    }
}

impl SubcorticalNucleus for EvalSequence {
    fn create_pathways(&mut self, thal: &mut Thalamus,
            _cortical_areas: &mut CorticalAreas) -> CmnResult<()> {
        // Wire up I/O pathways:
        for layer in self.layers.values_mut() {
            layer.pathway = Pathway::new(thal, layer.sub());
        }
        Ok(())
    }

    /// Pre-cycle:
    ///
    /// * Writes output SDR to thalamic tract
    /// *
    ///
    fn pre_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            completion_pool: &mut CompletionPool) -> CmnResult<()> {
        let pattern_idx = if SEQUENTIAL_SDR {
            // Choose a non-random SDR:
            self.trial_iter.global_cycle_idx % self.sdrs.pattern_count
        } else {
            // Choose a random SDR:
            Range::new(0, self.sdrs.pattern_count).sample(&mut self.sdrs.rng)
        };

        // Write sdr to pathway:
        for layer in self.layers.values() {
            if let Pathway::Output { ref tx } = layer.pathway {
                debug_assert!(layer.sub().axon_domain().is_output());

                match layer.sub().name() {
                    "external_0" => {
                        let future_sdrs = self.sdrs.lock.clone().read().err_into();

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

                        completion_pool.complete_work(future_write)?;
                    },
                    // "external_1" => {
                    //     let mut write_guard = tx.send()
                    //         .map(|buf_opt| buf_opt.map(|buf| buf.write_u8()))
                    //         .flatten()
                    //         .wait()
                    //         .expect("future err")
                    //         .expect("write guard is None");

                    //     let x = (self.cycles_complete as f64 / 10000.).cos();
                    //     let y = (self.cycles_complete as f64 / 10000.).sin();

                    //     // self.encoder_2d.encode([x, y], &mut write_guard);
                    //     // completion_pool.complete_work(  )?;
                    // },
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
            _completion_pool: &mut CompletionPool) -> CmnResult<()> {
        for layer in self.layers.values() {
            if let Pathway::Input { srcs: _ } = layer.pathway {
                debug_assert!(layer.sub().axon_domain().is_input());
            }
        }

        match self.trial_iter.incr() {
            IncrResult::TrialComplete { scheme_idx: _, train: _, collect: _ } => {},
            _ir @ _ => {},
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

    let eval_nucl = EvalSequence::new(&layer_map_schemes,
        &area_schemes, IN_AREA);

    let cortex_builder = Cortex::builder(layer_map_schemes, area_schemes)
        .ca_settings(ca_settings())
        .subcortical_nucleus(eval_nucl);

    let cortex = cortex_builder.build().unwrap();

    let controls = ::spawn_threads(cortex, PRI_AREA);

    ::join_threads(controls)
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
            .layer(LayerScheme::define(SPT_LYR)
                .depth(1)
                .tags(LayerTags::PSAL)
                .axon_domain(AxonDomain::output(&[at1]))
                .cellular(CellScheme::spiny_stellate()
                    .tft(TuftScheme::basal().proximal()
                        .syns_per_den_l2(5)
                        .src_lyr(TuftSourceLayer::define("aff_in_0")
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
            .layer(LayerScheme::define("v")
                .depth(1)
                .tags(LayerTags::PML)
                .axon_domain(AxonDomain::output(&[at2]))
                .cellular(CellScheme::pyramidal()
                    // .tft(TuftScheme::basal().proximal()
                    //     .syns_per_den_l2(3)
                    //     .src_lyr(TuftSourceLayer::define(SPT_LYR)
                    //         .syn_reach(0)
                    //         .prevalence(1)
                    //     )
                    // )
                    .tft(TuftScheme::basal().distal()
                        .dens_per_tft_l2(4)
                        .syns_per_den_l2(5)
                        .max_active_dens_l2(2)
                        .thresh_init(500)
                        .src_lyr(TuftSourceLayer::define("v")
                            .syn_reach(5)
                            .prevalence(1)
                        )
                    )
                )
            )
            .layer(LayerScheme::define("v_inhib_col")
                .cellular(CellScheme::control(
                        ControlCellKind::IntraColumnInhib {
                            host_lyr_name: "v".into(),
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