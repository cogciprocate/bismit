use std::ops::Range;
use std::io::{self, Write};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::str::{FromStr};
use time::{self, Timespec, Duration};

use ::{Cortex, OclEvent, LayerMapSchemeList, AreaSchemeList, CorticalAreaSettings};
use ::map::SliceTractMap;
// use config;

const INITIAL_TEST_ITERATIONS: u32 = 1;
const STATUS_EVERY: u32 = 5000;
const PRINT_DETAILS_EVERY: u32 = 10000;
const GUI_CONTROL: bool = true;
const PRINT_AFF_OUT: bool = false;


/// Cycle control commands.
#[derive(Clone, Debug)]
pub enum CyCtl {
    None,
    Iterate(u32),
    Sample(Range<u8>, Arc<Mutex<Vec<u8>>>),
    RequestCurrentAreaInfo,
    RequestCurrentIter,
    // ViewAllSlices(bool),
    // ViewBufferDebug(bool),
    Stop,
    Exit,
}


/// Cycle result responses.
///
/// Information about the cycling of the things and the stuff (and some of the
/// non-stuff too... but not that much of it really... well... a fair
/// amount... but not like a ton).
#[derive(Clone, Debug)]
pub enum CyRes {
    // None,
    CurrentIter(u32),
    Status(Box<Status>),
    AreaInfo(Box<AreaInfo>),
    // OtherShit(SliceTractMap),
}


#[derive(Clone, Debug)]
pub struct AreaInfo {
    pub name: String,
    pub aff_out_slc_range: Range<u8>,
    pub tract_map: SliceTractMap,
}


/// Cycle status.
#[derive(Clone, Debug)]
pub struct Status {
    // pub dims: (u32, u32),
    pub cur_cycle: u32,
    pub prev_cycles: u32,
    // pub cur_elapsed: Duration,
    pub prev_elapsed: Duration,
    pub cur_start_time: Option<Timespec>,
}

impl Status {
    pub fn new(/*dims: (u32, u32)*/) -> Status {
        Status {
            // dims: dims,
            cur_cycle: 0,
            prev_cycles: 0,
            // cur_elapsed: Duration::seconds(0),
            prev_elapsed: Duration::seconds(0),
            cur_start_time: Some(time::get_time()),
        }
    }

    pub fn cur_cycle(&self) -> u32 {
        self.cur_cycle
    }

    pub fn cur_elapsed(&self) -> Duration {
        match self.cur_start_time {
            Some(start) => time::get_time() - start,
            None => Duration::zero(),
        }
    }

    pub fn cur_cps(&self) -> f32 {
        match self.cur_start_time {
            Some(_) => Status::cps(self.cur_cycle, self.cur_elapsed()),
            None => 0.0,
        }
    }


    pub fn ttl_cycles(&self) -> u32 {
        self.cur_cycle + self.prev_cycles
    }

    pub fn ttl_elapsed(&self) -> Duration {
        self.prev_elapsed + self.cur_elapsed()
    }

    pub fn ttl_cps(&self) -> f32 {
        Status::cps(self.ttl_cycles(), self.ttl_elapsed())
    }

    fn cps(cycle: u32, elapsed: Duration) -> f32 {
        if elapsed.num_milliseconds() > 0 {
            (cycle as f32 / elapsed.num_milliseconds() as f32) * 1000.0
        } else {
            0.0
        }
    }
}


struct RunInfo {
    cortex: Cortex,
    cycle_iters: u32,
    bypass_act: bool,
    autorun_iters: u32,
    first_run: bool,
    view_all_axons: bool,
    view_sdr_only: bool,
    area_name: String,
    status: Status,
    // loop_start_time: Timespec,
}


pub enum LoopAction {
    None,
    Break,
    Continue,
}


pub struct CycleLoop;

