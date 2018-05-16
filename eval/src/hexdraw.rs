#![allow(unused_variables, unused_mut, dead_code, unused_imports)]

use std::thread;
use vibi::bismit::ocl::{ WriteGuard};
use vibi::bismit::cmn::{self, TractDims, TractFrameMut};
use vibi::bismit::map::*;
use vibi::bismit::{map, Cortex, CorticalAreaSettings, InputGenerator, /*Subcortex*/};
use vibi::bismit::encode::Vector2dWriter;
use vibi::bismit::flywheel::{Command, Request, Response};
use ::{Controls, Params};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

// const ENCODE_DIM: u32 = 64;
const ENCODE_DIMS: [u32; 2] = [130, 400];
const AREA_DIM: u32 = 32;
const HEX_GRP_RADIUS: usize = 6;

// enum CompletionResult {
//     None,
//     Break,
// }

fn complete(controls: &Controls) {
    // Cycle and finish queues:
    controls.cmd_tx.send(Command::Iterate(1)).unwrap();
    controls.req_tx.send(Request::FinishQueues).unwrap();
    controls.cmd_tx.send(Command::None).unwrap();

    // Wait for completion.
    loop {
        debug!("Attempting to receive...");
        match controls.res_rx.recv() {
            Ok(res) => match res {
                Response::Status(status) => {
                    debug!("Status: {:?}", status);
                },
                Response::QueuesFinished(cycle_iter) => {
                    debug!("Queues finished (cycle: {})", cycle_iter);
                    break;
                },
                Response::Exiting => {
                    // exiting = true;
                    break;
                },
                res @ _ => panic!("Unknown response received: {:?}", res),
            },
            Err(_) => {
                // exiting = true;
                break;
            }
        };
    }
}

pub fn draw_tilegroup(params: &Params, controls: &Controls) {
    debug!("EVAL HEXDRAW DRAW: 0");

    // Populates the thing.
    fn pop(offset: [i32; 2], val: u8, guard: &mut [u8]) {
        let dims = [ENCODE_DIMS[0] as i32, ENCODE_DIMS[1] as i32];
        let start = [(dims[0] / 2) + offset[0], (dims[1] / 2) + offset[1]];
        cmn::populate_hex_tile_grps(HEX_GRP_RADIUS, dims, start, val, guard);
    }

    debug!("EVAL HEXDRAW DRAW: 1000");

    // Write to tract:
    debug!("Locking tract buffer...");
    let mut guard = params.tract_buffer.clone().write().wait().unwrap();
    // assert!(HEX_GRP_RADIUS % 2 == 0);
    let ofs_dist = ((HEX_GRP_RADIUS + 1) / 2) as i32;


    debug!("EVAL HEXDRAW DRAW: 2000");

    // let ofs_dist = HEX_GRP_RADIUS as i32 - 1;

    // pop([0, 0], 255, guard.as_mut_slice());

    pop([0, ofs_dist], 1, guard.as_mut_slice());
    pop([-ofs_dist, 0], 102, guard.as_mut_slice());
    pop([ofs_dist, -ofs_dist], 204, guard.as_mut_slice());

    pop([-ofs_dist, ofs_dist], 51, guard.as_mut_slice());
    pop([0, -ofs_dist], 153, guard.as_mut_slice());
    pop([ofs_dist, 0], 255, guard.as_mut_slice());

    WriteGuard::release(guard);

    debug!("EVAL HEXDRAW DRAW: 4000");

    complete(controls);

    debug!("EVAL HEXDRAW DRAW: 9999");
}


/// Encodes and renders a 2d vector.
pub fn draw_coord(params: &Params, controls: &Controls) {
    let tract_dims = TractDims::new(1, ENCODE_DIMS[0], ENCODE_DIMS[1]);
    let mut encoder = Vector2dWriter::new(tract_dims);

    let mut x = 0.0;
    let mut y = 0.0;

    loop {
        {
            let mut guard = params.tract_buffer.clone().write().wait().unwrap();
            // let mut tract = TractFrameMut::new(guard.as_mut_slice(), tract_dims);

            encoder.encode([x, y], guard.as_mut_slice());
            // WriteGuard::release(guard);
        }

        complete(controls);

        // ::std::thread::sleep(::std::time::Duration::from_millis(1000));
        x -= 0.001;
        y += 1000.00000;
    }
}


