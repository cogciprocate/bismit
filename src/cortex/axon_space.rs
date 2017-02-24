use std::collections::HashMap;
use std::ops::Range;
use ocl::{ProQue, Buffer, EventList, Queue, Event};
use ocl::traits::MemLen;
use cmn::{self, CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, AxonDomainRoute, ExecutionCommand, CorticalBuffer,
    ThalamicTract};
use ::Thalamus;
use tract_terminal::{OclBufferSource, OclBufferTarget};
use cortex::{SensoryFilter};
#[cfg(test)] pub use self::tests::{AxonSpaceTest, AxnCoords};

const DISABLE_IO: bool = false;


#[derive(Debug)]
pub enum IoExeCmd {
    Read(usize),
    Write(usize),
    FilteredWrite(usize),
}

impl IoExeCmd {
    pub fn is_filtered_write(&self) -> bool {
        if let IoExeCmd::FilteredWrite(_) = *self { true } else { false }
    }
}


/// Information needed to read from and write to the thalamus for a layer
/// uniquely identified by `key`.
///
#[derive(Debug)]
pub struct IoInfo {
    key: LayerAddress,
    axn_range: Range<u32>,
    exe_cmd: IoExeCmd,
}

impl IoInfo {
    pub fn new(src_lyr_key: LayerAddress, axn_range: Range<u32>, exe_cmd: IoExeCmd) -> IoInfo {
        IoInfo {
            key: src_lyr_key,
            axn_range: axn_range,
            exe_cmd: exe_cmd,
        }
    }

    #[inline]
    pub fn filter_chain_idx(&self) -> Option<usize> {
        match self.exe_cmd {
            IoExeCmd::FilteredWrite(filter_chain_idx) => Some(filter_chain_idx),
            _ => None,
        }
    }

    #[inline] pub fn key(&self) -> &LayerAddress { &self.key }
    #[inline] pub fn axn_range(&self) -> Range<u32> { self.axn_range.clone() }
    #[inline] pub fn exe_cmd(&self) -> &IoExeCmd { &self.exe_cmd }
}


/// A group of `IoInfo` structs sharing a common set of `LayerTags`.
///
#[derive(Debug)]
pub struct IoInfoGroup {
    layers: Vec<IoInfo>,
}