impl CycleLoop {
    pub fn run(autorun_iters: u32, control_rx: Receiver<CyCtl>, mut result_tx: Sender<CyRes>,
                lm_schemes: LayerMapSchemeList, a_schemes: AreaSchemeList,
                ca_settings: Option<CorticalAreaSettings>) -> bool {
        let cortex = Cortex::new(lm_schemes, a_schemes, ca_settings);
        // config::disable_stuff(&mut cortex);

        let area_name = "v1".to_owned();

        // let area_dims = {
        //     let dims = cortex.area(&area_name).dims();
        //     (dims.v_size(), dims.u_size())
        // };

        let mut ri = RunInfo {
            cortex: cortex,
            cycle_iters: if autorun_iters > 0 {
                    autorun_iters
                } else {
                    INITIAL_TEST_ITERATIONS
                },
            bypass_act: false,
            autorun_iters: autorun_iters,
            first_run: true,
            view_all_axons: false,
            view_sdr_only: true,
            area_name: area_name,
            status: Status::new(),
            // loop_start_time: time::get_time(),
        };

        // result_tx.send(CyRes::Status(ri.status.clone())).expect("Error sending initial status.");

        loop {
            if GUI_CONTROL {
                match control_rx.recv() {
                    Ok(cyctl) => match cyctl {
                        CyCtl::Iterate(i) => ri.cycle_iters = i,
                        CyCtl::Exit => break,
                        CyCtl::Sample(range, buf) => {
                            refresh_hex_grid_buf(&ri, range, buf);
                            continue;
                        },
                        CyCtl::RequestCurrentAreaInfo => {
                            result_tx.send(CyRes::AreaInfo(Box::new(AreaInfo {
                                name: ri.area_name.to_string(),
                                aff_out_slc_range: ri.cortex.area(&ri.area_name).area_map().aff_out_slc_range(),
                                tract_map: ri.cortex.area(&ri.area_name).axn_tract_map(),
                            }))).expect("Error sending area info.");
                            continue;
                        },
                        _ => continue,
                    },

                    Err(e) => panic!("run(): control_rx.recv(): '{:?}'", e),
                }
            } else {
                match prompt(&mut ri) {
                    LoopAction::Continue => continue,
                    LoopAction::Break => break,
                    LoopAction::None => (),
                }
            }

            // ri.loop_start_time = time::get_time();
            ri.status.cur_start_time = Some(time::get_time());
            ri.status.cur_cycle = 0;
            // ri.status.cur_elapsed = Duration::seconds(0);

            // Send a Status with updated time:
            result_tx.send(CyRes::Status(Box::new(ri.status.clone()))).ok();

            if ri.cycle_iters > 1 {
                print!("Running {} iterations... \n", ri.cycle_iters);
            }

            match loop_cycles(&mut ri, &control_rx, &mut result_tx) {
                CyCtl::Exit => break,
                _ => (),
            }

            match cycle_print(&mut ri) {
                LoopAction::Continue => continue,
                LoopAction::Break => break,
                LoopAction::None => (),
            }

            ri.status.prev_cycles += ri.status.cur_cycle;
            ri.status.prev_elapsed = ri.status.prev_elapsed + ri.status.cur_elapsed();
            ri.status.cur_cycle = 0;
            ri.status.cur_start_time = None;
            // ri.status.cur_elapsed = Duration::seconds(0);
            // Send status with updated totals:
            result_tx.send(CyRes::Status(Box::new(ri.status.clone()))).ok();
        }

        println!("");

        true
    }
}


fn refresh_hex_grid_buf(ri: &RunInfo, slc_range: Range<u8>, buf: Arc<Mutex<Vec<u8>>>)
        -> Option<OclEvent>
{
    let axn_range = ri.cortex.area(&ri.area_name).axn_tract_map().axn_id_range(slc_range.clone());

    // match buf.try_lock() {
    match buf.lock() {
        // Ok(ref mut b) => ri.cortex.area(&ri.area_name).sample_aff_out(&mut b[range]),
        Ok(ref mut b) => Some(ri.cortex.area(&ri.area_name)
            .sample_axn_slc_range(slc_range, &mut b[axn_range])),
        Err(_) => None,
    }
}


