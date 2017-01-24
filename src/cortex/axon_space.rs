use std::collections::HashMap;
use std::ops::Range;
use ocl::{ProQue, Buffer, EventList};
use ocl::traits::MemLen;
use cmn::{self, CmnResult};
use map::{AreaMap, LayerAddress, ExecutionGraph, AxonDomainRoute, ExecutionCommand, CorticalBuffer,
    ThalamicTract};
use ::Thalamus;
use tract_terminal::{OclBufferSource, OclBufferTarget};
use cortex::{SensoryFilter};
#[cfg(test)] pub use self::tests::{AxonSpaceTest, AxnCoords};


#[derive(Debug)]
pub enum IoExeCmd {
    Read(usize),
    Write(usize),
}


/// Information needed to read from and write to the thalamus for a layer
/// uniquely identified by `key`.
///
#[derive(Debug)]
pub struct IoInfo {
    key: LayerAddress,
    axn_range: Range<u32>,
    filter_chain_idx: Option<usize>,
    exe_cmd: IoExeCmd,
}

impl IoInfo {
    pub fn new(src_lyr_key: LayerAddress, axn_range: Range<u32>,
            filter_chain_idx: Option<usize>, exe_cmd: IoExeCmd) -> IoInfo
    {
        IoInfo {
            key: src_lyr_key,
            axn_range: axn_range,
            filter_chain_idx: filter_chain_idx,
            exe_cmd: exe_cmd,
        }
    }

    #[inline] pub fn key(&self) -> &LayerAddress { &self.key }
    #[inline] pub fn axn_range(&self) -> Range<u32> { self.axn_range.clone() }
    #[inline] pub fn filter_chain_idx(&self) -> &Option<usize> { &self.filter_chain_idx }
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
            // group_tags: LayerTags,
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
            let (tract_key, filter_chain_idx, io_cmd) = if let AxonDomainRoute::Output = group_route {

                let lyr_slc_id_range = area_map.layers()
                    .layer_info(lyr_addr.layer_id()).expect("IoInfoCache::new(): \
                        Internal consistency error. Source layer address is invalid.")
                    .slc_range().expect("IoInfoCache::new(): \
                        Internal consistency error. Source layer has no slices.");

                let mut srcs: Vec<CorticalBuffer> = Vec::with_capacity(lyr_slc_id_range.len());
                let mut tars: Vec<ThalamicTract> = Vec::with_capacity(lyr_slc_id_range.len());

                for slc_id in lyr_slc_id_range.start..lyr_slc_id_range.end {
                    srcs.push(CorticalBuffer::axon_slice(axn_states, lyr_addr, slc_id));
                    tars.push(ThalamicTract::axon_slice(lyr_addr, slc_id));
                }

                let exe_cmd = ExecutionCommand::corticothalamic_read(srcs, tars);

                // // Create a read I/O execution command:
                // let exe_cmd = ExecutionCommand::corticothalamic_read(
                //     CorticalBuffer::axon_layer(axn_states, lyr_addr),
                //     ThalamicTract::layer(lyr_addr, None)
                // );

                let io_cmd = IoExeCmd::Read(exe_graph.add_command(exe_cmd));

                (lyr_addr, None, io_cmd)
            } else {
                let src_lyr_addr = src_lyr_addr.clone().expect("IoInfoCache::new(): \
                    Internal consistency error. Source layer address for an input layer is empty.");

                // Determine the filter chain id:
                let filter_chain_idx = filter_chains.iter().position(
                    |&(ref addr, _)| {
                        src_lyr_addr == *addr
                    }
                );

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
                    write_cmd_srcs.push(ThalamicTract::axon_slice(src_lyr_addr, slc_id));
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
                    write_cmd_tars.push(CorticalBuffer::axon_slice(axn_states, lyr_addr, slc_id))
                }


                let exe_cmd = ExecutionCommand::thalamocortical_write(write_cmd_srcs, write_cmd_tars);

                let io_cmd = IoExeCmd::Write(exe_graph.add_command(exe_cmd));

