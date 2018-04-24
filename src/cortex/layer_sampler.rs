#![allow(unused_imports, dead_code)]

use std::mem;
use std::collections::HashMap;
use futures::{Future, Poll, Async, task::Context};
use ::{Error as CmnError, Thalamus, CorticalAreas, TractReceiver, SamplerKind,
    SamplerBufferKind, FutureRecv, FutureReadGuardVec, ReadGuardVec, CellSampleIdxs,
    FutureCorticalSamples, CorticalSampler, CorticalSamples};


/// Cortical layer samples.
#[derive(Debug)]
pub struct CorticalLayerSamples {
    samples: CorticalSamples,
}


/// Future samples.
#[derive(Debug)]
pub struct FutureCorticalLayerSamples {
    samples: FutureCorticalSamples,
}

impl Future for FutureCorticalLayerSamples {
    type Item = CorticalLayerSamples;
    type Error = CmnError;

    fn poll(&mut self, cx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.samples.poll(cx).map(|a| a.map(|s| CorticalLayerSamples { samples: s }))
    }
}


/// A cortical layer sampler.
#[derive(Debug)]
pub struct CorticalLayerSampler {
    sampler: CorticalSampler,
}

impl CorticalLayerSampler {
    /// Creates and returns a new `CorticalLayerSamplerBuilder`.
    pub fn builder<'b>(area_name: &'b str, layer_name: &'b str,
            thal: &'b mut Thalamus, cortical_areas: &'b mut CorticalAreas)
            -> CorticalLayerSamplerBuilder<'b> {
        CorticalLayerSamplerBuilder::new(area_name, layer_name, thal, cortical_areas)
    }

    /// Begins receiving for all samplers and returns a future representing
    /// reception completion.
    pub fn recv(&self) -> FutureCorticalLayerSamples {
        FutureCorticalLayerSamples {
            samples: FutureCorticalSamples::new(&self.sampler.rxs),
        }
    }

    /// Changes the backpressure setting for all contained tract receivers
    /// (samplers).
    pub fn set_backpressure(&self, bp: bool) {
        for &(_, ref rx) in self.sampler.rxs.iter() {
            rx.set_backpressure(bp);
        }
    }
}


/// A cortical layer sampler builder.
#[derive(Debug)]
pub struct CorticalLayerSamplerBuilder<'b> {
    area_name: &'b str,
    layer_name: &'b str,
    idxs: CellSampleIdxs,
    thal: &'b mut Thalamus,
    cortical_areas: &'b mut CorticalAreas,
    soma_states: bool,
    soma_energies: bool,
    soma_activities: bool,
    soma_flag_sets: bool,
    tuft_states: bool,
    tuft_best_den_ids: bool,
    tuft_best_den_states_raw: bool,
    tuft_best_den_states: bool,
    tuft_prev_states: bool,
    tuft_prev_best_den_ids: bool,
    tuft_prev_best_den_states_raw: bool,
    tuft_prev_best_den_states: bool,
    den_states: bool,
    den_states_raw: bool,
    den_energies: bool,
    den_activities: bool,
    den_thresholds: bool,
    syn_states: bool,
    syn_strengths: bool,
    syn_src_col_v_offs: bool,
    syn_src_col_u_offs: bool,
    syn_flag_sets: bool,

}

