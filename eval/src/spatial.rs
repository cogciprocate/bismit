
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
use rand;
use rand::distributions::{Range, IndependentSample};
use vibi::bismit::ocl::{Buffer, RwVec, WriteGuard};
use vibi::bismit::{Cortex, Thalamus, SubcorticalNucleus};
use vibi::bismit::flywheel::{Command, Request, Response};
use vibi::bismit::map::{AxonDomainRoute, AreaMap};
use vibi::bismit::encode::{self, ScalarSdrWriter};


pub struct Params {
    pub cmd_tx: Sender<Command>,
    pub req_tx: Sender<Request>,
    pub res_rx: Receiver<Response>,
    pub tract_buffer: RwVec<u8>,
    pub axns: Buffer<u8>,
    pub l4_axns: Buffer<u8>,
    pub area_map: AreaMap,
    pub encode_dim: u32,
    pub area_dim: u32,
}


pub(crate) struct Nucleus {
    area_name: String,
}

impl Nucleus {
    pub fn new<S: Into<String>>(area_name: S, lyr_name: &'static str, tar_area: &'static str,
            cortex: &Cortex) -> Nucleus
    {
        let area_name = area_name.into();

        // let v0_ext_lyr_addr = *cortex.areas().by_key(area_name.as_str()).unwrap()
        //     .area_map().layer_map().layers().by_key(lyr_name).unwrap().layer_addr();
        // let v1_in_lyr_buf = cortex.areas().by_key(tar_area).unwrap()
        //     .axns().create_layer_sub_buffer(v0_ext_lyr_addr, AxonDomainRoute::Input);
        // let axns = cortex.areas().by_key(tar_area).unwrap()
        //     .axns().states().clone();
        // let area_map = cortex.areas().by_key(area_name.as_str()).unwrap()
        //     .area_map().clone();

        Nucleus {
            area_name: area_name.into()
        }
    }
}

impl SubcorticalNucleus for Nucleus {
    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }

    fn pre_cycle(&mut self, _thal: &mut Thalamus) {

    }

    fn post_cycle(&mut self, _thal: &mut Thalamus) {

    }
}


fn cycle(params: &Params, training_iters: usize, collect_iters: usize, pattern_count: usize,
         sdrs: &Vec<Vec<u8>>, activity_counts: &mut Vec<Vec<usize>>)
{
    let mut rng = rand::weak_rng();
    let mut exiting = false;
    let mut cycle_count = 0u32;

    // Main loop:
    for i in 0..training_iters + collect_iters {
        // Write a random SDR.
        let pattern_idx = Range::new(0, pattern_count).ind_sample(&mut rng);
        debug!("Locking tract buffer...");
        let mut guard = params.tract_buffer.clone().write().wait().unwrap();
        debug_assert!(guard.len() == sdrs[pattern_idx].len());
        for (src, dst) in sdrs[pattern_idx].iter().zip(guard.iter_mut()) {
            *dst = *src;
        }
        WriteGuard::release(guard);

        // Cycle.
        params.cmd_tx.send(Command::Iterate(1)).unwrap();

        // Wait for completion.
        loop {
            debug!("Attempting to receive...");
            match params.res_rx.recv() {
                Ok(res) => match res {
                    Response::Status(status) => {
                        debug!("Status: {:?}", status);
                        if status.prev_cycles > cycle_count {
                            params.req_tx.send(Request::FinishQueues).unwrap();
                            params.cmd_tx.send(Command::None).unwrap();
                        }
                    },
                    Response::QueuesFinished(prev_cycles) => {
                        if prev_cycles > cycle_count {
                            debug!("Queues finished for: {}", prev_cycles);
                            cycle_count = cycle_count.wrapping_add(1);
                            break;
                        }
                    },
                    Response::Exiting => {
                        exiting = true;
                        break;
                    },
                    res @ _ => panic!("Unknown response received: {:?}", res),
                },
                Err(_) => {
                    exiting = true;
                    break;
                }
            };
        }

        if i >= training_iters {
            // Increment the cell activity counts.
            let l4_axns = params.l4_axns.map().read().enq().unwrap();
            for (counts, &axn) in activity_counts.iter_mut().zip(l4_axns.iter()) {
                counts[pattern_idx] += (axn > 0) as usize;
            }
        }

        if exiting { break; }
    }
}


// pub(crate) fn eval(cmd_tx: Sender<Command>, req_tx: Sender<Request>, res_rx: Receiver<Response>,
//         tract_buffer: RwVec<u8>, axns: Buffer<u8>, l4_axns: Buffer<u8>, area_map: AreaMap,
//         encode_dim: u32, area_dim: u32)
pub(crate) fn eval(params: Params) {
    const SPARSITY: usize = 48;
    let pattern_count = 64;
    let sdr_len = (params.encode_dim * params.encode_dim) as usize;
    let sdr_active_count = sdr_len / SPARSITY;

    let mut rng = rand::weak_rng();

    // Produce randomized indexes:
    let pattern_indices: Vec<_> = (0..pattern_count).map(|_| {
            encode::gen_axn_idxs(&mut rng, sdr_active_count, sdr_len)
        }).collect();

    // Create sdr from randomized indexes:
    let sdrs: Vec<_> = pattern_indices.iter().map(|axn_idxs| {
            let mut sdr = vec![0u8; sdr_len];
            for &axn_idx in axn_idxs.iter() {
                sdr[axn_idx] = Range::new(96, 160).ind_sample(&mut rng);
            }
            sdr
        }).collect();

    let cell_count = (params.area_dim * params.area_dim) as usize;

    // The number of times each cell has become active for each pattern:
    let mut activity_counts_start = vec![vec![0; pattern_count]; cell_count];
    let mut activity_counts_end = vec![vec![0; pattern_count]; cell_count];

    // Get the flywheel moving:
    params.cmd_tx.send(Command::None).unwrap();

    let training_iters_start = 0;
    let collect_iters_start = 20000;
    cycle(&params, training_iters_start, collect_iters_start, pattern_count, &sdrs, &mut activity_counts_start);

    let training_iters_end = 100000;
    let collect_iters_end = 20000;
    cycle(&params, training_iters_end, collect_iters_end, pattern_count, &sdrs, &mut activity_counts_end);

    println!("\nStart Activity Counts:");
    print_activity_counts(&activity_counts_start, collect_iters_start / 1000);

    println!("\nEnd Activity Counts:");
    print_activity_counts(&activity_counts_end, collect_iters_end / 1000);

    params.cmd_tx.send(Command::Exit).unwrap();

    println!("Spatial evaluation complete.");
    params.res_rx.recv().unwrap();
}

fn print_activity_counts(activity_counts: &Vec<Vec<usize>>, min: usize) {
    let pattern_count = activity_counts[0].len();
    let mut cond_counts: Vec<(usize, usize)> = Vec::with_capacity(pattern_count);

    for (axn_idx, counts) in activity_counts.iter().enumerate() {
        let mut ttl_count = 0;
        cond_counts.clear();

        for (pattern_idx, &count) in counts.iter().enumerate() {
            if count > min {
                ttl_count += count;
                cond_counts.push((pattern_idx, count));
            }
        }
        if ttl_count > 0 {
            println!("Cell [{}]({}): {:?}", axn_idx, ttl_count, cond_counts);
        }
    }
}