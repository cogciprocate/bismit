#![allow(dead_code)]

// use std::mem;
use std::collections::{HashMap};
use rand::{self, XorShiftRng};
use rand::distributions::{Range, IndependentSample};
use qutex::QrwLock;
use vibi::bismit::futures::Future;
use vibi::bismit::{map, encode, Result as CmnResult, Cortex, CorticalAreaSettings, Thalamus,
    SubcorticalNucleus, SubcorticalNucleusLayer, WorkPool, /*WorkPoolRemote,*/ MapStore,
    CorticalArea, /*SamplerKind,*/ /*SamplerBufferKind*/};
use vibi::bismit::map::*;
use ::{IncrResult, TrialIter, Layer};
use ::spatial::{/*self, Samplers,*/ TrialData, TrialResults};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 48;
const AREA_DIM: u32 = 16;
const SEQUENTIAL_SDR: bool = true;


/// A `SubcorticalNucleus` which runs several evaluations of a spiny stellate
/// cell layer and its accompanying control cells (smoother).
struct EvalMotor {
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
    rng: XorShiftRng,
    // samplers: Option<Samplers>,
}

impl EvalMotor {
    pub fn new<S: Into<String>>(layer_map_schemes: &LayerMapSchemeList,
            area_schemes: &AreaSchemeList, area_name: S)
            -> EvalMotor {
        let area_name = area_name.into();
        let area_scheme = &area_schemes[&area_name];
        let layer_map_scheme = &layer_map_schemes[area_scheme.layer_map_name()];
        let mut layers = HashMap::with_capacity(4);

        for layer_scheme in layer_map_scheme.layers() {
            let sub_layer = SubcorticalNucleusLayer::from_schemes(layer_scheme, area_scheme, None);

            let layer = Layer {
                sub: sub_layer,
                pathway: None,
            };

            layers.insert(layer.sub().addr().clone(), layer);
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

        EvalMotor {
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
            rng,
            // samplers: None,
        }
    }
}

impl SubcorticalNucleus for EvalMotor {
    fn create_pathways(&mut self, thal: &mut Thalamus,
            _cortical_areas: &mut MapStore<&'static str, CorticalArea>) -> CmnResult<()> {
        // Wire up output (sdr) pathways.
        for layer in self.layers.values_mut() {
            let tx = thal.input_pathway(*layer.sub().addr(), true);
            layer.pathway = Some(tx);
        }

        Ok(())
    }

    /// Pre-cycle:
    ///
    /// * Writes output SDR to thalamic tract
    /// *
    ///
    fn pre_cycle(&mut self, _thal: &mut Thalamus, work_pool: &mut WorkPool) -> CmnResult<()> {
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
                .map_err(|err| panic!("{:?}", err));

            work_pool.submit_work(future_write)?;
        }

        Ok(())
    }

    /// Post-cycle:
    ///
    /// * Blocks to wait for sampler channels
    /// * Increments the cell activity counts
    ///
    fn post_cycle(&mut self, _thal: &mut Thalamus, _work_pool: &mut WorkPool) -> CmnResult<()> {
        if self.trial_iter.current_counter().is_collecting() {

        }

        match self.trial_iter.incr() {
            IncrResult::TrialComplete { scheme_idx: _, train: _, collect: _ } => {

            },
            _ir @ _ => {
                if self.trial_iter.current_counter.is_last_cycle() {

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

    let eval_nucl = EvalMotor::new(&layer_map_schemes,
        &area_schemes, IN_AREA);

    let cortex_builder = Cortex::builder(layer_map_schemes, area_schemes)
        .ca_settings(ca_settings())
        .subcortical_nucleus(eval_nucl);

    let cortex = cortex_builder.build().unwrap();

    let controls = ::spawn_threads(cortex, PRI_AREA);

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
        // .disable_mcols()
        // .disable_regrowth()
        // .disable_learning()
        // .build_opt(BuildOpt::cmplr_def("DEBUG_SMOOTHER_OVERLAP", 1))
}