//! Sampling for a cortex.

use std::mem;
use std::collections::HashMap;
use futures::{Future, Poll, Async, task::Context};
use ::{Error as CmnError, Thalamus, CorticalAreas, TractReceiver, SamplerKind,
    SamplerBufferKind, FutureRecv, FutureReadGuardVec, ReadGuardVec};


/// Layer samples.
#[derive(Debug)]
pub struct CorticalSamples {
    samples: HashMap<SamplerKind, ReadGuardVec>,
}

impl CorticalSamples {
    // TODO: Change return type to a result (with custom error).
    pub fn sample(&self, sk: &SamplerKind) -> Option<&ReadGuardVec> {
        self.samples.get(sk)
    }

    // TODO: Change return type to a result (with custom error).
    pub fn take_sample(&mut self, sk: &SamplerKind) -> Option<ReadGuardVec> {
        self.samples.remove(sk)
    }

    pub fn count(&self) -> usize {
        self.samples.len()
    }
}


/// The completion state for a sampler.
#[derive(Debug)]
pub enum RxState {
    Rx(FutureRecv),
    Lock(FutureReadGuardVec),
    Complete(ReadGuardVec),
}


/// Future layer samples.
#[derive(Debug)]
pub struct FutureCorticalSamples(Vec<(SamplerKind, RxState)>);

impl FutureCorticalSamples {
    pub fn new(rxs: &[(SamplerKind, TractReceiver)]) -> FutureCorticalSamples {
        let fls = rxs.iter().map(|&(ref sk, ref rx)| {
            (sk.clone(), RxState::Rx(rx.recv(true)))
        }).collect();
        FutureCorticalSamples(fls)
    }
}

impl Future for FutureCorticalSamples {
    type Item = CorticalSamples;
    type Error = CmnError;

