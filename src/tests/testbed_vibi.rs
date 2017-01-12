//! A complex but (linearly) small set of layer and area maps.
///
/// Must be usable for cycling 10k+ times in less than a minute on limited
/// hardware (CPU, etc.).
///

use cortex::Cortex;
use map::{self, LayerMapScheme, LayerMapSchemeList, LayerMapKind, AreaScheme,
    AreaSchemeList, CellScheme, InputScheme, AxonTopology, LayerKind, AxonDomain,
    AxonTag, InputTrack};
use encode::GlyphSequences;

// const MOTOR_UID: u16 = 654;
const ROSE_UID: u16 = 435;

/// A complex but (linearly) small set of layer maps.
pub fn define_layer_map_sl() -> LayerMapSchemeList {
    // let motor_tag = AxonTag::unique();
    // let rose_tag = AxonTag::unique();
    // let unused_tag = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("cortical_lm", LayerMapKind::Cortical)
            .input_layer("motor_ctx", map::DEFAULT,
                AxonDomain::input(&[(InputTrack::Afferent, GlyphSequences::val_lyr_tags())]),
                AxonTopology::Horizontal
            )
            .input_layer("rose_ctx", map::DEFAULT,
                AxonDomain::input(&[(InputTrack::Other,
                    &GlyphSequences::val_lyr_tags() | &AxonTag::custom(ROSE_UID).into()
                )]),
                AxonTopology::Horizontal
            )
            .input_layer("eff_in", map::DEFAULT,
                AxonDomain::input(&[(InputTrack::Efferent, [map::THAL_SP])]),
                AxonTopology::Spatial
            )
            .input_layer("aff_in", map::DEFAULT,
                AxonDomain::input(&[(InputTrack::Afferent, GlyphSequences::img_lyr_tags())]),
                AxonTopology::Spatial
            )
            .input_layer("unused", map::UNUSED_TESTING, AxonDomain::Local, AxonTopology::Spatial)
            .layer("mcols", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
                CellScheme::minicolumn("iv", "iii"))
            .layer("iv_inhib", 0, map::DEFAULT, AxonDomain::Local, CellScheme::inhibitory(4, "iv"))

            // .layer("iv", 1, map::PSAL, AxonDomain::Local,
            //     CellScheme::spiny_stellate(4, vec!["aff_in"], 400, 8))

            // .layer("iii", 2, map::PTAL, AxonDomain::Local,
            //     CellScheme::pyramidal(1, 4, vec!["iii"], 800, 10)
            //         .apical(vec!["eff_in"/*, "olfac"*/], 12))

            .layer("iv", 1, map::PSAL, AxonDomain::Local,
                CellScheme::spiny_stellate(&[("aff_in", 8, 1)], 4, 300)
            )

            .layer("iii", 2, map::PTAL, AxonDomain::Local,
                CellScheme::pyramidal(&[("iii", 10, 1)], 1, 3, 300)
                    // .apical(&[("eff_in", 12)], 1, 4, 800)
            )
        )
        .lmap(LayerMapScheme::new("gly_seq_lm", LayerMapKind::Subcortical)
            .layer("spatial", 1, map::DEFAULT,
                AxonDomain::output(GlyphSequences::img_lyr_tags()),
                LayerKind::Axonal(AxonTopology::Spatial))
            .layer("horiz_ns", 1, map::DEFAULT,
                AxonDomain::output(GlyphSequences::val_lyr_tags()),
                LayerKind::Axonal(AxonTopology::Horizontal))
        )
        .lmap(LayerMapScheme::new("gly_seq_rose_lm", LayerMapKind::Subcortical)
            .layer("spatial_rose", 1, map::DEFAULT,
                AxonDomain::output(GlyphSequences::img_lyr_tags()),
                LayerKind::Axonal(AxonTopology::Spatial))
            .layer("horiz_ns_rose", 1, map::DEFAULT,
                AxonDomain::output(GlyphSequences::val_lyr_tags()),
                LayerKind::Axonal(AxonTopology::Horizontal))
        )
        // .lmap(LayerMapScheme::new("o0_lm", Thalamic)
        //     .layer("ganglion", 1, map::NS_OUT | LayerTags::uid(OLFAC_UID), LayerKind::Axonal(Horizontal))
        // )
}


