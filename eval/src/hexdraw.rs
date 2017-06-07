
use std::thread;
use std::sync::mpsc::{self, /*Sender, Receiver, TryRecvError*/};
// use rand;
// use rand::distributions::{Range, IndependentSample};
use vibi::window;
use vibi::bismit::cmn;
use vibi::bismit::map::*;
use vibi::bismit::flywheel::Flywheel;
use vibi::bismit::ocl::{/*Buffer, RwVec,*/ WriteGuard};
use vibi::bismit::{map, Cortex, /*Thalamus, SubcorticalNucleus,*/ CorticalAreaSettings, /*Subcortex*/};
use vibi::bismit::flywheel::{Command, Request, Response};
// use vibi::bismit::map::{AxonDomainRoute, AreaMap};
// use vibi::bismit::encode::{self, ScalarSdrWriter};
use spatial::Params;


static PRI_AREA: &'static str = "v1";
static IN_AREA: &'static str = "v0";
static EXT_LYR: &'static str = "external_0";
static SPT_LYR: &'static str = "iv";

const ENCODE_DIM: u32 = 64;
const AREA_DIM: u32 = 16;
const HEX_GRP_SIDE: usize = 4;


pub fn draw(params: &Params) {
    // Populates the thing.
    fn pop(offset: [i32; 2], val: u8, guard: &mut [u8]) {
        let dims = [ENCODE_DIM as i32, ENCODE_DIM as i32];
        let start = [(dims[0] / 2) + offset[0], (dims[1] / 2) + offset[1]];
        cmn::populate_hex_tile_grps(HEX_GRP_SIDE, dims, start, val, guard);
    }

    // Write to tract:
    debug!("Locking tract buffer...");
    let mut guard = params.tract_buffer.clone().write().wait().unwrap();
    // assert!(HEX_GRP_SIDE % 2 == 0);
    let ofs_dist = (HEX_GRP_SIDE / 2) as i32;

    // pop([0, 0], 255, guard.as_mut_slice());

    pop([0, ofs_dist], 1, guard.as_mut_slice());
    pop([-ofs_dist, 0], 102, guard.as_mut_slice());
    pop([ofs_dist, -ofs_dist], 204, guard.as_mut_slice());

    pop([-ofs_dist, ofs_dist], 51, guard.as_mut_slice());
    pop([0, -ofs_dist], 153, guard.as_mut_slice());
    pop([ofs_dist, 0], 255, guard.as_mut_slice());

    WriteGuard::release(guard);

    // Cycle and finish queues:
    params.cmd_tx.send(Command::Iterate(1)).unwrap();
    params.req_tx.send(Request::FinishQueues(0)).unwrap();
    params.cmd_tx.send(Command::None).unwrap();

    // Wait for completion.
    loop {
        debug!("Attempting to receive...");
        match params.res_rx.recv() {
            Ok(res) => match res {
                Response::Status(status) => {
                    debug!("Status: {:?}", status);
                    // if status.prev_cycles == 0 {
                        // params.req_tx.send(Request::FinishQueues(0)).unwrap();
                        // params.cmd_tx.send(Command::None).unwrap();
                    // }
                },
                Response::QueuesFinished(id) => {
                    if id == 0 {
                        debug!("Queues finished (id: {})", id);
                        // cycle_count = cycle_count.wrapping_add(1);
                        break;
                    }
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
    // params.cmd_tx.send(Command::Exit).unwrap();
    // params.cmd_tx.send(Command::None).unwrap();
    // println!("Drawing complete.\n");
}


/// Draws an arbitrary pattern as an sdr.
pub fn eval() {
    let (command_tx, command_rx) = mpsc::channel();
    let (vibi_request_tx, vibi_request_rx) = mpsc::channel();
    let (vibi_response_tx, vibi_response_rx) = mpsc::channel();
    let vibi_command_tx = command_tx.clone();

    let (spatial_request_tx, spatial_request_rx) = mpsc::channel();
    let (spatial_response_tx, spatial_response_rx) = mpsc::channel();
    let spatial_command_tx = command_tx;

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
    let in_tract_buffer = cortex.thal().tract().buffer(in_tract_idx).unwrap().clone();
    let axns = cortex.areas().by_key(PRI_AREA).unwrap().axns().states().clone();
    let area_map = cortex.areas().by_key(PRI_AREA).unwrap().area_map().clone();

    let mut flywheel = Flywheel::new(cortex, command_rx, PRI_AREA);
    flywheel.add_req_res_pair(vibi_request_rx, vibi_response_tx);
    flywheel.add_req_res_pair(spatial_request_rx, spatial_response_tx);

    // Flywheel thread:
    let th_flywheel = thread::Builder::new().name("flywheel".to_string()).spawn(move || {
        flywheel.spin();
    }).expect("Error creating 'flywheel' thread");

    // Vibi thread:
    let th_win = thread::Builder::new().name("win".to_string()).spawn(move || {
        println!("Opening vibi window...");
        window::Window::open(vibi_command_tx, vibi_request_tx, vibi_response_rx);
    }).expect("Error creating 'win' thread");

    let params = Params { cmd_tx: spatial_command_tx, req_tx: spatial_request_tx,
        res_rx: spatial_response_rx, tract_buffer: in_tract_buffer, axns,
        l4_axns: v1_spt_lyr_buf, area_map, encode_dim: ENCODE_DIM, area_dim: AREA_DIM };

    // Get the flywheel moving:
    params.cmd_tx.send(Command::None).unwrap();

    // Sleep until vibi window opens (need a better mech. for this):
    thread::sleep(::std::time::Duration::new(1, 0));

    // Draw (only need to draw 1):
    for _ in 0..5000 {
        draw(&params);
    }

    if let Err(e) = th_win.join() { println!("th_win.join(): Error: '{:?}'", e); }
    println!("Vibi window closed.");
    if let Err(e) = th_flywheel.join() { println!("th_flywheel.join(): Error: '{:?}'", e); }
    println!("Flywheel stopped.");
}


fn define_lm_schemes() -> LayerMapSchemeList {
    let at0 = AxonTag::unique();

    LayerMapSchemeList::new()
        .lmap(LayerMapScheme::new("visual", LayerMapKind::Cortical)
            .input_layer("aff_in", map::DEFAULT,
                AxonDomain::input(&[(InputTrack::Afferent, &[map::THAL_SP, at0])]),
                AxonTopology::Spatial )
            .layer("mcols", 1, map::DEFAULT, AxonDomain::output(&[map::THAL_SP]),
                CellScheme::minicolumn("iv", "iii", 9999) )
            .layer(SPT_LYR, 1, map::PSAL, AxonDomain::Local,
                CellScheme::spiny_stellate(&[("aff_in", 4, 1)], 7, 600) )
            .layer("iv_inhib", 0, map::DEFAULT, AxonDomain::Local, CellScheme::inhib("iv", 4, 0))
            .layer("iii", 1, map::PTAL, AxonDomain::Local,
                CellScheme::pyramidal(&[("iii", 20, 1)], 1, 6, 500) ) )
        .lmap(LayerMapScheme::new("v0_lm", LayerMapKind::Subcortical)
            .layer(EXT_LYR, 1, map::DEFAULT,
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
