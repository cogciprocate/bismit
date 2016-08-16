use std::ops::Range;
// use std::io::{self, Write};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
// use std::str::{FromStr};
use time::{self, Timespec, Duration};

use ::{Cortex, OclEvent, LayerMapSchemeList, AreaSchemeList, CorticalAreaSettings};
use ::map::SliceTractMap;


/// Cycle control commands.
#[derive(Clone, Debug)]
pub enum CyCtl {
    None,
    Iterate(u32),
    Sample(Range<u8>, Arc<Mutex<Vec<u8>>>),
    // RequestCurrentAreaInfo,
    RequestCurrentIter,
    // ViewAllSlices(bool),
    // ViewBufferDebug(bool),
    Stop,
    Exit,
}


/// Cycle result responses.
#[derive(Clone, Debug)]
pub enum CyRes {
    // None,
    CurrentIter(u32),
    Status(Box<Status>),
    // AreaInfo(Box<AreaInfo>),
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
    pub cur_cycle: u32,
    pub prev_cycles: u32,
    pub prev_elapsed: Duration,
    pub cur_start_time: Option<Timespec>,
}

impl Status {
    pub fn new() -> Status {
        Status {
            cur_cycle: 0,
            prev_cycles: 0,
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



pub enum LoopAction {
    None,
    Break,
    Continue,
}


pub struct Flywheel {
    control_rx: Receiver<CyCtl>,
    result_tx: Sender<CyRes>,
    cortex: Cortex,
    cycle_iters: u32,
    area_name: String,
    status: Status,
}

impl Flywheel {
    pub fn new(control_rx: Receiver<CyCtl>, result_tx: Sender<CyRes>,
                lm_schemes: LayerMapSchemeList, a_schemes: AreaSchemeList,
                ca_settings: Option<CorticalAreaSettings>) -> Flywheel {
        let cortex = Cortex::new(lm_schemes, a_schemes, ca_settings);
        let area_name = "v1".to_owned();

        Flywheel {
            control_rx: control_rx,
            result_tx: result_tx,
            cortex: cortex,
            cycle_iters: 1,
            area_name: area_name,
            status: Status::new(),
        }
    }

    pub fn spin(&mut self) {
        loop {
            match self.control_rx.recv() {
                Ok(cyctl) => match cyctl {
                    CyCtl::Iterate(i) => self.cycle_iters = i,
                    CyCtl::Exit => break,
                    CyCtl::Sample(range, buf) => {
                        self.refresh_hex_grid_buf(range, buf);
                        continue;
                    },
                    // CyCtl::RequestCurrentAreaInfo => {
                    //     result_tx.send(CyRes::AreaInfo(Box::new(AreaInfo {
                    //         name: self.area_name.to_string(),
                    //         aff_out_slc_range: self.cortex.area(&self.area_name).area_map().aff_out_slc_range(),
                    //         tract_map: self.cortex.area(self.area_name).axn_tract_map(),
                    //     }))).expect("Error sending area info.");
                    //     continue;
                    // },
                    _ => continue,
                },

                Err(e) => panic!("run(): control_rx.recv(): '{:?}'", e),
            }

            self.status.cur_start_time = Some(time::get_time());
            self.status.cur_cycle = 0;

            // Send a Status with updated time:
            self.result_tx.send(CyRes::Status(Box::new(self.status.clone()))).ok();

            // Run primary loop and check for exit response:
            match self.cycle_loop() {
                CyCtl::Exit => break,
                _ => (),
            }

            self.status.prev_cycles += self.status.cur_cycle;
            self.status.prev_elapsed = self.status.prev_elapsed + self.status.cur_elapsed();
            self.status.cur_cycle = 0;
            self.status.cur_start_time = None;
            self.result_tx.send(CyRes::Status(Box::new(self.status.clone()))).ok();
        }
    }

    fn cycle_loop(&mut self) -> CyCtl {
        loop {
            if self.status.cur_cycle >= (self.cycle_iters - 1) { break; }

            self.cortex.cycle();

            // Update current cycle:
            self.status.cur_cycle += 1;

            // Respond to any requests:
            // Not sure why we're incrementing `cur_cycle` a second time.
            if let Ok(c) = self.control_rx.try_recv() {
                match c {
                    CyCtl::RequestCurrentIter => {
                        self.result_tx.send(CyRes::CurrentIter(self.status.cur_cycle + 1)).unwrap()
                    },
                    // If a new sample has been requested, fulfill it:
                    CyCtl::Sample(range, buf) => {
                        self.refresh_hex_grid_buf(range, buf);
                    },
                    CyCtl::Stop => {
                        return CyCtl::Stop;
                    },
                    _ => return c,
                }
            }
        }

        CyCtl::None
    }


    fn refresh_hex_grid_buf(&self, slc_range: Range<u8>, buf: Arc<Mutex<Vec<u8>>>)
                -> Option<OclEvent> {
        let axn_range = self.cortex.area(&self.area_name).axn_tract_map().axn_id_range(slc_range.clone());

        // match buf.try_lock() {
        match buf.lock() {
            // Ok(ref mut b) => self.cortex.area(&self.area_name).sample_aff_out(&mut b[range]),
            Ok(ref mut b) => Some(self.cortex.area(&self.area_name)
                .sample_axn_slc_range(slc_range, &mut b[axn_range])),
            Err(_) => None,
        }
    }
}