impl IoInfoGroup {
    pub fn new(area_map: &AreaMap,
            group_route: AxonDomainRoute,
            tract_keys: Vec<(LayerAddress, Option<LayerAddress>)>,
            filter_chains: &Vec<(LayerAddress, Vec<SensoryFilter>)>,
            exe_graph: &mut ExecutionGraph,
            axn_states: &Buffer<u8>,
            thal: &Thalamus,
        ) -> IoInfoGroup
    {
        // Create a container for our i/o layer(s):
        let mut layers = Vec::<IoInfo>::with_capacity(tract_keys.len());

        for (lyr_addr, src_lyr_addr) in tract_keys.into_iter() {
            let (tract_key, io_cmd) = if let AxonDomainRoute::Output = group_route {
                /*=============================================================================
                ==================================== OUTPUT ===================================
                =============================================================================*/

                let lyr_slc_id_range = area_map.layers()
                    .layer_info(lyr_addr.layer_id()).expect("IoInfoCache::new(): \
                        Internal consistency error. Source layer address is invalid.")
                    .slc_range().expect("IoInfoCache::new(): \
                        Internal consistency error. Source layer has no slices.");

                let mut srcs: Vec<CorticalBuffer> = Vec::with_capacity(lyr_slc_id_range.len());
                let mut tars: Vec<ThalamicTract> = Vec::with_capacity(lyr_slc_id_range.len());

                for slc_id in lyr_slc_id_range.start..lyr_slc_id_range.end {
                    srcs.push(CorticalBuffer::axon_slice(axn_states, lyr_addr.area_id(), slc_id));
                    tars.push(ThalamicTract::axon_slice(lyr_addr.area_id(), slc_id));
                }

                let exe_cmd = ExecutionCommand::corticothalamic_read(srcs, tars);
                let io_cmd = IoExeCmd::Read(exe_graph.add_command(exe_cmd).expect("IoInfoGroup::new"));

                (lyr_addr, io_cmd)
            } else {
                /*=============================================================================
                ==================================== INPUT ====================================
                =============================================================================*/

                let src_lyr_addr = src_lyr_addr.clone().expect("IoInfoCache::new(): \
                    Internal consistency error. Source layer address for an input layer is empty.");

                // Determine the filter chain id:
                let filter_chain_idx = filter_chains.iter().position(
                    |&(ref addr, _)| {
                        src_lyr_addr == *addr
                    }
                );

                // If this is a filtered input layer, the first filter within
                // the filter chain will take care of the write command.
                // Otherwise, create one.
                let io_cmd = if let Some(idx) = filter_chain_idx {
                    IoExeCmd::FilteredWrite(idx)
                } else {
                    // Get source layer absolute slice id range:
                    let src_lyr_slc_id_range = thal.area_map(src_lyr_addr.area_id())
                        .and_then(|area| area.layer(src_lyr_addr.layer_id()))
                        .expect(&format!("IoInfoCache::new(): Unable to find source layer ({:?}) \
                            for i/o layer ({:?})", src_lyr_addr, lyr_addr))
                        .slc_range()
                        .expect(&format!("IoInfoCache::new(): Source layer ({:?}) for i/o layer ({:?}) \
                            has no slices (depth of zero).", src_lyr_addr, lyr_addr));

                    // Set write command source blocks:
                    let mut write_cmd_srcs: Vec<ThalamicTract> = Vec::with_capacity(src_lyr_slc_id_range.len());

                    for slc_id in src_lyr_slc_id_range.start..src_lyr_slc_id_range.end {
                        write_cmd_srcs.push(ThalamicTract::axon_slice(src_lyr_addr.area_id(), slc_id));
                    }

                    // Get target layer absolute slice id range:
                    let tar_lyr_slc_id_range = area_map.layers()
                        .layer_info(lyr_addr.layer_id()).expect("IoInfoCache::new(): \
                            Internal consistency error. Target layer address is invalid.")
                        .src_lyr(&src_lyr_addr).expect("IoInfoCache::new(): \
                            Internal consistency error. Target layer address not found within layer.")
                        .tar_slc_range();

                    // Set write command target blocks:
                    let mut write_cmd_tars: Vec<CorticalBuffer> = Vec::with_capacity(tar_lyr_slc_id_range.len());

                    for slc_id in tar_lyr_slc_id_range.start..tar_lyr_slc_id_range.end {
                        write_cmd_tars.push(CorticalBuffer::axon_slice(axn_states, lyr_addr.area_id(), slc_id))
                    }

                    let exe_cmd = ExecutionCommand::thalamocortical_write(write_cmd_srcs, write_cmd_tars);

                    IoExeCmd::Write(exe_graph.add_command(exe_cmd).expect("IoInfoGroup::new"))
                };

                (src_lyr_addr, io_cmd)
            };

            /*=============================================================================
            ===============================================================================
            =============================================================================*/

            let axn_range = area_map.lyr_axn_range(&lyr_addr, src_lyr_addr.as_ref()).expect(
                &format!("IoInfoCache::new(): Internal consistency error: \
                    lyr_addr: {:?}, src_lyr_addr: {:?}.", &lyr_addr, src_lyr_addr));

            let io_layer = IoInfo::new(tract_key, axn_range, io_cmd);
            layers.push(io_layer);
        }

        IoInfoGroup {
            layers: layers,
        }
    }

    #[inline] pub fn layers(&self) -> &[IoInfo] { self.layers.as_slice() }
    #[inline] pub fn layers_mut(&mut self) -> &mut [IoInfo] { self.layers.as_mut_slice() }
}



/// A collection of all of the information needed to read from and write to
/// i/o layers via the thalamus.
#[derive(Debug)]
pub struct IoInfoCache {
    groups: HashMap<AxonDomainRoute, (IoInfoGroup, EventList)>,
}

