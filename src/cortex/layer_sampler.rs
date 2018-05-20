//! A cortical layer sampler.

use std::ops::{RangeBounds, Bound::{Excluded, Included, Unbounded}};
use num::Integer;
use futures::{Future, Poll, task::Context};
use ocl::ReadGuard;
use map::{axon_idx, AxonBoundError};
use cortex::{Cell as CellMap, Tuft as TuftMap, Dendrite as DendriteMap, Synapse as SynapseMap};
use ::{Error as CmnError, Thalamus, CorticalAreas,  SamplerKind, CellSampleIdxs,
    FutureCorticalSamples, CorticalSampler, CorticalSamples, LayerAddress,
    DataCellLayerMap, SlcId};


/// Determines start and end indexes for a range and length.
fn bounds_start_end<R, T>(range: R, len: T) -> (T, T)
        where T: Integer + Copy, R: RangeBounds<T> {
    let start = match range.start() {
        Included(&n) => n,
        Excluded(&n) => n + T::one(),
        Unbounded    => T::zero(),
    };
    let end = match range.end() {
        Included(&n) => n + T::one(),
        Excluded(&n) => n,
        Unbounded    => len,
    };
    assert!(start <= end);
    assert!(end <= len);
    (start, end)
}


/// A synapse sample.
#[derive(Debug)]
pub struct Synapse<'d> {
    den: &'d Dendrite<'d>,
    map: SynapseMap<'d>,
}

impl<'d> Synapse<'d> {
    /// Returns the index of this synapse's source axon.
    pub fn src_axon_idx(&self) -> Result<u32, AxonBoundError> {
        let layer = self.den.tuft.cell.layer.map();
        let slc_count = layer.slice_map().axon_idzs().len() as SlcId;
        let src_slc_id = self.src_slc_id();
        debug_assert!(src_slc_id < slc_count);

        let (src_slc_axon_idz, src_slc_dims) = unsafe {
            (*layer.slice_map().axon_idzs().get_unchecked(src_slc_id as usize),
                &*layer.slice_map().dims().get_unchecked(src_slc_id as usize))
        };

        axon_idx(src_slc_axon_idz, slc_count, src_slc_id,
            src_slc_dims.v_size(), src_slc_dims.v_scale(),
            self.den.tuft.cell.map.v_id(), self.src_col_v_ofs(),
            src_slc_dims.u_size(), src_slc_dims.u_scale(),
            self.den.tuft.cell.map.u_id(), self.src_col_u_ofs())
    }

    /// Returns the source axon state.
    pub fn src_axon_state(&self) -> Result<u8, AxonBoundError> {
        let vec = self.den.tuft.cell.layer.axon_states().expect("Axon state sample unavailable.");
        // Index safely for this one just in case:
        self.src_axon_idx().map(|idx| vec[idx as usize])
    }