/// Draws an arbitrary pattern as an sdr.
pub fn eval(sub: Option<&str>) {
    let layer_map_schemes = define_lm_schemes();
    let area_schemes = define_a_schemes();

    let dummy = InputGenerator::new(&layer_map_schemes, &area_schemes, "v0").unwrap();
    // let subcortex = Subcortex::new().nucleus(dummy);

    let cortex = Cortex::builder(layer_map_schemes, area_schemes)
        .ca_settings(ca_settings())
        .subcortical_nucleus(dummy)
        .build().unwrap();

    // let v0_ext_lyr_addr = cortex.thal().area_maps().by_key(IN_AREA).expect("bad area")
    //     .layer_map().layers().by_key(EXT_LYR).expect("bad lyr").layer_addr();
    let v0_ext_lyr_addr = cortex.thal().layer_addr(IN_AREA, EXT_LYR);

    // let v1_spt_lyr_buf = {
    //     let pri_area_map = cortex.thal().area_maps().by_key(PRI_AREA).expect("bad area");
    //     let v1_spt_lyr_addr = *pri_area_map.layer_map().layers().by_key(SPT_LYR)
    //         .expect("bad lyr").layer_addr();
    //     let v1_spt_lyr_axn_range = pri_area_map.lyr_axn_range(&v1_spt_lyr_addr, None).unwrap();
    //     println!("####### v1_spt_lyr_axn_range: {:?}", v1_spt_lyr_axn_range);
    //     cortex.areas().by_key(PRI_AREA).unwrap().axns()
    //         .create_sub_buffer(&v1_spt_lyr_axn_range).unwrap()
    // };

    let in_tract_idx = cortex.thal().tract().index_of(v0_ext_lyr_addr).unwrap();
    let in_tract_buffer = cortex.thal().tract().buffer_rwvec(in_tract_idx).unwrap().clone();
    let axns = cortex.areas().by_key(PRI_AREA).unwrap().axns().states().clone();
    let area_map = cortex.areas().by_key(PRI_AREA).unwrap().area_map().clone();

    let controls = ::spawn_threads(cortex, PRI_AREA);

    let params = Params { tract_buffer: in_tract_buffer, axns,
        /*l4_axns: v1_spt_lyr_buf,*/ area_map, encode_dim: ENCODE_DIMS, area_dim: AREA_DIM };

    // Get the flywheel moving:
    controls.cmd_tx.send(Command::None).unwrap();

    // Sleep until vibi window opens (need a better mech. for this):
    thread::sleep(::std::time::Duration::new(1, 0));

    match sub {
        None | Some("tilegroup") => {
            draw_tilegroup(&params, &controls);
        }
        Some("coord") => {
            draw_coord(&params, &controls);
        }
        s @ _ => println!("eval-motor: Unknown option specified: {:?}", s),
    }

    ::join_threads(controls);

}


fn define_lm_schemes() -> LayerMapSchemeList {
    let at0 = AxonTag::unique();
    let at1 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            // .input_layer("aff_in", LayerTags::DEFAULT,
            //     AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]),
            //     AxonTopology::Spatial
            // )
            .layer(LayerScheme::define("aff_in")
                .axonal(AxonTopology::Spatial)
                .axon_domain(AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]))
            )
            // .layer_old("dummy_out", 1, LayerTags::DEFAULT, AxonDomain::output(&[AxonTag::unique()]),
            //     LayerKind::Axonal(AxonTopology::Spatial)
            // )
            // .layer_old(SPT_LYR, 1, LayerTags::PSAL, AxonDomain::Local,
            //     CellScheme::ssc(&[("aff_in", 4, 1)], 7, 600)
            // )
            .layer(LayerScheme::define(SPT_LYR)
                .depth(1)
                .tags(LayerTags::PSAL)
                .axon_domain(AxonDomain::output(&[at1]))
                .cellular(CellScheme::spiny_stellate()
                    .tft(TuftScheme::basal().proximal()
                        .syns_per_den(32)
                        .src_lyr(TuftSourceLayer::define("aff_in")
                            .syn_reach(7)
                            .prevalence(1)
                        )
                    )
                )
            )
            // .layer_old("iv_inhib", 0, LayerTags::DEFAULT, AxonDomain::Local,
            //     CellScheme::inhib(SPT_LYR, 4, 0)
            // )
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
            // .layer_old("iii", 1, LayerTags::PTAL, AxonDomain::Local,
            //     CellScheme::pyr(&[("iii", 20, 1)], 1, 6, 0, 500)
            // )

        )
        .lmap(LayerMapScheme::new("v0_lm", LayerMapKind::Subcortical)
            // .layer_old(EXT_LYR, 1, LayerTags::DEFAULT,
            //     AxonDomain::output(&[map::THAL_SP, at0]),
            //     LayerKind::Axonal(AxonTopology::Spatial)
            // )
            .layer(LayerScheme::define(EXT_LYR)
                .depth(1)
                .axonal(AxonTopology::Spatial)
                .axon_domain(AxonDomain::output(&[map::THAL_SP, at0]))
            )
        )
}


fn define_a_schemes() -> AreaSchemeList {
    AreaSchemeList::new()
        // .area(AreaScheme::new("v0", "v0_lm", ENCODE_DIM)
        //     .subcortex())
        .area(AreaScheme::irregular("v0", "v0_lm", ENCODE_DIMS)
            .subcortex())

        .area(AreaScheme::new(PRI_AREA, "visual", AREA_DIM)
            .eff_areas(vec!["v0"]) )
}

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