impl IoInfoCache {
    pub fn new(area_map: &AreaMap, filter_chains: &Vec<(LayerAddress, Vec<SensoryFilter>)>,
        exe_graph: &mut ExecutionGraph, axn_states: &Buffer<u8>, thal: &Thalamus) -> IoInfoCache
    {
        let group_route_list = [AxonDomainRoute::Input, AxonDomainRoute::Output];

        let mut groups = HashMap::with_capacity(group_route_list.len());

        // for &group_tags in group_tags_list.iter() {
        for group_route in group_route_list.into_iter() {
            // If the layer is an output layer, consult the layer info
            // directly. If an input layer, consult the layer source info for
            // that layer. Either way, construct a tuple of '(area_name,
            // src_lyr_tags, src_lyr_key)' which can be used to construct a
            // key to access the correct thalamic tract:
            let tract_keys: Vec<(LayerAddress, Option<LayerAddress>)> =
                if let AxonDomainRoute::Output = *group_route {
                    area_map.layers().iter()
                        .filter(|li| li.axn_domain().is_output())
                        .map(|li| {
                            let lyr_addr = LayerAddress::new(area_map.area_id(), li.layer_id());
                            (lyr_addr, None)
                        }).collect()
                } else {
                    // [NOTE]: Iterator flat mapping `sli` doesn't easily work
                    // because it needs `li` to build its `LayerAddress`:
                    let mut tract_keys = Vec::with_capacity(16);

                    for li in area_map.layers().iter() {
                        if li.axn_domain().is_input() {
                            let lyr_addr = LayerAddress::new(area_map.area_id(), li.layer_id());

                            for sli in li.sources().iter() {
                                tract_keys.push((lyr_addr, Some(sli.layer_addr().clone())));
                            }
                        }
                    }
                    tract_keys.shrink_to_fit();
                    tract_keys
                };

            // If there was nothing in the area map for this group's tags,
            // continue to the next set of tags in the `group_tags_list`:
            if tract_keys.len() != 0 {
                let io_lyr_grp = IoInfoGroup::new(area_map, group_route.clone(),
                    tract_keys, filter_chains, exe_graph, axn_states, thal);
                groups.insert(group_route.clone(), (io_lyr_grp, EventList::new()));
            }
        }

        groups.shrink_to_fit();

        IoInfoCache {
            groups: groups,
        }
    }

    pub fn group(&self, group_route: AxonDomainRoute) -> Option<(&[IoInfo], &EventList)> {
        self.groups.get(&group_route)
            .map(|&(ref lg, ref events)| (lg.layers(), events))
    }

    pub fn group_mut(&mut self, group_route: AxonDomainRoute) -> Option<(&mut [IoInfo], &mut EventList)> {
        self.groups.get_mut(&group_route)
            .map(|&mut (ref mut lg, ref mut events)| (lg.layers_mut(), events))
    }

    #[allow(dead_code)]
    pub fn group_info(&self, group_route: AxonDomainRoute) -> Option<&[IoInfo]> {
        self.groups.get(&group_route).map(|&(ref lg, _)| lg.layers())
    }

    #[allow(dead_code)]
    pub fn group_info_mut(&mut self, group_route: AxonDomainRoute) -> Option<&mut [IoInfo]> {
        self.groups.get_mut(&group_route).map(|&mut (ref mut lg, _)| lg.layers_mut())
    }

    #[allow(dead_code)]
    pub fn group_events(&self, group_route: AxonDomainRoute) -> Option<&EventList> {
        self.groups.get(&group_route).map(|&(_, ref events)| events)
    }

    #[allow(dead_code)]
    pub fn group_events_mut(&mut self, group_route: AxonDomainRoute) -> Option<&mut EventList> {
        self.groups.get_mut(&group_route).map(|&mut (_, ref mut events)| events)
    }
}



pub struct AxonSpace {
    area_id: usize,
    area_name: &'static str,
    states: Buffer<u8>,
    filter_chains: Vec<(LayerAddress, Vec<SensoryFilter>)>,
    io_info: IoInfoCache,
}

