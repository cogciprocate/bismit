//! Encode a sequence of scalar values and display their representation.

#![allow(unused_imports)]

extern crate vibi;
#[macro_use] extern crate lazy_static;

use vibi::window;
use vibi::bismit::{Cortex, CorticalAreaSettings, Subcortex, Flywheel, TestScNucleus};
use vibi::bismit::map::{self, LayerTags, LayerMapKind, LayerMapScheme, LayerMapSchemeList,
    AreaSchemeList, CellScheme, FilterScheme, InputScheme, AxonTopology, LayerKind, AreaScheme,
    AxonDomain, AxonTag, InputTrack, AxonTags};
use vibi::bismit::encode::{ReversoScalarSequence, HexMoldTest};

// const MOTOR_UID: u32 = 101;
// const U0: u16 = 1000;
// const U1: u16 = U0 + 1;

lazy_static! {
    static ref AT0: AxonTag = AxonTag::unique();
    static ref AT1: AxonTag = AxonTag::unique();
}

fn main() {
    use std::thread;
    use std::sync::mpsc;

    let (command_tx, command_rx) = mpsc::channel();
    let (request_tx, request_rx) = mpsc::channel();
    let (response_tx, response_rx) = mpsc::channel();

    let th_flywheel = thread::Builder::new().name("flywheel".to_string()).spawn(move || {
        let cortex = Cortex::new(define_lm_schemes(), define_a_schemes(), Some(ca_settings()))
            .sub(Subcortex::new().nucleus(Box::new(TestScNucleus::new("m0"))));

        /////// [DO NOT REMOVE]: Sets up a custom external pathway:
        // let ep_idx = cortex.thal().ext_pathway_idx(&"v0".to_owned()).unwrap();
        // let ep_area_id = cortex.thal().area_map_by_name("v0").unwrap().area_id();
        // let lyr0_addr = cortex.thal().area_map(ep_area_id).expect("A").layers()
        //     .layer_info_by_sig(&(None, &[map::THAL_SP, AxonTag::custom(U0)]).into())
        //         .expect("B").layer_addr().clone();
        // let lyr1_addr = cortex.thal().area_map(ep_area_id).unwrap().layers()
        //     .layer_info_by_sig(&(None, &[map::THAL_SP, AxonTag::custom(U1)]).into())
        //         .unwrap().layer_addr().clone();
        // cortex.thal_mut().ext_pathway(ep_idx).unwrap().specify_encoder(Box::new(
        //     // HexMoldTest::new(6, [48, 48])
        //     ReversoScalarSequence::new((0.0, 76.0), 1.0, &[lyr0_addr, lyr1_addr])
        // )).unwrap();
        ///////

        // let mut flywheel = Flywheel::from_blueprint(define_lm_schemes(),
        //     define_a_schemes(), None, command_rx);
        let mut flywheel = Flywheel::new(cortex, command_rx, "v1".to_string());
        flywheel.add_req_res_pair(request_rx, response_tx);
        flywheel.spin();
    }).expect("Error creating 'flywheel' thread");

    let th_win = thread::Builder::new().name("win".to_string()).spawn(move || {
        window::Window::open(command_tx, request_tx, response_rx);
    }).expect("Error creating 'win' thread");

    if let Err(e) = th_win.join() { println!("th_win.join(): Error: '{:?}'", e); }
    if let Err(e) = th_flywheel.join() { println!("th_flywheel.join(): Error: '{:?}'", e); }
}