impl<'b> CorticalLayerSamplerBuilder<'b> {
    /// Creates and returns a new `CorticalLayerSamplerBuilder`.
    pub fn new(area_name: &'b str, layer_name: &'b str, thal: &'b mut Thalamus,
            cortical_areas: &'b mut CorticalAreas) -> CorticalLayerSamplerBuilder<'b> {
        CorticalLayerSamplerBuilder {
            area_name, layer_name,
            idxs: CellSampleIdxs::All,
            thal, cortical_areas,
            soma_states: false,
            soma_energies: false,
            soma_activities: false,
            soma_flag_sets: false,
            tuft_states: false,
            tuft_best_den_ids: false,
            tuft_best_den_states_raw: false,
            tuft_best_den_states: false,
            tuft_prev_states: false,
            tuft_prev_best_den_ids: false,
            tuft_prev_best_den_states_raw: false,
            tuft_prev_best_den_states: false,
            den_states: false,
            den_states_raw: false,
            den_energies: false,
            den_activities: false,
            den_thresholds: false,
            syn_states: false,
            syn_strengths: false,
            syn_src_col_v_offs: false,
            syn_src_col_u_offs: false,
            syn_flag_sets: false,
        }
    }

    // This isn't currently hooked up:
    pub fn idxs<'a>(&'a mut self, idxs: CellSampleIdxs) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.idxs = idxs;
        self
    }

    /// Use all soma fields.
    pub fn soma<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.soma_states = true;
        self.soma_energies = true;
        self.soma_activities = true;
        self.soma_flag_sets = true;
        self
    }

    /// Use all tuft fields.
    pub fn tuft<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_states = true;
        self.tuft_best_den_ids = true;
        self.tuft_best_den_states_raw = true;
        self.tuft_best_den_states = true;
        self.tuft_prev_states = true;
        self.tuft_prev_best_den_ids = true;
        self.tuft_prev_best_den_states_raw = true;
        self.tuft_prev_best_den_states = true;
        self
    }

    /// Use all den fields.
    pub fn den<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.den_states = true;
        self.den_states_raw = true;
        self.den_energies = true;
        self.den_activities = true;
        self.den_thresholds = true;
        self
    }

    /// Use all syn fields.
    pub fn syn<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.syn_states = true;
        self.syn_strengths = true;
        self.syn_src_col_v_offs = true;
        self.syn_src_col_u_offs = true;
        self.syn_flag_sets = true;
        self
    }

    /// Build and return a new `CorticalLayerSampler`.
    pub fn build(&mut self) -> CorticalLayerSampler {
        let layer_addr = self.thal.area_maps().by_key(self.area_name).expect("invalid area name")
            .layer_map().layers().by_key(self.layer_name).expect("invalid layer name")
            .layer_addr();

        let mut sampler_kinds = Vec::with_capacity(32);

        if self.soma_states { sampler_kinds.push(SamplerKind::SomaStates(layer_addr)) }
        if self.soma_energies { sampler_kinds.push(SamplerKind::SomaEnergies(layer_addr),) }
        if self.soma_activities { sampler_kinds.push(SamplerKind::SomaActivities(layer_addr),) }
        if self.soma_flag_sets { sampler_kinds.push(SamplerKind::SomaFlagSets(layer_addr),) }

        if self.tuft_states { sampler_kinds.push(SamplerKind::TuftStates(layer_addr),) }
        if self.tuft_best_den_ids { sampler_kinds.push(SamplerKind::TuftBestDenIds(layer_addr),) }
        if self.tuft_best_den_states_raw { sampler_kinds.push(SamplerKind::TuftBestDenStatesRaw(layer_addr),) }
        if self.tuft_best_den_states { sampler_kinds.push(SamplerKind::TuftBestDenStates(layer_addr),) }
        if self.tuft_prev_states { sampler_kinds.push(SamplerKind::TuftPrevStates(layer_addr),) }
        if self.tuft_prev_best_den_ids { sampler_kinds.push(SamplerKind::TuftPrevBestDenIds(layer_addr),) }
        if self.tuft_prev_best_den_states_raw { sampler_kinds.push(SamplerKind::TuftPrevBestDenStatesRaw(layer_addr),) }
        if self.tuft_prev_best_den_states { sampler_kinds.push(SamplerKind::TuftPrevBestDenStates(layer_addr),) }

        if self.den_states { sampler_kinds.push(SamplerKind::DenStates(layer_addr),) }
        if self.den_states_raw { sampler_kinds.push(SamplerKind::DenStatesRaw(layer_addr),) }
        if self.den_energies { sampler_kinds.push(SamplerKind::DenEnergies(layer_addr),) }
        if self.den_activities { sampler_kinds.push(SamplerKind::DenActivities(layer_addr),) }
        if self.den_thresholds { sampler_kinds.push(SamplerKind::DenThresholds(layer_addr),) }

        if self.syn_states { sampler_kinds.push(SamplerKind::SynStates(layer_addr),) }
        if self.syn_strengths { sampler_kinds.push(SamplerKind::SynStrengths(layer_addr),) }
        if self.syn_src_col_v_offs { sampler_kinds.push(SamplerKind::SynSrcColVOffs(layer_addr),) }
        if self.syn_src_col_u_offs { sampler_kinds.push(SamplerKind::SynSrcColUOffs(layer_addr),) }
        if self.syn_flag_sets { sampler_kinds.push(SamplerKind::SynFlagSets(layer_addr),) }

        CorticalLayerSampler {
            sampler: CorticalSampler::new(self.area_name, sampler_kinds,
                self.idxs.clone(), self.thal, self.cortical_areas),
        }
    }
}