impl AxonSpace {
    pub fn new(area_map: &AreaMap, ocl_pq: &ProQue, write_queue: &Queue,
        exe_graph: &mut ExecutionGraph, thal: &Thalamus) -> CmnResult<AxonSpace>
    {
        println!("{mt}{mt}AXONS::NEW(): new axons with: total axons: {}",
            area_map.slices().to_len_padded(ocl_pq.max_wg_size().unwrap()), mt = cmn::MT);

        // let states = Buffer::<u8>::new(ocl_pq.queue().clone(), None, area_map.slices(), None).unwrap();
        let states = Buffer::<u8>::new(write_queue.clone(), None, area_map.slices(), None, None::<(_, Option<()>)>).unwrap();

        /*=============================================================================
        =================================== FILTERS ===================================
        =============================================================================*/

        let mut filter_chains = Vec::with_capacity(4);

        for &(ref track, ref tags, ref chain_scheme) in area_map.filter_chain_schemes() {
            let (src_lyr_info, _) = area_map.layers().src_layer_info_by_sig(&(track, tags).into())
                .expect(&format!("Unable to find a layer within the area map matching the axon \
                    domain (track: '{:?}', tags: '{:?}') specified by the filter chain scheme: '{:?}'.",
                    track, tags, chain_scheme));

            let mut layer_filters_rev: Vec<SensoryFilter> = Vec::with_capacity(4);

            // Create in reverse order so we can link each kernel to the next
            // filter in the chain:
            for (i, pf) in chain_scheme.iter().rev().enumerate() {
                let filter_idx = chain_scheme.len() - 1 - i;

                let filter = {
                    let filter_is_last = filter_idx == chain_scheme.len() - 1;

                    let (output_buffer, output_slc_range) = if filter_is_last {
                        debug_assert!(i == 0);
                        (&states,
                            src_lyr_info.tar_slc_range().clone())
                    } else {
                        debug_assert!(i > 0);
                        (layer_filters_rev[i - 1].input_buffer(),
                            0..(src_lyr_info.tar_slc_range().len() as u8))
                    };

                    let filter_is_first = filter_idx == 0;

                    let src_tract_info = if filter_is_first {
                        let src_lyr_addr = src_lyr_info.layer_addr();

                        // Get source layer absolute slice id range:
                        let src_lyr_slc_id_range = thal.area_map(src_lyr_addr.area_id())
                            .and_then(|area| area.layer(src_lyr_addr.layer_id()))
                                .expect(&format!("AxonSpace::new(): Unable to find source layer \
                                    ({:?}) for filter chain ({:?})", src_lyr_addr, chain_scheme))
                            .slc_range()
                                .expect(&format!("AxonSpace::new(): Source layer ({:?}) for \
                                    filter chain ({:?}) has no slices (depth of zero).",
                                    src_lyr_addr, chain_scheme))
                            .clone();

                        Some((src_lyr_addr.area_id(), src_lyr_slc_id_range))
                    } else {
                        None
                    };

                    SensoryFilter::new(
                        area_map.area_id(),
                        filter_idx,
                        chain_scheme.len(),
                        pf.filter_name(),
                        pf.cl_file_name(),
                        src_tract_info,
                        src_lyr_info.dims(),
                        output_buffer,
                        output_slc_range,
                        &ocl_pq,
                        &write_queue,
                        exe_graph)?
                };

                layer_filters_rev.push(filter);
            }

            // [DEBUG]:
            // println!("###### ADDING FILTER CHAIN: tags: {}", tags);
            let layer_filters = layer_filters_rev.into_iter().rev().collect();
            filter_chains.push((src_lyr_info.layer_addr().clone(), layer_filters));
        }

        filter_chains.shrink_to_fit();

        /*=============================================================================
        ===================================== I/O =====================================
        =============================================================================*/

        let io_info = IoInfoCache::new(&area_map, &filter_chains, exe_graph, &states, thal);

        Ok(AxonSpace {
            area_id: area_map.area_id(),
            area_name: area_map.area_name(),
            states: states,
            filter_chains: filter_chains,
            io_info: io_info,
        })
    }

