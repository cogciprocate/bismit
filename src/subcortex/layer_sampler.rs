// #![allow(dead_code, unused_imports, unused_variables)]

use std::mem;
use std::collections::HashMap;
use futures::{Future, Poll, Async};
use ::{Error as CmnError, Thalamus, CorticalAreas, TractReceiver, SamplerKind,
    SamplerBufferKind, FutureRecv, FutureReadGuardUntyped, ReadGuardUntyped};


pub struct LayerSamples(pub HashMap<SamplerKind, ReadGuardUntyped>);


#[derive(Debug)]
pub enum RxState {
    Rx(FutureRecv),
    Lock(FutureReadGuardUntyped),
    Complete(ReadGuardUntyped),
}


#[derive(Debug)]
pub struct FutureLayerSamples(Vec<(SamplerKind, RxState)>);

impl FutureLayerSamples {
    pub fn new(rxs: &[(SamplerKind, TractReceiver)]) -> FutureLayerSamples {
        let fls = rxs.iter().map(|&(ref sk, ref rx)| {
            (sk.clone(), RxState::Rx(rx.recv(true)))
        }).collect();
        FutureLayerSamples(fls)
    }
}

impl Future for FutureLayerSamples {
    type Item = LayerSamples;
    type Error = CmnError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Poll each rx, returning `NotReady` if any is not ready:
        for &mut (_, ref mut state) in self.0.iter_mut() {
            let mut new_state = None;

            // Progress samplers in the `Rx` state:
            if let RxState::Rx(ref mut future_recv) = *state {
                match future_recv.poll() {
                    Ok(Async::Ready(buf)) => {
                        let future_read_guard = match buf {
                            Some(b) => FutureReadGuardUntyped::from(b),
                            // If the rx returned a `None`, `wait_for_frame`
                            // must be `false`.
                            //
                            // NOTE: This doesn't have to be an error (add a
                            // `RxState::Skip` variant if not?).
                            //
                            None => return Err(CmnError::from("FutureLayerSamples::poll: \
                                'wait_for_frame' is set to 'false'.")),
                        };
                        new_state = Some(RxState::Lock(future_read_guard));
                    },
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => return Err(err.into()),
                }
            }

            // Update state:
            if let Some(new_state) = new_state.take() {
                mem::replace(state, new_state);
            }

            // Progress samplers in the `Lock` state:
            if let RxState::Lock(ref mut future_guard) = *state {
                match future_guard.poll() {
                    Ok(Async::Ready(guard)) => {
                        new_state = Some(RxState::Complete(guard));
                    }
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => return Err(err.into()),
                }
            }

            // Update state:
            if let Some(new_state) = new_state.take() {
                mem::replace(state, new_state);
            }
        }

        // All rxs are ready/complete:
        let mut bufs = HashMap::new();
        for (sk, state) in self.0.drain(..) {

            match state {
                RxState::Complete(buf) => { bufs.insert(sk, buf); },
                _ => unreachable!(),
            }
        }
        Ok(Async::Ready(LayerSamples(bufs)))
    }
}


#[derive(Debug)]
pub enum CellSampleIdxs {
    All,
    Single(usize),
    Range(usize, usize),
    Modulo(usize),
}


#[derive(Debug)]
pub struct LayerSampler {
    idxs: CellSampleIdxs,
    rxs: Vec<(SamplerKind, TractReceiver)>,
}

impl LayerSampler {
    /// Returns a new layer sampler.
    pub fn new(area_name: &str, sampler_kinds: Vec<SamplerKind>, idxs: CellSampleIdxs,
            _thal: &mut Thalamus, cortical_areas: &mut CorticalAreas) -> LayerSampler {
        let area = cortical_areas.by_key_mut(area_name).unwrap();
        let mut rxs = Vec::with_capacity(sampler_kinds.len());

        for sk in sampler_kinds.into_iter() {
            let rx = area.sampler(sk.clone(), SamplerBufferKind::Single, true);
            rxs.push((sk, rx))
        }

        LayerSampler {
            idxs,
            rxs,
        }
    }

    /// Returns a new layer sampler which samples everything within a layer.
    pub fn everything(area_name: &str, layer_name: &str, idxs: CellSampleIdxs,
            thal: &mut Thalamus, cortical_areas: &mut CorticalAreas) -> LayerSampler {
        let layer_addr = thal.area_maps().by_key(area_name).expect("invalid area name")
            .layer_map().layers().by_key(layer_name).expect("invalid layer name")
            .layer_addr();

        let sampler_kinds = vec![
            SamplerKind::Axons(Some(layer_addr)),
            SamplerKind::SomaStates(layer_addr),
            SamplerKind::SomaEnergies(layer_addr),
            SamplerKind::SomaActivities(layer_addr),
            SamplerKind::SomaFlagSets(layer_addr),
            SamplerKind::TuftStates(layer_addr),
            SamplerKind::TuftBestDenIds(layer_addr),
            SamplerKind::TuftBestDenStatesRaw(layer_addr),
            SamplerKind::TuftBestDenStates(layer_addr),
            SamplerKind::TuftPrevStates(layer_addr),
            SamplerKind::TuftPrevBestDenIds(layer_addr),
            SamplerKind::TuftPrevBestDenStatesRaw(layer_addr),
            SamplerKind::TuftPrevBestDenStates(layer_addr),
            SamplerKind::DenStates(layer_addr),
            SamplerKind::DenStatesRaw(layer_addr),
            SamplerKind::DenEnergies(layer_addr),
            SamplerKind::DenActivities(layer_addr),
            SamplerKind::DenThresholds(layer_addr),
            SamplerKind::SynStates(layer_addr),
            SamplerKind::SynStrengths(layer_addr),
            SamplerKind::SynSrcColVOffs(layer_addr),
            SamplerKind::SynSrcColUOffs(layer_addr),
            SamplerKind::SynFlagSets(layer_addr),
        ];

        LayerSampler::new(area_name, sampler_kinds, idxs, thal, cortical_areas)
    }

    pub fn recv(&self) -> FutureLayerSamples {
        FutureLayerSamples::new(&self.rxs)
    }
}
