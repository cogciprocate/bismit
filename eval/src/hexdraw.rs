
use std::thread;
use vibi::bismit::cmn;
use vibi::bismit::map::*;
use vibi::bismit::ocl::{ WriteGuard};
use vibi::bismit::{map, Cortex, CorticalAreaSettings};
use vibi::bismit::flywheel::{Command, Request, Response};
use ::{Controls, Params};


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 64;
const AREA_DIM: u32 = 16;
const HEX_GRP_RADIUS: usize = 6;


pub fn draw(params: &Params, controls: &Controls) {
    // Populates the thing.
    fn pop(offset: [i32; 2], val: u8, guard: &mut [u8]) {
        let dims = [ENCODE_DIM as i32, ENCODE_DIM as i32];
        let start = [(dims[0] / 2) + offset[0], (dims[1] / 2) + offset[1]];
        cmn::populate_hex_tile_grps(HEX_GRP_RADIUS, dims, start, val, guard);
    }

    // Write to tract:
    debug!("Locking tract buffer...");
    let mut guard = params.tract_buffer.clone().write().wait().unwrap();
    // assert!(HEX_GRP_RADIUS % 2 == 0);
    let ofs_dist = ((HEX_GRP_RADIUS + 1) / 2) as i32;

    // let ofs_dist = HEX_GRP_RADIUS as i32 - 1;

    // pop([0, 0], 255, guard.as_mut_slice());

    pop([0, ofs_dist], 1, guard.as_mut_slice());
    pop([-ofs_dist, 0], 102, guard.as_mut_slice());
    pop([ofs_dist, -ofs_dist], 204, guard.as_mut_slice());

    pop([-ofs_dist, ofs_dist], 51, guard.as_mut_slice());
    pop([0, -ofs_dist], 153, guard.as_mut_slice());
    pop([ofs_dist, 0], 255, guard.as_mut_slice());

    WriteGuard::release(guard);

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

    // // Exit:
    // controls.cmd_tx.send(Command::Exit).unwrap();
    // controls.cmd_tx.send(Command::None).unwrap();
    // println!("Drawing complete.\n");
}


/// Draws an arbitrary pattern as an sdr.
pub fn eval() {
    let cortex = Cortex::new(define_lm_schemes(), define_a_schemes(), Some(ca_settings()));

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
    let in_tract_buffer = cortex.thal().tract().buffer_rwvec(in_tract_idx).unwrap().clone();
    let axns = cortex.areas().by_key(PRI_AREA).unwrap().axns().states().clone();
    let area_map = cortex.areas().by_key(PRI_AREA).unwrap().area_map().clone();

    let controls = ::spawn_threads(cortex, PRI_AREA);

    let params = Params { tract_buffer: in_tract_buffer, axns,
        l4_axns: v1_spt_lyr_buf, area_map, encode_dim: ENCODE_DIM, area_dim: AREA_DIM };

    // Get the flywheel moving:
    controls.cmd_tx.send(Command::None).unwrap();

    // Sleep until vibi window opens (need a better mech. for this):
    thread::sleep(::std::time::Duration::new(1, 0));

    // Draw (only need to draw 1):
    for _ in 0..5000 {
        draw(&params, &controls);
    }

    ::join_threads(controls);

}


fn define_lm_schemes() -> LayerMapSchemeList {
    let at0 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            .input_layer("aff_in", LayerTags::DEFAULT,
                AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]),
                AxonTopology::Spatial )
            // .layer("mcols", 1, LayerTags::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
            //     CellScheme::minicolumn(SPT_LYR, "iii", 9999) )
            .layer(SPT_LYR, 1, LayerTags::PSAL, AxonDomain::Local,
                CellScheme::spiny_stellate(&[("aff_in", 4, 1)], 7, 600) )
            .layer("iv_inhib", 0, LayerTags::DEFAULT, AxonDomain::Local, CellScheme::inhib(SPT_LYR, 4, 0))
            .layer("iii", 1, LayerTags::PTAL, AxonDomain::Local,
                CellScheme::pyramidal(&[("iii", 20, 1)], 1, 6, 500) ) )
        .lmap(LayerMapScheme::new("v0_lm", LayerMapKind::Subcortical)
            .layer(EXT_LYR, 1, LayerTags::DEFAULT,
                AxonDomain::output(&[map::THAL_SP, at0]),
                LayerKind::Axonal(AxonTopology::Spatial)) )
}


fn define_a_schemes() -> AreaSchemeList {
    AreaSchemeList::new()
        .area(AreaScheme::new("v0", "v0_lm", ENCODE_DIM)
            .subcortex() )
        .area(AreaScheme::new(PRI_AREA, "visual", AREA_DIM)
            .eff_areas(vec!["v0"]) )
}

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