    pub fn set_exe_order_intake(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        let (io_info_grp, _) = self.io_info.group(AxonDomainRoute::Input).unwrap();

        for io_lyr in io_info_grp {
            match *io_lyr.exe_cmd() {
                IoExeCmd::Write(cmd_idx) => {
                    exe_graph.order_next(cmd_idx)?;
                },
                IoExeCmd::FilteredWrite(filter_chain_idx) => {
                    if let Some(last_filter) = self.filter_chains[filter_chain_idx].1.first() {
                        last_filter.set_exe_order_write(exe_graph)?;
                    }
                }
                _ => panic!("AxonSpace::set_exe_order_intake: Internal error [0]."),
            }

            if let IoExeCmd::FilteredWrite(filter_chain_idx) = *io_lyr.exe_cmd() {
                for filter in self.filter_chains[filter_chain_idx].1.iter() {
                    filter.set_exe_order_cycle(exe_graph)?;
                }
            }
        }

        Ok(())
    }

    pub fn set_exe_order_output(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        let (io_info_grp, _) = self.io_info.group(AxonDomainRoute::Output).unwrap();

        for io_lyr in io_info_grp {
            match *io_lyr.exe_cmd() {
                IoExeCmd::Read(cmd_idx) => {
                    exe_graph.order_next(cmd_idx)?;
                },
                _ => panic!("AxonSpace::set_exe_order_output: Internal error."),
            }
        }

        Ok(())
    }

    /// Reads input from thalamus and writes to axon space.
    ///
    // [TODO]: Store thal tract index instead of using (LayerAddress) key.
    //
    pub fn intake(&mut self, thal: &mut Thalamus, exe_graph: &mut ExecutionGraph,
            bypass_filters: bool) -> CmnResult<()>
    {
        if let Some((io_lyrs, mut new_events)) = self.io_info.group_mut(AxonDomainRoute::Input) {
            for io_lyr in io_lyrs.iter_mut() {
                let tract_source = thal.tract_terminal_source(io_lyr.key())?;

                if !DISABLE_IO && !bypass_filters && io_lyr.exe_cmd().is_filtered_write() {
                    let filter_chain_idx = io_lyr.filter_chain_idx().unwrap();
                    let filter_chain = &mut self.filter_chains[filter_chain_idx].1;
                    // let mut filter_event = filter_chain[0].write(tract_source)?;
                    filter_chain[0].write(tract_source, exe_graph)?;

                    for filter in filter_chain.iter() {
                        filter.cycle(exe_graph)?;
                    }
                } else {
                    let axn_range = io_lyr.axn_range();
                    let area_name = self.area_name;

                    if let &IoExeCmd::Write(cmd_idx) = io_lyr.exe_cmd() {
                        let event = if DISABLE_IO {
                            Event::empty()
                        } else {
                            OclBufferTarget::new(&self.states, axn_range, tract_source.dims().clone(),
                                Some(&mut new_events), false)
                            .map_err(|err|
                                err.prepend(&format!("CorticalArea::intake():: \
                                Source tract length must be equal to the target axon range length \
                                (area: '{}', layer_addr: '{:?}'): ", area_name, io_lyr.key())))?
                            // .copy_from_slice_buffer(tract_source)?;
                            .copy_from_slice_buffer_v2(tract_source, Some(exe_graph.get_req_events(cmd_idx)?))?
                        };

                        exe_graph.set_cmd_event(cmd_idx, event)?;
                    } else {
                        panic!("CorticalArea::intake():: Invalid 'IoExeCmd' type: {:?}", io_lyr.exe_cmd());
                    }
                }
            }
        }
        Ok(())
    }