fn loop_cycles(ri: &mut RunInfo, control_rx: &Receiver<CyCtl>, result_tx: &mut Sender<CyRes>)
        -> CyCtl
{
    if !ri.view_sdr_only { print!("\nRunning {} sense only loop(s) ... \n", ri.cycle_iters - 1); }

    loop {
        if ri.status.cur_cycle >= (ri.cycle_iters - 1) { break; }

        let elapsed = ri.status.cur_elapsed();

        if ri.status.cur_cycle % STATUS_EVERY == 0 || ri.status.cur_cycle == (ri.cycle_iters - 2) {
            if ri.status.cur_cycle > 0 || (ri.cycle_iters > 1 && ri.status.cur_cycle == 0) {
                print!("[{}: {:01}ms]", ri.status.cur_cycle, elapsed.num_milliseconds());
            }
            io::stdout().flush().ok();
        }

        if ri.status.cur_cycle % PRINT_DETAILS_EVERY == 0 {
            if !ri.view_sdr_only {
                // output_czar::print_sense_only(&mut ri.cortex, &ri.area_name);
                panic!("Currently disabled. Needs update.");
            }
        }

        if !ri.bypass_act {
             // TESTING `InputTract` SHIT:
            // match ri.cortex.external_tract_mut("v0".to_owned()) {
            //     Ok(ref mut input_tract) => {
            //         for i in 0..input_tract.frame().len() {
            //             let lo_h = ((i < input_tract.frame().len() / 2)
            //                     & (ri.status.cur_cycle % 2 == 0)) as u8
            //                 * 255;

            //             let hi_h = ((i >= input_tract.frame().len() / 2)
            //                     & !(ri.status.cur_cycle % 2 == 0)) as u8
            //                 * 255;

            //             input_tract[i] = lo_h + hi_h;

            //             if i < input_tract.frame().len() / 2 {
            //                 // input_tract[i] = 255;
            //                 // input_tract[i] = (ri.status.cur_cycle % 2 == 0) as u8 * 255;
            //                 if ri.status.cur_cycle % 2 == 0 {
            //                     assert!(input_tract[i] == 255);
            //                 } else {
            //                     // input_tract[i] = 0;
            //                     assert!(input_tract[i] == 0);
            //                 }
            //             } else {
            //                 // input_tract[i] = !(ri.status.cur_cycle % 2 == 0) as u8 * 255;
            //                 if ri.status.cur_cycle % 2 == 0 {
            //                     // input_tract[i] = 0;
            //                     assert!(input_tract[i] == 0);
            //                 } else {
            //                     // input_tract[i] = 255;
            //                     assert!(input_tract[i] == 255);
            //                 }
            //             }
            //         }
            //     },
            //     Err(_) => (),
            // }

            // // DEBUG?:
            // match ri.cortex.input_tract_mut("v0b".to_owned()) {
            //     Ok(ref mut input_tract) => {
            //         for tc in input_tract.iter_mut() {
            //             *tc = (ri.status.cur_cycle % 255) as u8;
            //         }
            //     },
            //     Err(_) => (),
            // }

            ri.cortex.cycle();
        }

        // Update current cycle:
        ri.status.cur_cycle += 1;

        // Respond to any requests:
        // Not sure why we're incrementing `cur_cycle` a second time.
        if let Ok(c) = control_rx.try_recv() {
            match c {
                CyCtl::RequestCurrentIter => result_tx.send(
                    CyRes::CurrentIter(ri.status.cur_cycle + 1)).unwrap(),
                // If a new sample has been requested, fulfill it:
                CyCtl::Sample(range, buf) => {
                    // println!("###### CycleLoop::run(): CANDIDATE 2 (RUNTIME): range: {:?}",
                    //     range);
                    refresh_hex_grid_buf(&ri, range, buf);
                },
                CyCtl::Stop => {
                    // println!("\nSTOP RECIEVED!\n");
                    return CyCtl::Stop;
                },
                // Otherwise return with the control code:
                _ => return c,
            }
        }

        // ri.status.cur_elapsed = elapsed;
        // result_tx.send(CyRes::Status(Box::new(ri.status.clone()))).ok();
    }

    CyCtl::None
}


