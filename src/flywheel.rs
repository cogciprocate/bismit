//! Flywheel
//!
//!
//!
//
// The flywheel is crusty as hell and needs to be totally overhauled...
//
// * TODO:
// - Completely redesign.
// - Optional command line printing (and possibly a menu here instead of in vibi).
//
//
//
//
//


use std::num::Wrapping;
use std::sync::mpsc::{Sender, SyncSender, Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use time::{self, Timespec, Duration};
// use ocl::Buffer;
use cmn::{CmnResult};
use ::{Cortex, OclEvent, SamplerKind, SamplerBufferKind, TractReceiver};
use ::map::{SliceTractMap};



#[derive(Clone, Debug)]
pub enum PathwayConfig {
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
    Sampler { area_name: String, kind: SamplerKind, buffer_kind: SamplerBufferKind, backpressure: bool },
    FinishQueues,
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
pub struct Flywheel {
    command_rx: Receiver<Command>,
    req_res_pairs: Vec<(Receiver<Request>, Sender<Response>)>,
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

    pub fn add_req_res_pair(&mut self, req_rx: Receiver<Request>, res_tx: Sender<Response>) {
        self.req_res_pairs.push((req_rx, res_tx));
    }



    #[deprecated]
    pub fn add_sensory_rx<S: AsRef<str>>(&mut self, _sensory_rx: Receiver<SensoryFrame>,
            _pathway_name: S)
    {
        // let pathway_idx = self.cortex.thal_mut().input_generator_idx(pathway_name.as_ref()).unwrap();
        // self.sensory_rxs.push((sensory_rx, pathway_idx));
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

            // println!("Flywheel::spin: Receiving...");
            match self.command_rx.recv() {
                Ok(cmd) => match cmd {
                    Command::Iterate(i) => self.cycle_iters_max = i,
                    Command::Exit => {
                        self.exiting = true;
                        // println!("############# BREAKING");
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

            // Run primary loop and check for exit response:
            match self.cycle_loop() {
                Command::Exit => {
                    self.exiting = true;
                    break;
                },
                _ => (),
            }

            self.status.cycling = false;
            self.status.prev_cycles += self.status.cur_cycle;
            self.status.prev_elapsed = self.status.prev_elapsed + self.status.cur_elapsed();
            self.status.cur_cycle = Wrapping(0);
            self.status.cur_start_time = None;
            self.broadcast_status();
        }

        // Broadcast an `Exiting` to everyone.
        for &(_, ref res_tx) in self.req_res_pairs.iter() {
            // Handle this possible error?
            res_tx.send(Response::Exiting).ok();
        }
    }

    fn cycle_loop(&mut self) -> Command {
        // println!("Flywheel::cycle_loop: Looping {} times...", self.cycle_iters_max);

        loop {
            if (self.cycle_iters_max != 0) && (self.status.cur_cycle.0 >= self.cycle_iters_max) { break; }

            self.intake_sensory_frames().unwrap();

            self.cortex.cycle().expect("error cycling cortex");

            // Update cycle_counts:
            self.status.cur_cycle += Wrapping(1);
            self.status.cycle_counter += Wrapping(1);

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

            // self.output_motor_frames();

            // Process pending requests:
            self.fulfill_requests();
        }

        Command::None
    }

    fn fulfill_requests(&mut self) {
        // This shouldn't allocate (unless pushed to below), but this may be
        // better allocated on the `Flywheel`:
        let mut disconnected_pair_idxs = Vec::new();

        for (pair_idx, &(ref req_rx, ref res_tx)) in self.req_res_pairs.iter().enumerate() {
            loop {
                match req_rx.try_recv() {
                    Ok(r) => {
                        match r {
                            Request::Sampler { area_name, kind, buffer_kind, backpressure } => {
                                let tract_rx = self.cortex.areas_mut().by_key_mut(area_name.as_str()).unwrap()
                                    .sampler(kind, buffer_kind, backpressure);
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
                                match res_tx.send(Response::QueuesFinished(self.status.cycle_counter.0)) {
                                    Ok(_) => (),
                                    Err(err) => if !self.exiting { panic!("{}", err); }
                                }
                            },
                        }
                    }
                    Err(e) => match e {
                        TryRecvError::Empty => break,
                        // TODO: Have this either do nothing or check to see
                        // if any senders remain and exit if 0.
                        TryRecvError::Disconnected => {
                            disconnected_pair_idxs.push(pair_idx);
                            break;
                        },
                    },
                }
            }
        }

        for pair_idx in disconnected_pair_idxs {
            self.req_res_pairs.remove(pair_idx);
        }
    }

    // [NOTE]: Incoming array values beyond the length of destination slice will
    // be silently ignored.
    fn intake_sensory_frames(&mut self) -> CmnResult<()> {
        for &(ref sen_rx, _pathway_idx) in self.sensory_rxs.iter() {
            loop {
                match sen_rx.try_recv() {
                    Ok(s) => {
                        match s {
                            SensoryFrame::F32Array16(_arr) => {
                                // println!("Intaking sensory frame [pathway id: {}]: {:?} ...",
                                //     pathway_idx, arr);

                                // // let pathway = match try!(self.cortex.thal_mut().input_generator_frame(pathway_idx)) {
                                // let pathway = match self.cortex.thal_mut().input_generator(pathway_idx)? {
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
                                PathwayConfig::EncoderRanges(_ranges) => {

                                    // self.cortex.thal_mut().input_generator(pathway_idx)?
                                    //     .set_encoder_ranges(ranges);

                                    unimplemented!();
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

    #[allow(dead_code)]
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
        unimplemented!();
    }

    fn broadcast_status(&self) {
        for &(_, ref res_tx) in self.req_res_pairs.iter() {
            // TODO: Remove unnecessary (redundant) heap allocation:
            res_tx.send(Response::Status(Box::new(self.status.clone()))).unwrap();
        }
    }

    fn send_area_info(&self, res_tx: &Sender<Response>) {
        res_tx.send(Response::AreaInfo(Box::new(
            AreaInfo {
                name: self.area_name.to_string(),
                aff_out_slc_ids: self.cortex.areas().by_key(self.area_name.as_str())
                    .unwrap().area_map().aff_out_slc_ids(),
                tract_map: self.cortex.areas().by_key(self.area_name.as_str())
                    .unwrap().axon_tract_map(),
            }
        ))).expect("Error sending area info.");
    }
}