fn define_lm_schemes() -> LayerMapSchemeList {
    // const OLFAC_UID: u32 = 102;
    // let at0 = AxonTag::custom(U0);
    // let at1 = AxonTag::custom(U1);

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("v1_lm", LayerMapKind::Cortical)
            .input_layer("eff_in", map::DEFAULT,
                &[(InputTrack::Efferent, &[map::THAL_SP])],
                AxonTopology::Spatial
            )
            .input_layer("aff_in_0", map::DEFAULT,
                // &[(InputTrack::Afferent, &[map::THAL_SP, AxonTag::custom(U0)])],
                &[(InputTrack::Afferent, &[map::THAL_SP, *AT0])],
                AxonTopology::Spatial
            )
            .input_layer("aff_in_1", map::DEFAULT,
                // &[(InputTrack::Afferent, &[map::THAL_SP, AxonTag::custom(U1)])],
                &[(InputTrack::Afferent, &[map::THAL_SP, *AT1])],
                AxonTopology::Spatial
            )
            .layer("mcols", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
                CellScheme::minicolumn("iv", "iii"))
            .layer("iv_inhib", 0, map::DEFAULT, AxonDomain::Local, CellScheme::inhibitory(4, "iv"))

            .layer("iv", 1, map::PSAL, AxonDomain::Local,
                CellScheme::spiny_stellate(&[("aff_in_0", 14, 1), ("aff_in_1", 14, 1)], 6, 300)
            )

            .layer("iii", 2, map::PTAL, AxonDomain::Local,
                CellScheme::pyramidal(&[("iii", 20, 1)], 1, 5, 300)
                    // .apical(&[("eff_in", 22)], 1, 5, 500)
            )

            // .layer("v", 1, map::PMEL, AxonDomain::Local,
            //     CellScheme::pyramidal(&[("iii", 20, 4), ("v", 20, 1)], 1, 5, 500)
            //         // .apical(vec!["eff_in"/*, "olfac"*/], 18)
            // )
        )
        .lmap(LayerMapScheme::new("v0_lm", LayerMapKind::Subcortical)
            .layer("external_0", 1, map::DEFAULT,
                AxonDomain::output(&[map::THAL_SP, *AT0]),
                LayerKind::Axonal(AxonTopology::Spatial))
            // .layer("external_1", 1, map::DEFAULT,
            //     AxonDomain::output(&[map::THAL_SP, AxonTag::custom(U1)]),
            //     LayerKind::Axonal(AxonTopology::Spatial))
        )
}


fn define_a_schemes() -> AreaSchemeList {
    // let at0 = AxonTag::custom(U0);
    // ENCODE_SIZE: 64 --> range: (0.0, 172.0)
    // ENCODE_SIZE: 32 --> range: (0.0, 76.0)
    const ENCODE_SIZE: u32 = 48; // had been used for GlyphSequences
    const AREA_SIDE: u32 = 32;

    AreaSchemeList::new()
        // .area_ext("v0", "v0_lm", ENCODE_SIZE,
        //     // InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 10, scale: 1.4, hrz_dims: (16, 16) },
        //     // InputScheme::ReversoScalarSequence { range: (0.0, 172.0), incr: 1.0 }, // 64x64
        //     InputScheme::ReversoScalarSequence { range: (0.0, 76.0), incr: 1.0 }, // 32x32
        //     // InputScheme::VectorEncoder { ranges: vec![(0.0, 76.0), (0.0, 76.0)] },
        //     None,
        //     None,
        // )
        // // .area_ext("v0b", "v0b_lm", ENCODE_SIZE,
        // //     InputScheme::SensoryTract,
        // //     None,
        // //     None,
        // // )
        // .area("v1", "visual", AREA_SIDE,
        //     // Some(vec![FilterScheme::new("retina", None)]),
        //     None,
        //     Some(vec!["v0"]),
        //     // Some(vec!["v0b"]),
        // )
        .area(AreaScheme::new("v0", "v0_lm", ENCODE_SIZE)
            // .input(InputScheme::Custom { layer_count: 2 })
            .input(InputScheme::ScalarSdrGradiant { range: (-8.0, 8.0), way_span: 4.0, incr: 0.1 })
        )
        .area(AreaScheme::new("v1", "v1_lm", AREA_SIDE)
            .eff_areas(vec!["v0"])
            // .other_area("v0", Some(&[(&[map::THAL_SP], &[map::L2])]))
            .other_area("v0", None)
            // .filter_chain(map::FF_IN | LayerTags::uid(U0), vec![FilterScheme::new("retina", None)])
            // .filter_chain(map::FF_IN | LayerTags::uid(U1), vec![FilterScheme::new("retina", None)])
            // .filter_chain(InputTrack::Afferent, &[map::THAL_SP, AxonTag::custom(U0)], &[("retina", None)])
            // .filter_chain(InputTrack::Afferent, &[map::THAL_SP, AxonTag::custom(U1)], &[("retina", None)])
        )
        // .area(AreaScheme::new("m1", "m1_lm", AREA_SIDE)
        //     .eff_areas(vec!["v1", "v0"])
        // )
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
