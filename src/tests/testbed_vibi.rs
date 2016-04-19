use cortex::Cortex;
use map::{self, LayerTags};
use map::{LayerMapScheme, LayerMapSchemeList, AreaSchemeList, Axonal, Spatial, Horizontal, 
    Cortical, Thalamic, CellScheme, FilterScheme, InputScheme};

pub fn define_plmaps() -> LayerMapSchemeList {
    const MOTOR_UID: u32 = 654;
    const ROSE_UID: u32 = 435;

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("cortical_lm", Cortical)
            .axn_layer("motor_ctx", map::NS_IN | LayerTags::uid(MOTOR_UID), Horizontal)
            .axn_layer("rose_ctx", map::NS_IN | LayerTags::uid(ROSE_UID), Horizontal)
            .axn_layer("eff_in", map::FB_IN, Spatial)
            .axn_layer("aff_in", map::FF_IN, Spatial)
            .axn_layer("unused", map::UNUSED_TESTING, Spatial)
            .layer("mcols", 1, map::FF_FB_OUT, CellScheme::minicolumn("iv", "iii"))
            .layer("iv_inhib", 0, map::DEFAULT, CellScheme::inhibitory(4, "iv"))

            .layer("iv", 1, map::PSAL, 
                CellScheme::spiny_stellate(4, vec!["aff_in"], 400, 8))

            .layer("iii", 2, map::PTAL, 
                CellScheme::pyramidal(1, 4, vec!["iii"], 800, 10)
                    .apical(vec!["eff_in"/*, "olfac"*/], 12))
        )
        .lmap(LayerMapScheme::new("gly_seq_lm", Thalamic)
            .layer("spatial", 1, map::FF_OUT, Axonal(Spatial))
            .layer("horiz_ns", 1, map::NS_OUT | LayerTags::uid(MOTOR_UID), Axonal(Horizontal))
        )
        .lmap(LayerMapScheme::new("gly_seq_rose_lm", Thalamic)
            .layer("spatial", 1, map::FF_OUT | LayerTags::uid(9999) , Axonal(Spatial))
            .layer("horiz_ns", 1, map::NS_OUT | LayerTags::uid(ROSE_UID), Axonal(Horizontal))
        )
        // .lmap(LayerMapScheme::new("o0_lm", Thalamic)
        //     .layer("ganglion", 1, map::NS_OUT | LayerTags::uid(OLFAC_UID), Axonal(Horizontal))
        // )
}


pub fn define_pamaps() -> AreaSchemeList {
    const AREA_SIDE: u32 = 32;

    AreaSchemeList::new()        
        .area_ext("v0", "gly_seq_lm", AREA_SIDE,
            InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 10, scale: 1.4, hrz_dims: (16, 16) },
            None, 
            None,
        )
        .area_ext("v00", "gly_seq_rose_lm", AREA_SIDE,
            InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 1, scale: 1.4, hrz_dims: (16, 16) },
            None, 
            None,
        )
        .area("v1", "cortical_lm", AREA_SIDE, 
            Some(vec![FilterScheme::new("retina", None)]),            
            Some(vec!["v0", "v00"]),
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
pub fn disable_stuff(cortex: &mut Cortex) {
    for (_, area) in &mut cortex.areas {
        // area.psal_mut().dens_mut().syns_mut().set_offs_to_zero_temp();
        // area.bypass_inhib = true;
        // area.bypass_filters = true;
        // area.disable_pyrs = true;

        // area.disable_ssts = true;
        // area.disable_mcols = true;

        // area.disable_learning = true;
        // area.disable_regrowth = true;
    }
}


pub fn new_cortex() -> Cortex {
    let mut cortex = Cortex::new(define_plmaps(), define_pamaps());
    disable_stuff(&mut cortex);
    cortex
}