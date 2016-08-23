//! Encode a sequence of scalar values and display their representation.

#![allow(unused_imports)]

extern crate bismit;

use bismit::{Cortex, CorticalAreaSettings};
use bismit::map::{self, LayerTags, LayerMapKind, LayerMapScheme, LayerMapSchemeList,
    AreaSchemeList, CellScheme, FilterScheme, InputScheme, AxonKind, LayerKind};

fn main() {
    let mut cortex = Cortex::new(define_lm_schemes(), define_a_schemes(), None);


}

fn define_lm_schemes() -> LayerMapSchemeList {
    const MOTOR_UID: u32 = 101;
    // const OLFAC_UID: u32 = 102;

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            //.layer("test_noise", 1, map::DEFAULT, LayerKind::Axonal(Spatial))
            .axn_layer("motor_ctx", map::NS_IN | LayerTags::uid(MOTOR_UID), AxonKind::Horizontal)
            // .axn_layer("olfac", map::NS_IN | LayerTags::with_uid(OLFAC_UID), Horizontal)
            // .axn_layer("eff_in", map::FB_IN, AxonKind::Spatial)
            .axn_layer("aff_in", map::FF_IN, AxonKind::Spatial)
            // .axn_layer("out", map::FF_FB_OUT, Spatial)
            .axn_layer("unused", map::UNUSED_TESTING, AxonKind::Spatial)
            .layer("mcols", 1, map::FF_FB_OUT, CellScheme::minicolumn("iv", "iii"))
            .layer("iv_inhib", 0, map::DEFAULT, CellScheme::inhibitory(4, "iv"))

            .layer("iv", 1, map::PSAL,
                CellScheme::spiny_stellate(6, vec!["aff_in"], 400, 14))

            .layer("iii", 2, map::PTAL,
                CellScheme::pyramidal(1, 5, vec!["iii"], 500, 20)
                    // .apical(vec!["eff_in"/*, "olfac"*/], 18)
                )
        )
        .lmap(LayerMapScheme::new("v0_lm", LayerMapKind::Thalamic)
            .layer("external", 3, map::FF_OUT, LayerKind::Axonal(AxonKind::Spatial))
            // .layer("horiz_ns", 1, map::NS_OUT | LayerTags::uid(MOTOR_UID),
            //     LayerKind::Axonal(AxonKind::Horizontal))
        )
        // .lmap(LayerMapScheme::new("v0b_lm", LayerMapKind::Thalamic)
        //     .layer("spatial", 1, map::FF_OUT, LayerKind::Axonal(AxonKind::Spatial))
        //     // .layer("horiz_ns", 1, map::NS_OUT | LayerTags::uid(MOTOR_UID),
        //     //     LayerKind::Axonal(AxonKind::Horizontal))
        // )
}


fn define_a_schemes() -> AreaSchemeList {
    // const CYCLES_PER_FRAME: usize = 1;
    // const HZS: u32 = 16;
    const ENCODE_SIZE: u32 = 64; // had been used for GlyphSequences
    // const ENCODE_SIZE: u32 = 24; // for SensoryTract
    const AREA_SIDE: u32 = 48;

    AreaSchemeList::new()
        .area_ext("v0", "v0_lm", ENCODE_SIZE,
            // InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 10, scale: 1.4, hrz_dims: (16, 16) },
            InputScheme::ScalarSequence { range: (0.0, 172.0), incr: 1.0 },
            None,
            None,
        )
        // .area_ext("v0b", "v0b_lm", ENCODE_SIZE,
        //     InputScheme::SensoryTract,
        //     None,
        //     None,
        // )
        .area("v1", "visual", AREA_SIDE,
            // Some(vec![FilterScheme::new("retina", None)]),
            None,
            Some(vec!["v0"]),
            // Some(vec!["v0b"]),
        )
}

// #########################
// ##### DISABLE STUFF #####
// #########################
#[allow(unused_mut)]
pub fn ca_settings() -> CorticalAreaSettings {
    let mut settings = CorticalAreaSettings::new();

    // settings.bypass_inhib = true;
    // settings.bypass_filters = true;
    // settings.disable_pyrs = true;
    // settings.disable_ssts = true;
    // settings.disable_mcols = true;
    // settings.disable_regrowth = true;
    // settings.disable_learning = true;

    settings
}