    /// Reads output from axon space and writes to thalamus.
    ///
    // [TODO]: Store thal tract index instead of using (LayerAddress) key.
    //
    pub fn output(&self, read_queue: &Queue, thal: &mut Thalamus, exe_graph: &mut ExecutionGraph)
            -> CmnResult<()>
    {
        if let Some((io_lyrs, wait_events)) = self.io_info.group(AxonDomainRoute::Output) {
            for io_lyr in io_lyrs.iter() {
                if let &IoExeCmd::Read(cmd_idx) = io_lyr.exe_cmd() {
                    let event = if DISABLE_IO {
                        Event::empty()
                    } else {
                        let mut target = thal.tract_terminal_target(io_lyr.key())?;

                        let source = OclBufferSource::new(&self.states, io_lyr.axn_range(),
                                target.dims().clone(), Some(wait_events))
                            .map_err(|err| err.prepend(&format!("CorticalArea::output(): \
                                Target tract length must be equal to the source axon range length \
                                (area: '{}', layer_addr: '{:?}'): ", self.area_name, io_lyr.key()))
                            )?;

                        // let event = target.copy_from_ocl_buffer_v2(source,
                        //     Some(&exe_graph.get_req_events(cmd_idx)?), None)?;
                        target.copy_from_ocl_buffer_v2(source,
                            Some(exe_graph.get_req_events(cmd_idx)?), Some(read_queue))?
                    };

                    exe_graph.set_cmd_event(cmd_idx, event)?;
                } else {
                    panic!("CorticalArea::output():: Invalid 'IoExeCmd' type: {:?}", io_lyr.exe_cmd());
                }
            }
        }
        Ok(())
    }

    pub fn states(&self) -> &Buffer<u8> { &self.states }
    pub fn area_id(&self) -> usize { self.area_id }
    pub fn filter_chains(&self) -> &[(LayerAddress, Vec<SensoryFilter>)] { self.filter_chains.as_slice() }
    pub fn filter_chains_mut(&mut self) -> &mut [(LayerAddress, Vec<SensoryFilter>)] {
        self.filter_chains.as_mut_slice() }
    pub fn io_info(&self) -> &IoInfoCache { &self.io_info }
    pub fn io_info_mut(&mut self) -> &mut IoInfoCache { &mut self.io_info }
}



#[cfg(test)]
pub mod tests {
    #![allow(dead_code)]
    use super::{AxonSpace};
    use map::{AreaMap, AreaMapTest};
    use cmn::{CelCoords};

    pub trait AxonSpaceTest {
        fn axn_state(&self, idx: usize) -> u8;
        fn write_to_axon(&mut self, val: u8, idx: u32);
    }

    impl AxonSpaceTest for AxonSpace {
        fn axn_state(&self, idx: usize) -> u8 {
            self.states.default_queue().unwrap().finish().unwrap();
            let mut sdr = vec![0u8];
            self.states.cmd().read(&mut sdr).offset(idx).enq().unwrap();
            sdr[0]
        }

        fn write_to_axon(&mut self, val: u8, idx: u32) {
            self.states.default_queue().unwrap().finish().unwrap();
            let sdr = vec![val];
            self.states.cmd().write(&sdr).offset(idx as usize).enq().unwrap();
        }
    }

    pub struct AxnCoords {
        idx: u32,
        slc_id: u8,
        v_id: u32,
        u_id: u32,
    }

    impl AxnCoords {
        pub fn new(slc_id: u8, v_id: u32, u_id: u32, area_map: &AreaMap
            ) -> Result<AxnCoords, &'static str>
        {
            match area_map.axn_idx(slc_id, v_id, 0, u_id, 0) {
                Ok(idx) => Ok(AxnCoords { idx: idx, slc_id: slc_id, v_id: v_id, u_id: u_id }),
                Err(e) => Err(e),
            }
        }

        pub fn from_cel_coords(cel_base_axn_slc: u8, cel_coords: &CelCoords, area_map: &AreaMap
            ) -> Result<AxnCoords, &'static str>
        {
            AxnCoords::new(cel_base_axn_slc, cel_coords.v_id,
                cel_coords.u_id, area_map)
        }

        pub fn idx(&self) -> u32 {
            self.idx
        }
    }
}
