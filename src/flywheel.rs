//! Flywheel
//!
//!
//!
//
// * TODO:
// - Optional command line printing (and possibly a menu here instead of in vibi).
//
//
//


use std::num::Wrapping;
// use std::ops::Range;
use std::sync::mpsc::{Sender, SyncSender, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use time::{self, Timespec, Duration};
// use ocl::Buffer;
use cmn::{CmnResult};
use ::{Cortex, OclEvent, LayerMapSchemeList, AreaSchemeList, CorticalAreaSettings, SamplerKind,
    SamplerBufferKind, TractReceiver};
use ::map::{SliceTractMap, /*LayerAddress*/};



#[derive(Clone, Debug)]
pub enum PathwayConfig {
    // EncoderRanges(Arc<Mutex<Vec<(f32, f32)>>>),
    EncoderRanges(Vec<(f32, f32)>),
}


#[derive(Clone, Debug)]
pub enum Obs {
    Float64 { p: usize, len: usize }
}


// [NOTE]: Remove this or re-use it to be the one-time designator for an
// arc-mutex. Really no need to have a separate sensory frame unless sensory
// data is highly sporadic.
//
#[derive(Clone, Debug)]
pub enum SensoryFrame {
    F32Array16([f32; 16]),
    // TODO: Convert this into a `usize` referring to a previously stored
    // arc-mutex reference avoiding the need to create a new reference for
    // each frame (OR REDESIGN - SEE ABOVE):
    Tract(Arc<Mutex<Vec<u8>>>),
    PathwayConfig(PathwayConfig),
}


#[derive(Clone, Debug)]
pub enum MotorFrame {
    Action,
}


#[derive(Clone, Debug)]
pub struct AreaInfo {
    pub name: String,
    // pub aff_out_slc_range: Range<usize>,
    pub aff_out_slc_ids: Vec<u8>,
    pub tract_map: SliceTractMap,
}


/// Imperative cycle control commands.
#[derive(Clone, Debug)]
pub enum Command {
    None,
    Iterate(u32),
    Stop,
    Exit,
}


// Requests for and submissions of data.
#[derive(Clone, Debug)]
pub enum Request {
    CurrentIter,
    Status,
    AreaInfo,
    // Sample(Range<usize>, Arc<Mutex<Vec<u8>>>),
    Sampler { area_name: String, kind: SamplerKind, buffer_kind: SamplerBufferKind },
    FinishQueues,
    // Input(Obs),
    // GetAction,
}


/// Cycle result responses.
#[derive(Debug)]
pub enum Response {
    CycleStarted(u32),
    CurrentIter(u32),
    Status(Box<Status>),
    Ready,
    Motor(MotorFrame),
    AreaInfo(Box<AreaInfo>),
    SampleProgress(Option<OclEvent>),
    QueuesFinished(u64),
    Sampler(TractReceiver),
    Exiting,
}


/// Cycle status.
#[derive(Clone, Debug)]
pub struct Status {
    pub cycling: bool,
    pub cur_cycle: Wrapping<u32>,
    pub prev_cycles: Wrapping<u32>,
    pub prev_elapsed: Duration,
    pub cur_start_time: Option<Timespec>,
    pub cycle_counter: Wrapping<u64>,
}

impl Status {
    pub fn new() -> Status {
        Status {
            cycling: false,
            cur_cycle: Wrapping(0),
            prev_cycles: Wrapping(0),
            prev_elapsed: Duration::seconds(0),
            cur_start_time: Some(time::get_time()),
            cycle_counter: Wrapping(0),
        }
    }

    pub fn cur_cycle(&self) -> u32 {
        self.cur_cycle.0
    }

    pub fn cur_elapsed(&self) -> Duration {
        match self.cur_start_time {
            Some(start) => time::get_time() - start,
            None => Duration::zero(),
        }
    }

    pub fn cur_cps(&self) -> f32 {
        match self.cur_start_time {
            Some(_) => Status::cps(self.cur_cycle.0, self.cur_elapsed()),
            None => 0.0,
        }
    }


    pub fn ttl_cycles(&self) -> u32 {
        self.cur_cycle.0 + self.prev_cycles.0
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


pub enum LoopAction {
    None,
    Break,
    Continue,
}


/// An event loop for the cortex.
///
// Currently clocking in at ~488 bytes (2017-Apr-10)
pub struct Flywheel {
    command_rx: Receiver<Command>,
    req_res_pairs: Vec<(Receiver<Request>, Sender<Response>)>,
    // sen_mot_pairs: Vec<(Receiver<SensoryFrame>, Sender<MotorFrame>)>,
    sensory_rxs: Vec<(Receiver<SensoryFrame>, usize)>,
    motor_txs: Vec<SyncSender<MotorFrame>>,
    cortex: Cortex,
    cycle_iters_max: u32,
    area_name: String,
    status: Status,
    exiting: bool,
}

impl Flywheel {
    // TODO: Find some other way to set current area.
    pub fn new<S: Into<String>>(cortex: Cortex, command_rx: Receiver<Command>, area_name: S) -> Flywheel {
        // let area_name = "v1".to_owned();

        Flywheel {
            command_rx: command_rx,
            req_res_pairs: Vec::with_capacity(16),
            sensory_rxs: Vec::with_capacity(8),
            motor_txs: Vec::with_capacity(8),
            cortex: cortex,
            cycle_iters_max: 1,
            area_name: area_name.into(),
            status: Status::new(),
            exiting: false,
        }
    }

    pub fn from_blueprint<S: Into<String>>(
                lm_schemes: LayerMapSchemeList,
                a_schemes: AreaSchemeList,
                ca_settings: Option<CorticalAreaSettings>,
                command_rx: Receiver<Command>,
                area_name: S,
            ) -> Flywheel {
        let cortex = Cortex::new(lm_schemes, a_schemes, ca_settings);

        Flywheel::new(cortex, command_rx, area_name.into())
    }

    pub fn add_req_res_pair(&mut self, req_rx: Receiver<Request>, res_tx: Sender<Response>) {
        self.req_res_pairs.push((req_rx, res_tx));
    }

    // pub fn add_sen_mot_pair(&mut self, sen_rx: Receiver<SensoryFrame>, mot_tx: Sender<MotorFrame>) {
    //     self.sen_mot_pairs.push((sen_rx, mot_tx));
    // }

    pub fn add_sensory_rx<S: AsRef<str>>(&mut self, sensory_rx: Receiver<SensoryFrame>,
            pathway_name: S)
    {
        let pathway_idx = self.cortex.thal_mut().ext_pathway_idx(pathway_name.as_ref()).unwrap();
        self.sensory_rxs.push((sensory_rx, pathway_idx));
    }

    pub fn add_motor_tx(&mut self, motor_tx: SyncSender<MotorFrame>) {
        self.motor_txs.push(motor_tx);
    }

    pub fn cortex(&self) -> &Cortex {
        &self.cortex
    }

    pub fn cortex_mut(&mut self) -> &mut Cortex {
        &mut self.cortex
    }

    pub fn spin(&mut self) {
        loop {
            if self.exiting { break; }
            self.intake_sensory_frames().unwrap();
            self.fulfill_requests();

            // // DEBUG:
            // println!("Waiting on command...");

            match self.command_rx.recv() {
                Ok(cmd) => match cmd {
                    Command::Iterate(i) => self.cycle_iters_max = i,
                    // Command::Spin => self.cycle_iters_max = 0,
                    Command::Exit => {
                        self.exiting = true;
                        break;
                    },
                    _ => continue,
                },
                Err(e) => panic!("{}", e),
            }

            self.status.cur_cycle = Wrapping(0);
            self.status.cur_start_time = Some(time::get_time());
            self.status.cycling = true;
            self.broadcast_status();

            // // DEBUG:
            // println!("Starting cycle loop with {} iters...", self.cycle_iters_max);

            // Run primary loop and check for exit response:
            match self.cycle_loop() {
                Command::Exit => {
                    self.exiting = true;
                    break;
                },
                _ => (),
            }

            // ////// DEBUG:
            // println!(">>>>>> Cycle loop complete. Status: {:?}", self.status);

            self.status.cycling = false;
            self.status.prev_cycles += self.status.cur_cycle;
            self.status.prev_elapsed = self.status.prev_elapsed + self.status.cur_elapsed();
            self.status.cur_cycle = Wrapping(0);
            self.status.cur_start_time = None;
            self.broadcast_status();

            // // DEBUG:
            // println!("{} cycle loops (prev: {}) complete...", self.status.cur_cycle,
            //     self.status.prev_cycles);

            // // DEBUG:
            // println!("Cycle loop complete, prev_cycles: {}...", self.status.prev_cycles);
        }

        // Broadcast an `Exiting` to everyone.
        for &(_, ref res_tx) in self.req_res_pairs.iter() {
            // Handle this?
            res_tx.send(Response::Exiting).ok();
        }
    }

    fn cycle_loop(&mut self) -> Command {
        // // DEBUG:
        // println!("Cycle loop started...");

        loop {
            if (self.cycle_iters_max != 0) && (self.status.cur_cycle.0 >= self.cycle_iters_max) { break; }

            self.intake_sensory_frames().unwrap();

            self.cortex.cycle();

            // Update cycle_counts:
            self.status.cur_cycle += Wrapping(1);
            self.status.cycle_counter += Wrapping(1);

            // // DEBUG:
            // println!(">>>>>> Flywheel::cycle_loop: self.status: {:?}", self.status);

            // Respond to any commands:
            match self.command_rx.try_recv() {
                Ok(c) => match c {
                    Command::None => (),
                    Command::Stop => return Command::Stop,
                    _ => return c,
                },
                Err(e) => match e {
                    TryRecvError::Empty => (),
                    TryRecvError::Disconnected => panic!("Flywheel::cycle_loop(): \
                        Sender disconnected."),
                },
            }

            self.output_motor_frames();

            // Process pending requests:
            self.fulfill_requests();
        }

        Command::None
    }

    fn fulfill_requests(&mut self) {
        // ////// DEBUG:
        // println!("Fulfilling requests...");

        for &(ref req_rx, ref res_tx) in self.req_res_pairs.iter() {
            loop {
                match req_rx.try_recv() {
                    Ok(r) => {
                        // ////// DEBUG:
                        // println!("Fullfilling request: {:?}", r);
                        match r {
                            // Request::Sample(range, buf) => {
                            //     res_tx.send(Response::SampleProgress(self.refresh_buf(range, buf))).unwrap();
                            // },
                            Request::Sampler { area_name, kind, buffer_kind } => {
                                let tract_rx = self.cortex.areas_mut().by_key_mut(area_name.as_str()).unwrap()
                                    .sampler(kind, buffer_kind);
                                res_tx.send(Response::Sampler(tract_rx)).unwrap();
                            },
                            Request::AreaInfo => {
                                self.send_area_info(res_tx);
                            },
                            Request::Status => {
                                res_tx.send(Response::Status(Box::new(self.status.clone()))).unwrap();
                            }
                            Request::CurrentIter => {
                                res_tx.send(Response::CurrentIter(self.status.cur_cycle.0)).unwrap();
                            },
                            Request::FinishQueues => {
                                // Will block:
                                self.cortex.finish_queues();
                                // ////// DEBUG:
                                // println!("Queues have been finished. Sending 'Response::QueuesFinished'...");
                                match res_tx.send(Response::QueuesFinished(self.status.cycle_counter.0)) {
                                    Ok(_) => (),
                                    Err(err) => if !self.exiting { panic!("{:?}", err); }
                                }
                            },
                        }
                    }
                    Err(e) => match e {
                        TryRecvError::Empty => break,
                        // TODO: Have this either do nothing or check to see
                        // if any senders remain and exit if 0.
                        // TryRecvError::Disconnected => panic!("Flywheel::fulfill_requests(): \
                        //     Sender disconnected."),
                        TryRecvError::Disconnected => (),
                    },
                }
            }
        }
    }

    // [NOTE]: Incoming array values beyond the length of destination slice will
    // be silently ignored.
    fn intake_sensory_frames(&mut self) -> CmnResult<()> {
        // // DEBUG:
        // println!("Intaking sensory frames...");

        for &(ref sen_rx, pathway_idx) in self.sensory_rxs.iter() {
            loop {
                match sen_rx.try_recv() {
                    Ok(s) => {
                        match s {
                            SensoryFrame::F32Array16(_arr) => {
                                // println!("Intaking sensory frame [pathway id: {}]: {:?} ...",
                                //     pathway_idx, arr);

                                // // let pathway = match try!(self.cortex.thal_mut().ext_pathway_frame(pathway_idx)) {
                                // let pathway = match self.cortex.thal_mut().ext_pathway(pathway_idx)? {
                                //     InputGeneratorFrame::F32Slice(s) => s,
                                //     f @ _ => panic!(format!("Flywheel::intake_sensory_frames(): Unsupported \
                                //         InputGeneratorFrame variant: {:?}", f)),
                                // };

                                // for (i, dst) in pathway.iter_mut().enumerate() {
                                //     *dst = arr[i];
                                // }
                                unimplemented!();
                            },
                            SensoryFrame::PathwayConfig(pc) => match pc {
                                PathwayConfig::EncoderRanges(ranges) => {
                                    // match try!(self.cortex.thal_mut().ext_pathway(pathway_idx)).encoder() {
                                    //     &mut InputGeneratorEncoder::VectorEncoder(ref mut v) => {
                                    //         try!(v.set_ranges(&ranges.lock().unwrap()[..]));
                                    //     }
                                    //     _ => unimplemented!(),
                                    // }

                                    self.cortex.thal_mut().ext_pathway(pathway_idx)?
                                        .set_encoder_ranges(ranges);
                                }
                            },
                            SensoryFrame::Tract(_) => unimplemented!(),
                        }
                    }
                    Err(e) => match e {
                        TryRecvError::Empty => break,
                        TryRecvError::Disconnected => panic!("Flywheel::fulfill_io(): \
                            Sensory sender disconnected."),
                    },
                }
            }
        }

        Ok(())
    }

    fn output_motor_frames(&self) {
        // for ref mot_tx in self.motor_txs.iter() {
        //     match mot_tx.try_recv() {
        //         Ok(r) => {
        //             match r {

        //             }
        //         }
        //         Err(e) => match e {
        //             TryRecvError::Empty => (),
        //             TryRecvError::Disconnected => panic!("Flywheel::fulfill_io(): \
        //                 Sender disconnected."),
        //         },
        //     }
        // }
    }

    fn broadcast_status(&self) {
        // println!(">>>>>> Broadcasting status...");
        for &(_, ref res_tx) in self.req_res_pairs.iter() {
            // TODO: Remove unnecessary (redundant) heap allocation:
            res_tx.send(Response::Status(Box::new(self.status.clone()))).unwrap();
        }
    }

    // #[deprecated]
    // fn refresh_buf(&self, slc_range: Range<usize>, buf: Arc<Mutex<Vec<u8>>>)
    //             -> Option<OclEvent> {
    //     // // DEBUG:
    //     // println!("Refreshing buffer...");

    //     let axn_range = self.cortex.areas().by_key(self.area_name.as_str()).unwrap()
    //         .axn_tract_map().axn_id_range(slc_range.clone());

    //     // match buf.try_lock() {
    //     match buf.lock() {
    //         Ok(ref mut b) => Some(self.cortex.areas().by_key(self.area_name.as_str())
    //             .unwrap().sample_axn_slc_range(slc_range, &mut b[axn_range])),
    //         Err(_) => None,
    //     }
    // }

    fn send_area_info(&self, res_tx: &Sender<Response>) {
        res_tx.send(Response::AreaInfo(Box::new(
            AreaInfo {
                name: self.area_name.to_string(),
                // aff_out_slc_range: self.cortex.areas().by_key(self.area_name.as_str())
                //     .unwrap().area_map().aff_out_slc_range(),
                aff_out_slc_ids: self.cortex.areas().by_key(self.area_name.as_str())
                    .unwrap().area_map().aff_out_slc_ids(),
                tract_map: self.cortex.areas().by_key(self.area_name.as_str())
                    .unwrap().axn_tract_map(),
            }
        ))).expect("Error sending area info.")
    }
}

// impl Drop for Flywheel {
//     fn drop(&mut self) {
//         for &(_, ref res_tx) in self.req_res_pairs.iter() {
//             res_tx.send(Response::Exiting).unwrap();
//         }
//     }
// }

