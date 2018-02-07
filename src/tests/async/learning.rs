#![allow(dead_code, unused_imports, unused_variables)]

extern crate qutex;

use std::collections::{HashMap};
use self::qutex::QrwLock;
use futures::Future;
use ::{map, Result as CmnResult, Cortex, CorticalAreaSettings, Thalamus,
    SubcorticalNucleus, SubcorticalNucleusLayer, WorkPool, CorticalAreas};
use map::*;
use cmn::{TractFrameMut, TractDims};
use tests::testbed;

static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";

const ENCODE_DIMS_0: (u32, u32, u8) = (48, 48, 1);
const ENCODE_DIMS_1: (u32, u32, u8) = (30, 255, 1);
const AREA_DIM: u32 = 16;
const SEQUENTIAL_SDR: bool = true;


/// A `SubcorticalNucleus`.
struct LearningTest {
    area_name: String,
    area_id: usize,
    layers: HashMap<LayerAddress, SubcorticalNucleusLayer>,
    cycles_complete: usize,
}

impl LearningTest {
    pub fn new<S: Into<String>>(layer_map_schemes: &LayerMapSchemeList,
            area_schemes: &AreaSchemeList, area_name: S)
            -> LearningTest {
        let area_name = area_name.into();
        let area_scheme = &area_schemes[&area_name];
        let layer_map_scheme = &layer_map_schemes[area_scheme.layer_map_name()];
        let mut layers = HashMap::with_capacity(4);

        for layer_scheme in layer_map_scheme.layers() {
            // let lyr_dims = match layer_scheme.name() {
            //     "external_0" => None,
            //     ln @ _ => panic!("LearningTest::new: Unknown layer name: {}.", ln),
            // };

            let layer = SubcorticalNucleusLayer::from_schemes(layer_scheme, area_scheme, None);
            layers.insert(layer.addr().clone(), layer);
        }

        LearningTest {
            area_name: area_name,
            area_id: area_scheme.area_id(),
            layers,
            cycles_complete: 0,
        }
    }
}

impl SubcorticalNucleus for LearningTest {
    fn create_pathways(&mut self, thal: &mut Thalamus,
            cortical_areas: &mut CorticalAreas) -> CmnResult<()> {

        Ok(())
    }

    /// Pre-cycle:
    ///
    /// * Writes output SDR to thalamic tract
    /// *
    ///
    fn pre_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            work_pool: &mut WorkPool) -> CmnResult<()> {
        // let pattern_idx = if SEQUENTIAL_SDR {
        //     // Choose a non-random SDR:
        //     self.trial_iter.global_cycle_idx % self.sdrs.pattern_count
        // } else {
        //     // Choose a random SDR:
        //     Range::new(0, self.sdrs.pattern_count).ind_sample(&mut self.sdrs.rng)
        // };

        // Write sdr to pathway:
        for layer in self.layers.values() {
            // if let PathwayDir::Output { ref tx } = layer.pathway {
            //     debug_assert!(layer.axon_domain().is_output());

            //     match layer.sub().name() {
            //         "external_0" => {
            //             let future_sdrs = self.sdrs.lock.clone().read().from_err();

            //             let future_write_guard = tx.send()
            //                 .map(|buf_opt| buf_opt.map(|buf| buf.write_u8()))
            //                 .flatten();

            //             let future_write = future_write_guard
            //                 .join(future_sdrs)
            //                 .map(move |(tract_opt, sdrs)| {
            //                     tract_opt.map(|mut t| {
            //                         debug_assert!(t.len() == sdrs[pattern_idx].len());
            //                         t.copy_from_slice(&sdrs[pattern_idx]);
            //                     });
            //                 })
            //                 .map_err(|err| panic!("{}", err));

            //             work_pool.complete_work(future_write)?;
            //         },
            //         // "external_1" => {
            //         //     let mut write_guard = tx.send()
            //         //         .map(|buf_opt| buf_opt.map(|buf| buf.write_u8()))
            //         //         .flatten()
            //         //         .wait()
            //         //         .expect("future err")
            //         //         .expect("write guard is None");

            //         //     let x = (self.cycles_complete as f64 / 10000.).cos();
            //         //     let y = (self.cycles_complete as f64 / 10000.).sin();

            //         //     // self.encoder_2d.encode([x, y], &mut write_guard);
            //         //     // work_pool.complete_work(  )?;
            //         // },
            //         _ => (),
            //     }
            // }
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
            _work_pool: &mut WorkPool) -> CmnResult<()> {
        for layer in self.layers.values() {
            // if let PathwayDir::Input { srcs: _ } = layer.pathway {
            //     debug_assert!(layer.sub().axon_domain().is_input());
            // }
        }

        // match self.trial_iter.incr() {
        //     IncrResult::TrialComplete { scheme_idx: _, train: _, collect: _ } => {},
        //     _ir @ _ => {},
        // }

        Ok(())
    }

    fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer> {
        self.layers.get(&addr)
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
            .layer(LayerScheme::define("iv")
                .depth(1)
                .tags(LayerTags::PSAL)
                .axon_domain(AxonDomain::output(&[at1]))
                .cellular(CellScheme::spiny_stellate()
                    .tft(TuftScheme::basal().proximal()
                        .syns_per_den(32)
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
                            host_lyr_name: "iv".into(),
                            field_radius: 4,
                        },
                        0
                    )
                )
            )
            .layer(LayerScheme::define("iv_smooth")
                .cellular(CellScheme::control(
                        ControlCellKind::ActivitySmoother {
                            host_lyr_name: "iv".into(),
                            field_radius: 4,
                        },
                        1
                    )
                )
            )
            .layer(LayerScheme::define("iii")
                .depth(8)
                .tags(LayerTags::PTAL)
                .axon_domain(AxonDomain::output(&[at2]))
                .cellular(CellScheme::pyramidal()
                    .tft(TuftScheme::basal().proximal()
                        .syns_per_den(1)
                        .src_lyr(TuftSourceLayer::define("iv")
                            .syn_reach(0)
                            .prevalence(1)
                        )
                    )
                    .tft(TuftScheme::basal().distal()
                        .dens_per_tft(16)
                        .syns_per_den(32)
                        .max_active_dens_l2(3)
                        .thresh_init(500)
                        .src_lyr(TuftSourceLayer::define("iii")
                            .syn_reach(7)
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


#[test]
fn learning() {
    let layer_map_schemes = define_lm_schemes();
    let area_schemes = define_a_schemes();

    let nucl = LearningTest::new(&layer_map_schemes, &area_schemes, IN_AREA);

    let cortex_builder = Cortex::builder(layer_map_schemes, area_schemes)
        .subcortical_nucleus(nucl);

    let cortex = cortex_builder.build().unwrap();
}