fn cycle_print(ri: &mut RunInfo) -> LoopAction {
    if !ri.view_sdr_only { print!("\n\nRunning {} sense and print loop(s)...", 1usize); }

    if !ri.bypass_act {
        ri.cortex.cycle();
        ri.status.cur_cycle += 1;
    }

    if !ri.view_sdr_only {
        print!("\n\n=== Iteration {}/{} ===", ri.status.cur_cycle, ri.cycle_iters);

        if false {
            print!("\nSENSORY INPUT VECTOR:");
        }

        // output_czar::print_sense_and_print(&mut ri.cortex, &ri.area_name);
        panic!("Currently disabled. Needs update.");
    }

    if PRINT_AFF_OUT && !ri.view_all_axons {
        // if ri.view_sdr_only { ri.cortex.area_mut(&ri.area_name).psal_mut().dens.states.fill_vec(); }
        // ri.cortex.area_mut(&ri.area_name).axns.states.fill_vec();
        // print!("\n'{}' output:", &ri.area_name);
        // ri.cortex.area_mut(&ri.area_name).render_aff_out("", true);
        panic!("Currently disabled. Needs update.");;
    }

    if ri.view_all_axons {
        print!("\n\nAXON SPACE:\n");

        // ri.cortex.area_mut(&ri.area_name).render_axn_space();
        panic!("Currently disabled. Needs update.");;
    }

    if ri.status.cur_cycle > 1 {
        printlnc!(yellow: "-> {} cycles @ [> {:02.2} c/s <]",
            ri.status.cur_cycle, (ri.status.cur_cycle as f32
                / ri.status.cur_elapsed().num_milliseconds() as f32) * 1000.0);
    }

    if ri.cycle_iters > 1000 {
        ri.cycle_iters = 1;
    }

    if ri.autorun_iters > 0 {
        LoopAction::Break
    } else {
        LoopAction::None
    }
}


