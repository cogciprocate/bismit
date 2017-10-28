#![allow(unused_imports, dead_code, unused_mut)]

use std::collections::{BTreeMap, HashMap};
use rand;
use rand::distributions::{Range, IndependentSample};
use vibi::bismit::map::*;
use vibi::bismit::ocl::{Buffer, /*RwVec,*/ WriteGuard};
use vibi::bismit::{map, Cortex, Thalamus, SubcorticalNucleus, CorticalAreaSettings, Subcortex};
use vibi::bismit::flywheel::{Command, Request, Response};
// use vibi::bismit::map::{AreaMap};
use vibi::bismit::encode::{self};
use ::{Controls, Params};

static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 48;
const AREA_DIM: u32 = 16;
const SEQUENTIAL_SDR: bool = true;


// pub(crate) struct Nucleus {
//     area_name: String,
// }

// impl Nucleus {
//     pub fn new<S: Into<String>>(area_name: S, _lyr_name: &'static str, _tar_area: &'static str,
//             _cortex: &Cortex) -> Nucleus
//     {
//         let area_name = area_name.into();

//         Nucleus {
//             area_name: area_name.into()
//         }
//     }
// }

// impl SubcorticalNucleus for Nucleus {
//     fn area_name<'a>(&'a self) -> &'a str { &self.area_name }
//     fn pre_cycle(&mut self, _thal: &mut Thalamus) {}
//     fn post_cycle(&mut self, _thal: &mut Thalamus) {}
// }


pub fn eval() {

    // let mut cortex = Cortex::new(define_lm_schemes(), define_a_schemes(), Some(ca_settings()));
    let mut cortex = Cortex::builder(define_lm_schemes(), define_a_schemes())
        .ca_settings(ca_settings())
        // .sub(subcortex)
        .build().unwrap();

    // let nucl = Nucleus::new(IN_AREA, EXT_LYR, PRI_AREA, &cortex);
    // cortex.add_subcortex(Subcortex::new().nucl(nucl));

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
        .area(AreaScheme::new("v0", "v0_lm", ENCODE_DIM)
            .subcortex()
        )
        .area(AreaScheme::new(PRI_AREA, "visual", AREA_DIM)
            .eff_areas(vec!["v0"])
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