    /// Returns the synapse state.
    pub fn state(&self) -> u8 {
        let vec = self.den.tuft.cell.layer.syn_states().expect("Synapse state sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the synapse strength.
    pub fn strength(&self) -> i8 {
        let vec = self.den.tuft.cell.layer.syn_strengths().expect("Synapse strength sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the synapse source slice id.
    pub fn src_slc_id(&self) -> SlcId {
        let vec = self.den.tuft.cell.layer.syn_src_slc_ids().expect("Synapse src_slc_id sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the synapse source column `v` offset.
    pub fn src_col_v_ofs(&self) -> i8 {
        let vec = self.den.tuft.cell.layer.syn_src_col_v_offs().expect("Synapse src_col_v_ofs sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the synapse source column `u` offset.
    pub fn src_col_u_ofs(&self) -> i8 {
        let vec = self.den.tuft.cell.layer.syn_src_col_v_offs().expect("Synapse src_col_v_ofs sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the synapse flag set.
    pub fn flag_set(&self) -> u8 {
        let vec = self.den.tuft.cell.layer.syn_flag_sets().expect("Synapse flag_set sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the synapse state.
    pub fn try_state(&self) -> Option<u8> {
        self.den.tuft.cell.layer.syn_states().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the synapse strength.
    pub fn try_strength(&self) -> Option<i8> {
        self.den.tuft.cell.layer.syn_strengths().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the synapse source slice id.
    pub fn try_src_slc_id(&self) -> Option<SlcId> {
        self.den.tuft.cell.layer.syn_src_slc_ids().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the synapse source column `v` offset.
    pub fn try_src_col_v_ofs(&self) -> Option<i8> {
        self.den.tuft.cell.layer.syn_src_col_v_offs().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the synapse source column `u` offset.
    pub fn try_src_col_u_ofs(&self) -> Option<i8> {
        self.den.tuft.cell.layer.syn_src_col_u_offs().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the synapse flag set.
    pub fn try_flag_set(&self) -> Option<u8> {
        self.den.tuft.cell.layer.syn_flag_sets().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the synapse map.
    pub fn map(&self) -> &SynapseMap<'d> {
        &self.map
    }
}


/// An iterator over the synapses of a dendrite.
#[derive(Debug)]
pub struct Synapses<'d> {
    den: &'d Dendrite<'d>,
    next_id: u32,
    end_id: u32,
}

impl<'d> Iterator for Synapses<'d> {
    type Item = Synapse<'d>;

    fn next(&mut self) -> Option<Synapse<'d>> {
        let syn_id = self.next_id;
        self.next_id += 1;
        if syn_id < self.end_id {
            unsafe { Some(Synapse { den: self.den, map: self.den.map.synapse_unchecked(syn_id) }) }
        } else {
            None
        }
    }
}


/// A dendrite sample.
#[derive(Debug)]
pub struct Dendrite<'t> {
    tuft: &'t Tuft<'t>,
    map: DendriteMap<'t>,
}

impl<'t> Dendrite<'t> {
    /// Returns the dendrite state.
    pub fn state(&self) -> u8 {
        let vec = self.tuft.cell.layer.den_states().expect("Dendrite state sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the raw dendrite state.
    pub fn state_raw(&self) -> u8 {
        let vec = self.tuft.cell.layer.den_states_raw().expect("Dendrite state_raw sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the dendrite energy.
    pub fn energy(&self) -> u8 {
        let vec = self.tuft.cell.layer.den_energies().expect("Dendrite energy sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the dendrite activity rating.
    pub fn activity(&self) -> u8 {
        let vec = self.tuft.cell.layer.den_activities().expect("Dendrite activity sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the dendrite threshold.
    pub fn threshold(&self) -> u8 {
        let vec = self.tuft.cell.layer.den_thresholds().expect("Dendrite threshold sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the dendrite state.
    pub fn try_state(&self) -> Option<u8> {
        self.tuft.cell.layer.den_states().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the raw dendrite state.
    pub fn try_state_raw(&self) -> Option<u8> {
        self.tuft.cell.layer.den_states_raw().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the dendrite energy.
    pub fn try_energy(&self) -> Option<u8> {
        self.tuft.cell.layer.den_energies().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the dendrite activity rating.
    pub fn try_activity(&self) -> Option<u8> {
        self.tuft.cell.layer.den_activities().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the dendrite threshold.
    pub fn try_threshold(&self) -> Option<u8> {
        self.tuft.cell.layer.den_thresholds().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the dendrite map.
    pub fn map(&self) -> &DendriteMap<'t> {
        &self.map
    }

    /// Returns a synapse sample.
    pub fn synapse<'d>(&'d self, den_id: u32) -> Synapse<'d> {
        Synapse { den: self, map: self.map.synapse(den_id) }
    }

    /// Returns an iterator over the specified range of synapses.
    pub fn synapses<'d, R>(&'d self, range: R) -> Synapses<'d>
            where R: RangeBounds<u32> {
        let syns_per_den = unsafe {
            self.tuft.cell.map.tuft_info().get_unchecked(self.tuft.map.tuft_id())
                .dims().syns_per_den()
        };

        let (start, end) = bounds_start_end(range, syns_per_den);

        Synapses {
            den: self,
            next_id: start,
            end_id: end,
        }
    }
}


/// An iterator over the dendrites of a tuft.
#[derive(Debug)]
pub struct Dendrites<'t> {
    tuft: &'t Tuft<'t>,
    next_id: u32,
    end_id: u32,
}

impl<'t> Iterator for Dendrites<'t> {
    type Item = Dendrite<'t>;

    fn next(&mut self) -> Option<Dendrite<'t>> {
        let den_id = self.next_id;
        self.next_id += 1;
        if den_id < self.end_id {
            unsafe { Some(Dendrite { tuft: self.tuft, map: self.tuft.map.dendrite_unchecked(den_id) }) }
        } else {
            None
        }
    }
}


/// A tuft sample.
#[derive(Debug)]
pub struct Tuft<'c> {
    cell: &'c Cell<'c>,
    map: TuftMap<'c>
}

impl<'c> Tuft<'c> {
    /// Returns the tuft's state.
    pub fn state(&self) -> u8 {
        let vec = self.cell.layer.tuft_states().expect("Tuft state sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the tuft's best dendrite id.
    pub fn best_den_id(&self) -> u8 {
        let vec = self.cell.layer.tuft_best_den_ids().expect("Tuft best_den_id sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the tuft's best dendrite state (raw).
    pub fn best_den_state_raw(&self) -> u8 {
        let vec = self.cell.layer.tuft_best_den_states_raw().expect("Tuft best_den_state_raw sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the tuft's best dendrite state.
    pub fn best_den_state(&self) -> u8 {
        let vec = self.cell.layer.tuft_best_den_states().expect("Tuft best_den_state sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the tuft's previous state.
    pub fn prev_state(&self) -> u8 {
        let vec = self.cell.layer.tuft_prev_states().expect("Tuft prev_state sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the tuft's previous best dendrite id.
    pub fn prev_best_den_id(&self) -> u8 {
        let vec = self.cell.layer.tuft_prev_best_den_ids().expect("Tuft prev_best_den_id sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the tuft's previous best dendrite state (raw).
    pub fn prev_best_den_state_raw(&self) -> u8 {
        let vec = self.cell.layer.tuft_prev_best_den_states_raw().expect("Tuft prev_best_den_state_raw sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the tuft's previous best dendrite state.
    pub fn prev_best_den_state(&self) -> u8 {
        let vec = self.cell.layer.tuft_prev_best_den_states().expect("Tuft prev_best_den_state sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the tuft's state.
    pub fn try_state(&self) -> Option<u8> {
        self.cell.layer.tuft_states().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the tuft's best dendrite id.
    pub fn try_best_den_id(&self) -> Option<u8> {
        self.cell.layer.tuft_best_den_ids().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the tuft's best dendrite state (raw).
    pub fn try_best_den_state_raw(&self) -> Option<u8> {
        self.cell.layer.tuft_best_den_states_raw().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the tuft's best dendrite state.
    pub fn try_best_den_state(&self) -> Option<u8> {
        self.cell.layer.tuft_best_den_states().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the tuft's previous state.
    pub fn try_prev_state(&self) -> Option<u8> {
        self.cell.layer.tuft_prev_states().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the tuft's previous best dendrite id.
    pub fn try_prev_best_den_id(&self) -> Option<u8> {
        self.cell.layer.tuft_prev_best_den_ids().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the tuft's previous best dendrite state (raw).
    pub fn try_prev_best_den_state_raw(&self) -> Option<u8> {
        self.cell.layer.tuft_prev_best_den_states_raw().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the tuft's previous best dendrite state.
    pub fn try_prev_best_den_state(&self) -> Option<u8> {
        self.cell.layer.tuft_prev_best_den_states().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the tuft map.
    pub fn map(&self) -> &TuftMap<'c> {
        &self.map
    }

    /// Returns a dendrite sample.
    pub fn dendrite<'t>(&'t self, den_id: u32) -> Dendrite<'t> {
        Dendrite { tuft: self, map: self.map.dendrite(den_id) }
    }

    /// Returns an iterator over the specified range of dendrites.
    pub fn dendrites<'d, R>(&'d self, range: R) -> Dendrites<'d>
            where R: RangeBounds<u32> {
        let dens_per_tft = unsafe {
            self.cell.map.tuft_info().get_unchecked(self.map.tuft_id())
                .dims().dens_per_tft()
        };

        let (start, end) = bounds_start_end(range, dens_per_tft);

        Dendrites {
            tuft: self,
            next_id: start,
            end_id: end,
        }
    }
}


/// An iterator over the tufts of a cell.
#[derive(Debug)]
pub struct Tufts<'c> {
    cell: &'c Cell<'c>,
    next_id: usize,
    end_id: usize,
}

impl<'c> Iterator for Tufts<'c> {
    type Item = Tuft<'c>;

    fn next(&mut self) -> Option<Tuft<'c>> {
        let tft_id = self.next_id;
        self.next_id += 1;
        if tft_id < self.end_id {
            unsafe { Some(Tuft { cell: self.cell, map: self.cell.map.tuft_unchecked(tft_id) }) }
        } else {
            None
        }
    }
}


#[derive(Debug)]
/// A cell sample.
pub struct Cell<'l> {
    layer: &'l CorticalLayerSamples,
    map: CellMap<'l>,
}

impl<'l> Cell<'l> {
    /// Returns the cell's axon state.
    pub fn axon_state(&self) -> u8 {
        let vec = self.layer.axon_states().expect("Cell axon state sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.axon_idx() as usize) }
    }

    /// Returns the cell's soma state.
    pub fn state(&self) -> u8 {
        let vec = self.layer.soma_states().expect("Cell soma state sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the cell's energy.
    pub fn energy(&self) -> u8 {
        let vec = self.layer.soma_energies().expect("Cell soma energy sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the cell's activity rating.
    pub fn activity(&self) -> u8 {
        let vec = self.layer.soma_activities().expect("Cell soma activity sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the cell's flag set.
    pub fn flag_set(&self) -> u8 {
        let vec = self.layer.soma_flag_sets().expect("Cell soma flag_set sample unavailable.");
        unsafe { *vec.get_unchecked(self.map.idx() as usize) }
    }

    /// Returns the cell's axon state.
    pub fn try_axon_state(&self) -> Option<u8> {
        self.layer.axon_states().map(|vec| unsafe { *vec.get_unchecked(self.map.axon_idx() as usize) })
    }

    /// Returns the cell's soma state.
    pub fn try_state(&self) -> Option<u8> {
        self.layer.soma_states().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the cell's energy.
    pub fn try_energy(&self) -> Option<u8> {
        self.layer.soma_energies().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the cell's activity rating.
    pub fn try_activity(&self) -> Option<u8> {
        self.layer.soma_activities().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the cell's flag set.
    pub fn try_flag_set(&self) -> Option<u8> {
        self.layer.soma_flag_sets().map(|vec| unsafe { *vec.get_unchecked(self.map.idx() as usize) })
    }

    /// Returns the cell map.
    pub fn map(&self) -> &CellMap<'l> {
        &self.map
    }

    /// Returns a tuft sample.
    pub fn tuft<'c>(&'c self, tuft_id: usize) -> Tuft<'c> {
        Tuft { cell: self, map: self.map.tuft(tuft_id) }
    }

    /// Returns the first proximal (basal) tuft.
    pub fn tuft_proximal<'c>(&'c self) -> Option<Tuft<'c>> {
        self.map.tuft_proximal().map(|tm| Tuft { cell: self, map: tm })
    }

    /// Returns the first distal (basal) tuft found.
    pub fn tuft_distal<'c>(&'c self) -> Option<Tuft<'c>> {
        self.map.tuft_distal().map(|tm| Tuft { cell: self, map: tm })
    }

    /// Returns the first apical (distal) tuft found.
    pub fn tuft_apical<'c>(&'c self) -> Option<Tuft<'c>> {
        self.map.tuft_apical().map(|tm| Tuft { cell: self, map: tm })
    }

    /// Returns an iterator over the specified range of tufts.
    pub fn tufts<'d, R>(&'d self, range: R) -> Tufts<'d>
            where R: RangeBounds<usize> {
        let tfts_per_cel = self.map.tuft_info().len();
        let (start, end) = bounds_start_end(range, tfts_per_cel);

        Tufts {
            cell: self,
            next_id: start,
            end_id: end,
        }
    }
}


/// An iterator over the cells of a layer.
#[derive(Debug)]
pub struct Cells<'l> {
    layer: &'l CorticalLayerSamples,
    // NOTE: consider using something bigger for `slc_id` since it can easily
    // hit max:
    next_slc_id: SlcId,
    end_slc_id: SlcId,
    next_v_id: u32,
    start_v_id: u32,
    end_v_id: u32,
    next_u_id: u32,
    start_u_id: u32,
    end_u_id: u32,
}

impl<'l> Iterator for Cells<'l> {
    type Item = Cell<'l>;

    fn next(&mut self) -> Option<Cell<'l>> {
        if self.next_slc_id == self.end_slc_id {
            return None;
        }

        let slc_id = self.next_slc_id;
        let v_id = self.next_v_id;
        let u_id = self.next_u_id;

        // Increment:
        self.next_u_id += 1;
        if self.next_u_id == self.end_u_id {
            self.next_u_id = self.start_u_id;
            self.next_v_id += 1;
            if self.next_v_id == self.end_v_id {
                self.next_v_id = self.start_v_id;
                // NOTE: consider using something bigger for `slc_id` since it
                // can easily hit max:
                assert!(self.next_slc_id < SlcId::max_value());
                self.next_slc_id += 1;
            }
        }

        unsafe { Some(Cell { layer: self.layer,
            map: self.layer.map.cell_unchecked(slc_id, v_id, u_id) }) }
    }
}


/// Cortical layer layer.
#[derive(Debug)]
pub struct CorticalLayerSamples {
    map: DataCellLayerMap,
    axon_states: Option<ReadGuard<Vec<u8>>>,
    soma_states: Option<ReadGuard<Vec<u8>>>,
    soma_energies: Option<ReadGuard<Vec<u8>>>,
    soma_activities: Option<ReadGuard<Vec<u8>>>,
    soma_flag_sets: Option<ReadGuard<Vec<u8>>>,
    tuft_states: Option<ReadGuard<Vec<u8>>>,
    tuft_best_den_ids: Option<ReadGuard<Vec<u8>>>,
    tuft_best_den_states_raw: Option<ReadGuard<Vec<u8>>>,
    tuft_best_den_states: Option<ReadGuard<Vec<u8>>>,
    tuft_prev_states: Option<ReadGuard<Vec<u8>>>,
    tuft_prev_best_den_ids: Option<ReadGuard<Vec<u8>>>,
    tuft_prev_best_den_states_raw: Option<ReadGuard<Vec<u8>>>,
    tuft_prev_best_den_states: Option<ReadGuard<Vec<u8>>>,
    den_states: Option<ReadGuard<Vec<u8>>>,
    den_states_raw: Option<ReadGuard<Vec<u8>>>,
    den_energies: Option<ReadGuard<Vec<u8>>>,
    den_activities: Option<ReadGuard<Vec<u8>>>,
    den_thresholds: Option<ReadGuard<Vec<u8>>>,
    syn_states: Option<ReadGuard<Vec<u8>>>,
    syn_strengths: Option<ReadGuard<Vec<i8>>>,
    syn_src_slc_ids: Option<ReadGuard<Vec<u8>>>,
    syn_src_col_v_offs: Option<ReadGuard<Vec<i8>>>,
    syn_src_col_u_offs: Option<ReadGuard<Vec<i8>>>,
    syn_flag_sets: Option<ReadGuard<Vec<u8>>>,
}

impl CorticalLayerSamples {
    fn new(mut samples: CorticalSamples, map: DataCellLayerMap) -> CorticalLayerSamples {
        let axon_states = samples.take_sample(&SamplerKind::Axons(None)).map(|s| s.into_u8());
        let soma_states = samples.take_sample(&SamplerKind::SomaStates(map.layer_addr())).map(|s| s.into_u8());
        let soma_energies = samples.take_sample(&SamplerKind::SomaEnergies(map.layer_addr())).map(|s| s.into_u8());
        let soma_activities = samples.take_sample(&SamplerKind::SomaActivities(map.layer_addr())).map(|s| s.into_u8());
        let soma_flag_sets = samples.take_sample(&SamplerKind::SomaFlagSets(map.layer_addr())).map(|s| s.into_u8());
        let tuft_states = samples.take_sample(&SamplerKind::TuftStates(map.layer_addr())).map(|s| s.into_u8());
        let tuft_best_den_ids = samples.take_sample(&SamplerKind::TuftBestDenIds(map.layer_addr())).map(|s| s.into_u8());
        let tuft_best_den_states_raw = samples.take_sample(&SamplerKind::TuftBestDenStatesRaw(map.layer_addr())).map(|s| s.into_u8());
        let tuft_best_den_states = samples.take_sample(&SamplerKind::TuftBestDenStates(map.layer_addr())).map(|s| s.into_u8());
        let tuft_prev_states = samples.take_sample(&SamplerKind::TuftPrevStates(map.layer_addr())).map(|s| s.into_u8());
        let tuft_prev_best_den_ids = samples.take_sample(&SamplerKind::TuftPrevBestDenIds(map.layer_addr())).map(|s| s.into_u8());
        let tuft_prev_best_den_states_raw = samples.take_sample(&SamplerKind::TuftPrevBestDenStatesRaw(map.layer_addr())).map(|s| s.into_u8());
        let tuft_prev_best_den_states = samples.take_sample(&SamplerKind::TuftPrevBestDenStates(map.layer_addr())).map(|s| s.into_u8());
        let den_states = samples.take_sample(&SamplerKind::DenStates(map.layer_addr())).map(|s| s.into_u8());
        let den_states_raw = samples.take_sample(&SamplerKind::DenStatesRaw(map.layer_addr())).map(|s| s.into_u8());
        let den_energies = samples.take_sample(&SamplerKind::DenEnergies(map.layer_addr())).map(|s| s.into_u8());
        let den_activities = samples.take_sample(&SamplerKind::DenActivities(map.layer_addr())).map(|s| s.into_u8());
        let den_thresholds = samples.take_sample(&SamplerKind::DenThresholds(map.layer_addr())).map(|s| s.into_u8());
        let syn_states = samples.take_sample(&SamplerKind::SynStates(map.layer_addr())).map(|s| s.into_u8());
        let syn_strengths = samples.take_sample(&SamplerKind::SynStrengths(map.layer_addr())).map(|s| s.into_i8());
        let syn_src_slc_ids = samples.take_sample(&SamplerKind::SynSrcSlcIds(map.layer_addr())).map(|s| s.into_u8());
        let syn_src_col_v_offs = samples.take_sample(&SamplerKind::SynSrcColVOffs(map.layer_addr())).map(|s| s.into_i8());
        let syn_src_col_u_offs = samples.take_sample(&SamplerKind::SynSrcColUOffs(map.layer_addr())).map(|s| s.into_i8());
        let syn_flag_sets = samples.take_sample(&SamplerKind::SynFlagSets(map.layer_addr())).map(|s| s.into_u8());

        if let Some(ref vec) = axon_states { assert!(vec.len() >= (map.axon_idz() + map.dims().cells()) as usize); }
        let cell_count = map.dims().cells() as usize;
        if let Some(ref vec) = soma_states { assert!(vec.len() == cell_count); }
        if let Some(ref vec) = soma_energies { assert!(vec.len() == cell_count); }
        if let Some(ref vec) = soma_activities { assert!(vec.len() == cell_count); }
        if let Some(ref vec) = soma_flag_sets { assert!(vec.len() == cell_count); }
        let tuft_count = cell_count * map.tuft_count();
        if let Some(ref vec) = tuft_states { assert!(vec.len() == tuft_count); }
        if let Some(ref vec) = tuft_best_den_ids { assert!(vec.len() == tuft_count); }
        if let Some(ref vec) = tuft_best_den_states_raw { assert!(vec.len() == tuft_count); }
        if let Some(ref vec) = tuft_best_den_states { assert!(vec.len() == tuft_count); }
        if let Some(ref vec) = tuft_prev_states { assert!(vec.len() == tuft_count); }
        if let Some(ref vec) = tuft_prev_best_den_ids { assert!(vec.len() == tuft_count); }
        if let Some(ref vec) = tuft_prev_best_den_states_raw { assert!(vec.len() == tuft_count); }
        if let Some(ref vec) = tuft_prev_best_den_states { assert!(vec.len() == tuft_count); }
        if let Some(ref vec) = den_states { assert!(vec.len() == map.den_count() as usize); }
        if let Some(ref vec) = den_states_raw { assert!(vec.len() == map.den_count() as usize); }
        if let Some(ref vec) = den_energies { assert!(vec.len() == map.den_count() as usize); }
        if let Some(ref vec) = den_activities { assert!(vec.len() == map.den_count() as usize); }
        if let Some(ref vec) = den_thresholds { assert!(vec.len() == map.den_count() as usize); }
        if let Some(ref vec) = syn_states { assert!(vec.len() == map.syn_count() as usize); }
        if let Some(ref vec) = syn_strengths { assert!(vec.len() == map.syn_count() as usize); }
        if let Some(ref vec) = syn_src_slc_ids { assert!(vec.len() == map.syn_count() as usize); }
        if let Some(ref vec) = syn_src_col_v_offs { assert!(vec.len() == map.syn_count() as usize); }
        if let Some(ref vec) = syn_src_col_u_offs { assert!(vec.len() == map.syn_count() as usize); }
        if let Some(ref vec) = syn_flag_sets { assert!(vec.len() == map.syn_count() as usize); }

        CorticalLayerSamples {
            map,
            axon_states,
            soma_states,
            soma_energies,
            soma_activities,
            soma_flag_sets,
            tuft_states,
            tuft_best_den_ids,
            tuft_best_den_states_raw,
            tuft_best_den_states,
            tuft_prev_states,
            tuft_prev_best_den_ids,
            tuft_prev_best_den_states_raw,
            tuft_prev_best_den_states,
            den_states,
            den_states_raw,
            den_energies,
            den_activities,
            den_thresholds,
            syn_states,
            syn_strengths,
            syn_src_slc_ids,
            syn_src_col_v_offs,
            syn_src_col_u_offs,
            syn_flag_sets,
        }
    }

    pub fn axon_states(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.axon_states.as_ref()
    }

    pub fn soma_states(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.soma_states.as_ref()
    }

    pub fn soma_energies(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.soma_energies.as_ref()
    }

    pub fn soma_activities(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.soma_activities.as_ref()
    }

    pub fn soma_flag_sets(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.soma_flag_sets.as_ref()
    }

    pub fn tuft_states(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.tuft_states.as_ref()
    }

    pub fn tuft_best_den_ids(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.tuft_best_den_ids.as_ref()
    }

    pub fn tuft_best_den_states_raw(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.tuft_best_den_states_raw.as_ref()
    }

    pub fn tuft_best_den_states(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.tuft_best_den_states.as_ref()
    }

    pub fn tuft_prev_states(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.tuft_prev_states.as_ref()
    }

    pub fn tuft_prev_best_den_ids(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.tuft_prev_best_den_ids.as_ref()
    }

    pub fn tuft_prev_best_den_states_raw(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.tuft_prev_best_den_states_raw.as_ref()
    }

    pub fn tuft_prev_best_den_states(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.tuft_prev_best_den_states.as_ref()
    }

    pub fn den_states(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.den_states.as_ref()
    }

    pub fn den_states_raw(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.den_states_raw.as_ref()
    }

    pub fn den_energies(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.den_energies.as_ref()
    }

    pub fn den_activities(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.den_activities.as_ref()
    }

    pub fn den_thresholds(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.den_thresholds.as_ref()
    }

    pub fn syn_states(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.syn_states.as_ref()
    }

    pub fn syn_strengths(&self) -> Option<&ReadGuard<Vec<i8>>> {
        self.syn_strengths.as_ref()
    }

    pub fn syn_src_slc_ids(&self) -> Option<&ReadGuard<Vec<SlcId>>> {
        self.syn_src_slc_ids.as_ref()
    }

    pub fn syn_src_col_v_offs(&self) -> Option<&ReadGuard<Vec<i8>>> {
        self.syn_src_col_v_offs.as_ref()
    }

    pub fn syn_src_col_u_offs(&self) -> Option<&ReadGuard<Vec<i8>>> {
        self.syn_src_col_u_offs.as_ref()
    }

    pub fn syn_flag_sets(&self) -> Option<&ReadGuard<Vec<u8>>> {
        self.syn_flag_sets.as_ref()
    }

    /// Returns a cell sample.
    pub fn cell<'l>(&'l self, slc_id_lyr: SlcId, v_id: u32, u_id: u32) -> Cell<'l> {
        Cell { layer: self, map: self.map.cell(slc_id_lyr, v_id, u_id) }
    }

    /// Returns an iterator over the specified range of cells.
    pub fn cells<'d, Rs, R>(&'d self, slc_range: Rs, v_range: R, u_range: R) -> Cells<'d>
            where Rs: RangeBounds<SlcId>, R: RangeBounds<u32> {
        let dims = self.map.dims();
        let (slc_start, slc_end) = bounds_start_end(slc_range, dims.depth());
        let (v_start, v_end) = bounds_start_end(v_range, dims.v_size());
        let (u_start, u_end) = bounds_start_end(u_range, dims.u_size());

        Cells {
            layer: self,
            next_slc_id: slc_start,
            end_slc_id: slc_end,
            next_v_id: v_start,
            start_v_id: v_start,
            end_v_id: v_end,
            next_u_id: u_start,
            start_u_id: u_start,
            end_u_id: u_end,
        }
    }

    /// Returns a reference to the layer map.
    pub fn map(&self) -> &DataCellLayerMap {
        &self.map
    }
}


/// Future samples.
#[derive(Debug)]
pub struct FutureCorticalLayerSamples {
    samples: FutureCorticalSamples,
    map: Option<DataCellLayerMap>,
}

impl Future for FutureCorticalLayerSamples {
    type Item = CorticalLayerSamples;
    type Error = CmnError;

    fn poll(&mut self, cx: &mut Context) -> Poll<Self::Item, Self::Error> {
        self.samples.poll(cx).map(|a| a.map(|s|
            CorticalLayerSamples::new(s, self.map.take().unwrap())
        ))
    }
}


/// A cortical layer sampler.
#[derive(Debug)]
pub struct CorticalLayerSampler {
    sampler: CorticalSampler,
    layer_addr: LayerAddress,
    map: DataCellLayerMap,
}

impl CorticalLayerSampler {
    /// Creates and returns a new `CorticalLayerSamplerBuilder`.
    pub fn builder<'b>(area_name: &'b str, layer_name: &'b str,
            thal: &'b mut Thalamus, cortical_areas: &'b mut CorticalAreas)
            -> CorticalLayerSamplerBuilder<'b> {
        CorticalLayerSamplerBuilder::new(area_name, layer_name, thal, cortical_areas)
    }

    /// Returns a future representing reception completion.
    pub fn recv(&self) -> FutureCorticalLayerSamples {
        FutureCorticalLayerSamples {
            samples: FutureCorticalSamples::new(&self.sampler.rxs),
            map: Some(self.map.clone()),
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
    axons: bool,
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
    syn_src_slc_ids: bool,
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
            axons: false,
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
            syn_src_slc_ids: false,
            syn_src_col_v_offs: false,
            syn_src_col_u_offs: false,
            syn_flag_sets: false,
        }
    }

    // This isn't currently hooked up:
    //
    // NOTE: Implementing sample index ranges will require some pretty serious
    // redesigning and is probably best implemented by an entirely different
    // type. Indexing a specific range/group of cells means that there will
    // need to be separate groups of buffers for each tuft. There will also be
    // further complexities involved if rectangular/cubic sections are
    // desired.
    #[allow(dead_code)]
    fn idxs<'a>(&'a mut self, _idxs: CellSampleIdxs) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        unimplemented!();
        // self.idxs = idxs;
        // self
    }

    /// Includes all axon layers.
    pub fn axons<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.axons = true;
        self
    }

    /// Includes all soma fields.
    pub fn soma<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.soma_states = true;
        self.soma_energies = true;
        self.soma_activities = true;
        self.soma_flag_sets = true;
        self
    }

    pub fn soma_states<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.soma_states = true;
        self
    }

    pub fn soma_energies<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.soma_energies = true;
        self
    }

    pub fn soma_activites<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.soma_activities = true;
        self
    }

    pub fn soma_flag_sets<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.soma_flag_sets = true;
        self

    }


    /// Includes all tuft fields.
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

    pub fn tuft_states<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_states = true;
        self
    }

    pub fn tuft_best_den_ids<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_best_den_ids = true;
        self
    }

    pub fn tuft_best_den_states_raw<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_best_den_states_raw = true;
        self
    }

    pub fn tuft_best_den_states<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_best_den_states = true;
        self
    }

    pub fn tuft_prev_states<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_prev_states = true;
        self
    }

    pub fn tuft_prev_best_den_ids<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_prev_best_den_ids = true;
        self
    }

    pub fn tuft_prev_best_den_states_raw<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_prev_best_den_states_raw = true;
        self
    }

    pub fn tuft_prev_best_den_states<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.tuft_prev_best_den_states = true;
        self
    }

    /// Includes all den fields.
    pub fn den<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.den_states = true;
        self.den_states_raw = true;
        self.den_energies = true;
        self.den_activities = true;
        self.den_thresholds = true;
        self
    }

    pub fn den_states<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.den_states = true;
        self
    }

    pub fn den_states_raw<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.den_states_raw = true;
        self
    }

    pub fn den_energies<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.den_energies = true;
        self
    }

    pub fn den_activites<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.den_activities = true;
        self
    }

    pub fn den_thresholds<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.den_thresholds = true;
        self
    }


    /// Includes all syn fields.
    pub fn syn<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.syn_states = true;
        self.syn_strengths = true;
        self.syn_src_slc_ids = true;
        self.syn_src_col_v_offs = true;
        self.syn_src_col_u_offs = true;
        self.syn_flag_sets = true;
        self
    }

    pub fn syn_states<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.syn_states = true;
        self
    }

    pub fn syn_strengths<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.syn_strengths = true;
        self
    }

    pub fn syn_src_slc_ids<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.syn_src_slc_ids = true;
        self
    }

    pub fn syn_src_col_v_offs<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.syn_src_col_v_offs = true;
        self
    }

    pub fn syn_src_col_u_offs<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.syn_src_col_u_offs = true;
        self
    }

    pub fn syn_flag_sets<'a>(&'a mut self) -> &'a mut CorticalLayerSamplerBuilder<'b> {
        self.syn_flag_sets = true;
        self
    }

    /// Build and return a new `CorticalLayerSampler`.
    pub fn build(&mut self) -> CorticalLayerSampler {
        let layer_addr = self.thal.layer_addr(self.area_name, self.layer_name);

        let mut sampler_kinds = Vec::with_capacity(32);

        if self.axons { sampler_kinds.push(SamplerKind::Axons(None)) }
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
        if self.syn_src_slc_ids { sampler_kinds.push(SamplerKind::SynSrcSlcIds(layer_addr),) }
        if self.syn_src_col_v_offs { sampler_kinds.push(SamplerKind::SynSrcColVOffs(layer_addr),) }
        if self.syn_src_col_u_offs { sampler_kinds.push(SamplerKind::SynSrcColUOffs(layer_addr),) }
        if self.syn_flag_sets { sampler_kinds.push(SamplerKind::SynFlagSets(layer_addr),) }

        let map = DataCellLayerMap::from_names(self.area_name, self.layer_name, self.thal);

        CorticalLayerSampler {
            sampler: CorticalSampler::new(self.area_name, sampler_kinds,
                self.idxs.clone(), self.thal, self.cortical_areas),
            layer_addr,
            map,
        }
    }
}