                (src_lyr_addr, filter_chain_idx, io_cmd)
            };

            let axn_range = area_map.lyr_axn_range(&lyr_addr, src_lyr_addr.as_ref()).expect(
                &format!("IoInfoCache::new(): Internal consistency error: \
                    lyr_addr: {:?}, src_lyr_addr: {:?}.", &lyr_addr, src_lyr_addr));

            let io_layer = IoInfo::new(tract_key, axn_range, filter_chain_idx, io_cmd);
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
                            let lyr_addr = LayerAddress::new(li.layer_id(), area_map.area_id());
                            (lyr_addr, None)
                        }).collect()
                } else {
                    // [NOTE]: Iterator flat mapping `sli` doesn't easily work
                    // because it needs `li` to build its `LayerAddress`:
                    let mut tract_keys = Vec::with_capacity(16);

                    for li in area_map.layers().iter() {
                        if li.axn_domain().is_input() {
                            let lyr_addr = LayerAddress::new(li.layer_id(), area_map.area_id());

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
    pub states: Buffer<u8>,
    filter_chains: Vec<(LayerAddress, Vec<SensoryFilter>)>,
    io_info: IoInfoCache,
}

impl AxonSpace {
    pub fn new(area_map: &AreaMap, ocl_pq: &ProQue, exe_graph: &mut ExecutionGraph,
            thal: &Thalamus) -> AxonSpace
    {
        println!("{mt}{mt}AXONS::NEW(): new axons with: total axons: {}",
            area_map.slices().to_len_padded(ocl_pq.max_wg_size().unwrap()), mt = cmn::MT);

        let states = Buffer::<u8>::new(ocl_pq.queue().clone(), None, area_map.slices(), None).unwrap();

        /*=============================================================================
        =================================== FILTERS ===================================
        =============================================================================*/

        let mut filter_chains = Vec::with_capacity(4);

        for &(ref track, ref tags, ref chain_scheme) in area_map.filter_chain_schemes() {
            let (src_layer, _) = area_map.layers().src_layer_info_by_sig(&(track, tags).into())
                .expect(&format!("Unable to find a layer within the area map matching the axon \
                    domain (track: '{:?}', tags: '{:?}') specified by the filter chain scheme: '{:?}'.",
                    track, tags, chain_scheme));

            let mut layer_filters = Vec::with_capacity(4);

            for pf in chain_scheme.iter() {
                layer_filters.push(SensoryFilter::new(
                    pf.filter_name(),
                    pf.cl_file_name(),
                    src_layer,
                    &states,
                    &ocl_pq)
                );
            }

            // [DEBUG]:
            // println!("###### ADDING FILTER CHAIN: tags: {}", tags);
            layer_filters.shrink_to_fit();
            filter_chains.push((src_layer.layer_addr().clone(), layer_filters));
        }

        filter_chains.shrink_to_fit();

        /*=============================================================================
        ===================================== I/O =====================================
        =============================================================================*/

        let io_info = IoInfoCache::new(&area_map, &filter_chains, exe_graph, &states, thal);

        AxonSpace {
            area_id: area_map.area_id(),
            area_name: area_map.area_name(),
            states: states,
            filter_chains: filter_chains,
            io_info: io_info,
        }
    }

    pub fn set_exe_order_input(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        let (io_info_grp, _) = self.io_info.group(AxonDomainRoute::Input).unwrap();

        for io_info in io_info_grp {
            match *io_info.exe_cmd() {
                IoExeCmd::Write(cmd_idx) => {
                    exe_graph.order_next(cmd_idx)?;
                },
                _ => panic!("AxonSpace::set_exe_order_input: Internal error."),
            }
        }

        Ok(())
    }

    pub fn set_exe_order_output(&self, exe_graph: &mut ExecutionGraph) -> CmnResult<()> {
        let (io_info_grp, _) = self.io_info.group(AxonDomainRoute::Output).unwrap();

        for io_info in io_info_grp {
            match *io_info.exe_cmd() {
                IoExeCmd::Read(cmd_idx) => {
                    exe_graph.order_next(cmd_idx)?;
                },
                _ => panic!("AxonSpace::set_exe_order_output: Internal error."),
            }
        }

        Ok(())
    }

    /// Reads input from thalamus and writes to axon space.
    pub fn intake(&mut self, thal: &mut Thalamus, bypass_filters: bool) -> CmnResult<()> {
        if let Some((src_lyrs, mut new_events)) = self.io_info.group_mut(AxonDomainRoute::Input) {
            for src_lyr in src_lyrs.iter_mut() {
                let tract_source = thal.tract_terminal_source(src_lyr.key())?;

                if !self.filter_chains.is_empty() && !bypass_filters &&
                        src_lyr.filter_chain_idx().is_some()
                {
                    if let &Some(filter_chain_idx) = src_lyr.filter_chain_idx() {
                        let (_, ref mut filter_chain) = self.filter_chains[filter_chain_idx];
                        let mut filter_event = filter_chain[0].write(tract_source)?;

                        for filter in filter_chain.iter() {
                            filter_event = filter.cycle(&filter_event);
                        }
                    } else {
                        unreachable!();
                    }
                } else {
                    let axn_range = src_lyr.axn_range();
                    let area_name = self.area_name;

                    OclBufferTarget::new(&self.states, axn_range, tract_source.dims().clone(),
                            Some(&mut new_events), false)
                        .map_err(|err|
                            err.prepend(&format!("CorticalArea::intake():: \
                            Source tract length must be equal to the target axon range length \
                            (area: '{}', layer_addr: '{:?}'): ", area_name, src_lyr.key())))?
                        .copy_from_slice_buffer(tract_source)?;
                }
            }
        }
        Ok(())
    }

    /// Reads output from axon space and writes to thalamus.
    pub fn output(&self, thal: &mut Thalamus) -> CmnResult<()> {
        if let Some((src_lyrs, wait_events)) = self.io_info.group(AxonDomainRoute::Output) {
            for src_lyr in src_lyrs.iter() {
                let mut target = thal.tract_terminal_target(src_lyr.key())?;

                let source = OclBufferSource::new(&self.states, src_lyr.axn_range(),
                        target.dims().clone(), Some(wait_events))
                    .map_err(|err| err.prepend(&format!("CorticalArea::output(): \
                        Target tract length must be equal to the source axon range length \
                        (area: '{}', layer_addr: '{:?}'): ", self.area_name, src_lyr.key()))
                    )?;

                target.copy_from_ocl_buffer(source)?;
            }
        }
        Ok(())
    }

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
            let mut sdr = vec![0u8];
            self.states.cmd().read(&mut sdr).offset(idx).enq().unwrap();
            sdr[0]
        }

        fn write_to_axon(&mut self, val: u8, idx: u32) {
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