/// A complex but (linearly) small set of area maps.
pub fn define_area_sl() -> AreaSchemeList {
    const AREA_SIDE: u32 = 16;

    AreaSchemeList::new()
        // .area_ext("v0", "gly_seq_lm", AREA_SIDE,
        //     InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 10, scale: 1.4, hrz_dims: (16, 16) },
        //     None,
        //     None,
        // )
        // .area_ext("v00", "gly_seq_rose_lm", AREA_SIDE,
        //     InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 1, scale: 1.4, hrz_dims: (16, 16) },
        //     None,
        //     None,
        // )
        // .area("v1", "cortical_lm", AREA_SIDE,
        //     Some(vec![FilterScheme::new("retina", None)]),
        //     Some(vec!["v0", "v00"]),
        // )
        .area(AreaScheme::new("gly_seq", "gly_seq_lm", AREA_SIDE)
            .input(InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 10, scale: 1.4, hrz_dims: (16, 16) })
        )
        .area(AreaScheme::new("gly_seq_rose", "gly_seq_rose_lm", AREA_SIDE)
            .input(InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 1, scale: 1.4, hrz_dims: (16, 16) })
        )
        .area(AreaScheme::new("v1", "cortical_lm", AREA_SIDE,)
            .eff_areas(vec!["gly_seq"])
            .other_area("gly_seq_rose", Some(
                &[(GlyphSequences::val_lyr_tags(),
                    &GlyphSequences::val_lyr_tags() | &AxonTag::custom(ROSE_UID).into())]
            ))
            // .filter_chain(map::FF_IN, vec![FilterScheme::new("retina", None)]),
            .filter_chain(InputTrack::Afferent, GlyphSequences::img_lyr_tags(), &[("retina", None)]),
        )

        // .area("b1", "visual", AREA_SIDE, None, Some(vec!["v1"]))

        // .area("a1", "visual", AREA_SIDE, None, Some(vec!["b1"]))
        // .area("a2", "visual", AREA_SIDE, None, Some(vec!["a1"]))
        // .area("a3", "visual", AREA_SIDE, None, Some(vec!["a2"]))
        // .area("a4", "visual", AREA_SIDE, None, Some(vec!["a3"]))
        // .area("a5", "visual", AREA_SIDE, None, Some(vec!["a4"]))
        // .area("a6", "visual", AREA_SIDE, None, Some(vec!["a5"]))
        // .area("a7", "visual", AREA_SIDE, None, Some(vec!["a6"]))
        // .area("a8", "visual", AREA_SIDE, None, Some(vec!["a7"]))
        // .area("a9", "visual", AREA_SIDE, None, Some(vec!["a8"]))
        // .area("aA", "visual", AREA_SIDE, None, Some(vec!["a9"]))
        // .area("aB", "visual", AREA_SIDE, None, Some(vec!["aA"]))
        // .area("aC", "visual", AREA_SIDE, None, Some(vec!["aB"]))
        // .area("aD", "visual", AREA_SIDE, None, Some(vec!["aC"]))
        // .area("aE", "visual", AREA_SIDE, None, Some(vec!["aD"]))
        // .area("aF", "visual", AREA_SIDE, None, Some(vec!["aE"]))

}

#[allow(unused_variables)]
pub fn disable_stuff(_: &mut Cortex) {
    // for (_, area) in &mut cortex.areas {
    //     // area.psal_mut().dens_mut().syns_mut().set_offs_to_zero_temp();
    //     // area.bypass_inhib = true;
    //     // area.bypass_filters = true;
    //     // area.disable_pyrs = true;

    //     // area.disable_ssts = true;
    //     // area.disable_mcols = true;

    //     // area.disable_learning = true;
    //     // area.disable_regrowth = true;
    // }
}


pub fn new_cortex() -> Cortex {
    let mut cortex = Cortex::new(define_layer_map_sl(), define_area_sl(), None);
    disable_stuff(&mut cortex);
    cortex
}