fn prompt(ri: &mut RunInfo) -> LoopAction {
    if ri.cycle_iters == 0 {
        ri.cycle_iters = 1;
        ri.bypass_act = true;
    } else {
        ri.bypass_act = false;
    }

    if ri.autorun_iters == 0 {
        let in_string: String = if ri.first_run {
            ri.first_run = false;
            "\n".to_string()
        } else {
            let axn_state = if ri.view_all_axons { "on" } else { "off" };
            let view_state = if ri.view_sdr_only { "sdr" } else { "all" };

            rin(format!("bismit: [{ttl_i}/({loop_i})]: [v]iew:[{}] [a]xons:[{}] \
                [m]otor:[X] a[r]ea:[{}] [t]ests [q]uit [i]ters:[{iters}]",
                view_state, axn_state, ri.area_name,
                iters = ri.cycle_iters,
                loop_i = 0, //input_czar.counter(),
                ttl_i = ri.status.prev_cycles,
            ))
        };


        if "q\n" == in_string {
            print!("\nExiting interactive test mode... ");
            return LoopAction::Break;
        } else if "i\n" == in_string {
            let in_s = rin(format!("Iterations: [i={}]", ri.cycle_iters));
            if "\n" == in_s {
                return LoopAction::Continue;
            } else {
                let in_int = parse_iters(&in_s);
                match in_int {
                    Ok(x)    => {
                         ri.cycle_iters = x;
                         return LoopAction::None;
                    },
                    Err(_) => {
                        print!("Invalid number.\n");
                        return LoopAction::Continue;
                    },
                }
            }

        } else if "r\n" == in_string {
            let in_str = rin(format!("area name"));
            let in_s1 = in_str.trim();
            let new_area_name = in_s1.to_string();

            if ri.cortex.valid_area(&new_area_name) {
                ri.area_name = new_area_name;
            } else {
                print!("Invalid area.");
            }
            ri.bypass_act = true;
            return LoopAction::None;

        } else if "v\n" == in_string {
            ri.view_sdr_only = !ri.view_sdr_only;
            ri.bypass_act = true;
            return LoopAction::None;

        } else if "\n" == in_string {
            return LoopAction::None;
            // DO NOT REMOVE

        } else if "a\n" == in_string {
            ri.view_all_axons = !ri.view_all_axons;
            ri.bypass_act = true;
            return LoopAction::None;

        } else if "t\n" == in_string {
            let in_s = rin(format!("tests: [f]ract [c]ycles [l]earning [a]ctivate a[r]ea_output o[u]tput"));

            if "p\n" == in_s {
                //synapse_drill_down::print_pyrs(&mut cortex);
                return LoopAction::Continue;

            } else if "u\n" == in_s {
                // let in_str = rin(format!("area name"));
                // let in_s1 = in_str.trim();
                // let out_len = cortex.area(&in_s).dims.columns();
                // let t_vec: Vec<u8> = iter::repeat(0).take(out_len as usize).collect();
                // cortex.area_mut(&in_s).read_output(&mut t_vec, map::FF_OUT);
                // ocl::fmt::print_vec_simple(&t_vec);
                println!("\n##### PRINTING TEMPORARILY DISABLED #####");
                return LoopAction::Continue;

            } else if "c\n" == in_s {
                println!("\n##### DISABLED #####");
                //hybrid::test_cycles(&mut cortex, &area_name);
                return LoopAction::Continue;

            } else if "l\n" == in_s {
                println!("\n##### DISABLED #####");
                //learning::test_learning_cell_range(&mut cortex, inhib_layer_name, &area_name);
                return LoopAction::Continue;

            } else if "a\n" == in_s {
                println!("\n##### DISABLED #####");
                //learning::test_learning_activation(&mut cortex, &area_name);
                return LoopAction::Continue;

            // } else if "f\n" == in_s {
            //     let in_s = rin(format!("fractal seed"));
            //     let in_int: Option<u8> = in_s.trim().parse().ok();

            //     // let seed = match in_int {
            //     //     Some(x)    => x,
            //     //     None => {
            //     //         print!("\nError parsing number.");
            //     //         continue;
            //     //     },
            //     // };

            //     let in_s = rin(format!("cardinality factor: 256 * "));
            //     let in_int: Option<usize> = in_s.trim().parse().ok();

            //     let c_factor = match in_int {
            //         Some(x)    => x,
            //         None => {
            //             print!("\nError parsing number.");
            //             continue;
            //         },
            //     };

            //     // let tvec = cmn::gen_fract_sdr(seed, 256 * c_factor);
            //     // ocl::fmt::print_vec_simple(&tvec[..]);
            //     println!("\n##### PRINTING TEMPORARILY DISABLED #####");
            //     continue;

            // } else if "r\n" == in_s {
            //     let in_str = rin(format!("area name"));
            //     // let in_s = in_str.trim();
            //     //let in_int: Option<u8> = in_s.trim().parse().ok();

            //     println!("\n##### DISABLED #####");
            //     //cortex.print_area_output(&in_s);
            //     continue;

            } else {
                return LoopAction::Continue;
            }


        } else if "m\n" == in_string {
            // bypass_act = true;
            let in_s = rin(format!("motor: [s]witch(disconnected)"));
            if "s\n" == in_s {
                //input_czar.motor_state.switch();
                //println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
                return LoopAction::Continue;
                //cycle_iters = TEST_ITERATIONS;

            } else {
                return LoopAction::Continue;
            }
        } else {
            return LoopAction::Continue;
        }
    }

    LoopAction::None
}


pub fn parse_iters(in_s: &str) -> Result<u32, <u32 as FromStr>::Err> {
    in_s.trim().replace("k","000").replace("m","000000").parse()
}


pub fn rin(prompt: String) -> String {
    let mut in_string: String = String::new();
    print!("\n{}:> ", prompt);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut in_string).ok().expect("Failed to read line");
    in_string.to_lowercase()
}
