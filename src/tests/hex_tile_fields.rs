#![allow(dead_code, unused_variables, unused_mut)]

use ocl::builders::{BuildOpt};
use ::{map, Cortex, CorticalAreaSettings};
use map::*;

static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 24;
const AREA_DIM: u32 = 16;

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
                CellScheme::spiny_stellate(&[("aff_in", 8, 1)], 7, 400)
            )
            .layer("iv_inhib", 0, map::DEFAULT, AxonDomain::Local, CellScheme::inhib("iv", 4, 0))
            .layer("iv_smooth", 0, map::DEFAULT, AxonDomain::Local, CellScheme::smooth("iv", 6, 1))
            .layer("iii", 1, map::PTAL, AxonDomain::Local,
                CellScheme::pyramidal(&[("iii", 8, 1)], 1, 2, 500)
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
            .subcortex()
        )
        .area(AreaScheme::new(PRI_AREA, "visual", AREA_DIM)
            .eff_areas(vec!["v0"])
            // .filter_chain(map::FF_IN, vec![FilterScheme::new("retina", None)])
        )
}

pub fn ca_settings() -> CorticalAreaSettings {
    let mut settings = CorticalAreaSettings::new();
    // settings.bypass_inhib = true;
    // settings.bypass_filters = true;
    settings.disable_pyrs = true;
    // settings.disable_ssts = true;
    settings.disable_mcols = true;
    settings.disable_regrowth = true;
    settings.disable_learning = true;
    settings.build_opt(BuildOpt::cmplr_def("DEBUG_SMOOTHER_OVERLAP", 1));
    settings
}



/// Tests that all cells are processed an equal number of times by the
/// activity smoother layer.
#[test]
pub fn smoother_overlap() {
    let mut cortex = Cortex::new(define_lm_schemes(), define_a_schemes(), Some(ca_settings()));

    // Layer 4 spatial cell energies:
    let l4_spt_cel_enrgs = cortex.areas().by_key(PRI_AREA).unwrap()
        .psal_TEMP().energies().clone();

    let training_collect_iters = vec![5; 12];
    let cell_count = (AREA_DIM * AREA_DIM) as usize;
    assert_eq!(cell_count, l4_spt_cel_enrgs.len());
    let mut total_cycles = 0usize;

    for (t, iters) in training_collect_iters.into_iter().enumerate() {
        for i in 0..iters {
            cortex.cycle();
        }
        total_cycles += iters;

        let smoother_layers = 6;
        let energy_level_raw = smoother_layers * total_cycles;
        let energy_level = if energy_level_raw > 255 { 255 } else { energy_level_raw as u8 };

        let mut cel_energies_vec = vec![0; l4_spt_cel_enrgs.len()];
        l4_spt_cel_enrgs.read(&mut cel_energies_vec).enq().unwrap();

        for cel_idx in 0..cell_count {
            if cel_energies_vec[cel_idx] != energy_level {
                panic!("Energy level mismatch: expected: {}, found: {}",
                    cel_energies_vec[cel_idx], energy_level);
            }
        }
    }
}

