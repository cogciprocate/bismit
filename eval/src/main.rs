//! Encode a sequence of scalar values and display their representation.

#![allow(unused_imports, unused_variables, dead_code)]

extern crate rand;
extern crate vibi;
extern crate env_logger;
#[macro_use] extern crate log;

mod spatial;

use vibi::window;
use vibi::bismit::{map, Cortex, CorticalAreaSettings, Subcortex};
use vibi::bismit::map::*;
use vibi::bismit::flywheel::Flywheel;
use spatial::Params;

static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 64;
const AREA_DIM: u32 = 16;

fn main() {
    use std::thread;
    use std::sync::mpsc;

    env_logger::init().unwrap();

    let (command_tx, command_rx) = mpsc::channel();
    let (vibi_request_tx, vibi_request_rx) = mpsc::channel();
    let (vibi_response_tx, vibi_response_rx) = mpsc::channel();
    let vibi_command_tx = command_tx.clone();

    let (spatial_request_tx, spatial_request_rx) = mpsc::channel();
    let (spatial_response_tx, spatial_response_rx) = mpsc::channel();
    let spatial_command_tx = command_tx;

    let mut cortex = Cortex::new(define_lm_schemes(), define_a_schemes(), Some(ca_settings()));

    let v0_ext_lyr_addr = *cortex.thal().area_maps().by_key(IN_AREA).expect("bad area")
        .layer_map().layers().by_key(EXT_LYR).expect("bad lyr").layer_addr();

    let v1_spt_lyr_buf = {
        let pri_area_map = cortex.thal().area_maps().by_key(PRI_AREA).expect("bad area");
        let v1_spt_lyr_addr = *pri_area_map.layer_map().layers().by_key(SPT_LYR)
            .expect("bad lyr").layer_addr();
        let v1_spt_lyr_axn_range = pri_area_map.lyr_axn_range(&v1_spt_lyr_addr, None).unwrap();
        cortex.areas().by_key(PRI_AREA).unwrap().axns()
            .create_sub_buffer(&v1_spt_lyr_axn_range).unwrap()
    };

    let in_tract_idx = cortex.thal().tract().index_of(v0_ext_lyr_addr).unwrap();
    let in_tract_buffer = cortex.thal().tract().buffer(in_tract_idx).unwrap().clone();
    let axns = cortex.areas().by_key(PRI_AREA).unwrap().axns().states().clone();
    let area_map = cortex.areas().by_key(PRI_AREA).unwrap().area_map().clone();
    // let ssts =

    let nucl = spatial::Nucleus::new(IN_AREA, EXT_LYR, PRI_AREA, &cortex);
    cortex.add_subcortex(Subcortex::new().nucl(nucl));

    let mut flywheel = Flywheel::new(cortex, command_rx, PRI_AREA);
    flywheel.add_req_res_pair(vibi_request_rx, vibi_response_tx);
    flywheel.add_req_res_pair(spatial_request_rx, spatial_response_tx);

    let th_flywheel = thread::Builder::new().name("flywheel".to_string()).spawn(move || {
        flywheel.spin();
    }).expect("Error creating 'flywheel' thread");

    let th_win = thread::Builder::new().name("win".to_string()).spawn(move || {
        println!("Opening vibi window...");
        window::Window::open(vibi_command_tx, vibi_request_tx, vibi_response_rx);
    }).expect("Error creating 'win' thread");

    let params = Params { cmd_tx: spatial_command_tx, req_tx: spatial_request_tx,
        res_rx: spatial_response_rx, tract_buffer: in_tract_buffer, axns,
        l4_axns: v1_spt_lyr_buf, area_map, encode_dim: ENCODE_DIM, area_dim: AREA_DIM };

    // spatial::eval(spatial_command_tx, spatial_request_tx, spatial_response_rx,
    //     in_tract_buffer, axns, v1_spt_lyr_buf, area_map, ENCODE_DIM, AREA_DIM);
    spatial::eval(params);

    if let Err(e) = th_win.join() { println!("th_win.join(): Error: '{:?}'", e); }
    if let Err(e) = th_flywheel.join() { println!("th_flywheel.join(): Error: '{:?}'", e); }
}

fn define_lm_schemes() -> LayerMapSchemeList {
    let at0 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            .input_layer("aff_in", map::DEFAULT,
                AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]),
                AxonTopology::Spatial
            )
            .layer("mcols", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
                CellScheme::minicolumn("iv", "iii")
            )
            .layer(SPT_LYR, 1, map::PSAL, AxonDomain::Local,
                CellScheme::spiny_stellate(&[("aff_in", 16, 1)], 7, 600)
            )
            .layer("iv_inhib", 0, map::DEFAULT, AxonDomain::Local, CellScheme::inhibitory(4, "iv"))
            .layer("iii", 1, map::PTAL, AxonDomain::Local,
                CellScheme::pyramidal(&[("iii", 20, 1)], 1, 6, 500)
                    // .apical(&[("eff_in", 22)], 1, 5, 500)
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
            // .input(InputScheme::GlyphSequences { seq_lens: (5, 5), seq_count: 10,
            //    scale: 1.4, hrz_dims: (16, 16) }),
            // .input(InputScheme::ScalarSdrGradiant { range: (-8.0, 8.0), way_span: 16.0, incr: 0.1 }),
            // .input(InputScheme::None),
            // .input(InputScheme::Custom { layer_count: 1 }),
            // .custom_layer_count(1)
            .subcortex()
        )
        .area(AreaScheme::new(PRI_AREA, "visual", AREA_DIM)
            .eff_areas(vec!["v0"])
            // .filter_chain(map::FF_IN, vec![FilterScheme::new("retina", None)])
        )
}

// #########################
// ##### DISABLE STUFF #####
// #########################
#[allow(unused_mut)]
pub fn ca_settings() -> CorticalAreaSettings {
    let mut settings = CorticalAreaSettings::new();

    // settings.bypass_inhib = true;
    settings.bypass_filters = true;
    settings.disable_pyrs = true;
    // settings.disable_ssts = true;
    settings.disable_mcols = true;
    // settings.disable_regrowth = true;
    // settings.disable_learning = true;

    settings
}