    fn poll(&mut self, cx: &mut Context) -> Poll<Self::Item, Self::Error> {
        // Poll each rx, returning `NotReady` if any is not ready:
        for &mut (_, ref mut state) in self.0.iter_mut() {
            if let RxState::Complete(_) = *state { continue; }

            let mut new_state = None;

            // Progress samplers in the `Rx` state:
            if let RxState::Rx(ref mut future_recv) = *state {
                match future_recv.poll(cx) {
                    Ok(Async::Ready(buf)) => {
                        let future_read_guard = match buf {
                            Some(b) => FutureReadGuardVec::from(b),
                            // If the rx returned a `None`, `wait_for_frame`
                            // must be `false`.
                            //
                            // NOTE: This doesn't have to be an error (add a
                            // `RxState::Skip` variant if not?).
                            //
                            None => return Err(CmnError::from("FutureCorticalSamples::poll: \
                                'wait_for_frame' is set to 'false'.")),
                        };
                        new_state = Some(RxState::Lock(future_read_guard));
                    },
                    Ok(Async::Pending) => return Ok(Async::Pending),
                    Err(err) => return Err(err.into()),
                }
            }

            // Update state:
            if let Some(new_state) = new_state.take() {
                mem::replace(state, new_state);
            }

            // Progress samplers in the `Lock` state:
            if let RxState::Lock(ref mut future_guard) = *state {
                match future_guard.poll(cx) {
                    Ok(Async::Ready(guard)) => {
                        new_state = Some(RxState::Complete(guard));
                    }
                    Ok(Async::Pending) => return Ok(Async::Pending),
                    Err(err) => return Err(err.into()),
                }
            }

            // Update state:
            if let Some(new_state) = new_state.take() {
                mem::replace(state, new_state);
            }
        }

        // All rxs are ready/complete:
        let mut bufs = HashMap::with_capacity(self.0.len());
        for (sk, state) in self.0.drain(..) {
            match state {
                RxState::Complete(buf) => { bufs.insert(sk, buf); },
                _ => unreachable!(),
            }
        }
        Ok(Async::Ready(CorticalSamples { samples: bufs }))
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellSampleIdxs {
    All,
    Single(usize),
    Range(usize, usize),
    Modulo(usize),
}


#[derive(Debug)]
pub struct CorticalSampler {
    idxs: CellSampleIdxs,
    pub(crate) rxs: Vec<(SamplerKind, TractReceiver)>,
}

impl CorticalSampler {
    /// Returns a new layer sampler.
    pub fn new(area_name: &str, sampler_kinds: Vec<SamplerKind>, idxs: CellSampleIdxs,
            _thal: &mut Thalamus, cortical_areas: &mut CorticalAreas) -> CorticalSampler {
        // NOTE: Implementing sample index ranges will require some pretty
        // serious redesigning. Indexing a specific range/group of cells means
        // that there will need to be separate groups of buffers for each
        // tuft. There will also be further complexities involved if
        // rectangular/cubic sections are desired.
        assert!(idxs == CellSampleIdxs::All, "Only `CellSampleIdxs::All` is currently implemented.");
        let area = cortical_areas.by_key_mut(area_name).unwrap();
        let mut rxs = Vec::with_capacity(sampler_kinds.len());

        for sk in sampler_kinds.into_iter() {
            let rx = area.sampler(sk.clone(), SamplerBufferKind::Single, true);
            rxs.push((sk, rx))
        }

        CorticalSampler {
            idxs,
            rxs,
        }
    }

    // /// Returns a new layer sampler which samples everything within a layer.
    // #[deprecated(note = "This method is liable to become out of date.")]
    // pub fn everything(area_name: &str, layer_name: &str, idxs: CellSampleIdxs,
    //         thal: &mut Thalamus, cortical_areas: &mut CorticalAreas) -> CorticalSampler {
    //     let layer_addr = thal.area_maps().by_key(area_name).expect("invalid area name")
    //         .layer_map().layers().by_key(layer_name).expect("invalid layer name")
    //         .layer_addr();

    //     let sampler_kinds = vec![
    //         SamplerKind::Axons(Some(layer_addr)),
    //         SamplerKind::SomaStates(layer_addr),
    //         SamplerKind::SomaEnergies(layer_addr),
    //         SamplerKind::SomaActivities(layer_addr),
    //         SamplerKind::SomaFlagSets(layer_addr),
    //         SamplerKind::TuftStates(layer_addr),
    //         SamplerKind::TuftBestDenIds(layer_addr),
    //         SamplerKind::TuftBestDenStatesRaw(layer_addr),
    //         SamplerKind::TuftBestDenStates(layer_addr),
    //         SamplerKind::TuftPrevStates(layer_addr),
    //         SamplerKind::TuftPrevBestDenIds(layer_addr),
    //         SamplerKind::TuftPrevBestDenStatesRaw(layer_addr),
    //         SamplerKind::TuftPrevBestDenStates(layer_addr),
    //         SamplerKind::DenStates(layer_addr),
    //         SamplerKind::DenStatesRaw(layer_addr),
    //         SamplerKind::DenEnergies(layer_addr),
    //         SamplerKind::DenActivities(layer_addr),
    //         SamplerKind::DenThresholds(layer_addr),
    //         SamplerKind::SynStates(layer_addr),
    //         SamplerKind::SynStrengths(layer_addr),
    //         SamplerKind::SynSrcColVOffs(layer_addr),
    //         SamplerKind::SynSrcColUOffs(layer_addr),
    //         SamplerKind::SynFlagSets(layer_addr),
    //     ];

    //     CorticalSampler::new(area_name, sampler_kinds, idxs, thal, cortical_areas)
    // }

    /// Begins receiving for all samplers and returns a future representing
    /// reception completion.
    pub fn recv(&self) -> FutureCorticalSamples {
        FutureCorticalSamples::new(&self.rxs)
    }

    /// Changes the backpressure setting for all contained tract receivers
    /// (samplers).
    pub fn set_backpressure(&self, bp: bool) {
        for &(_, ref rx) in self.rxs.iter() {
            rx.set_backpressure(bp);
        }
    }
}
