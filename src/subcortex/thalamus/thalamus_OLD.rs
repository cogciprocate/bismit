//! A central relay and processing (encoding) station for all I/O between
//! cortical areas and nuclei of all types.
//!
//! Serves roles roughly analogous to those of the thalamus, internal capsule,
//! and any other cortico-cortical, cortico-subcortical, and
//! subcortico-cortical (including basal, thalamic, and spinal) axon tracts.
//! These roles may be refactored, reconfigured, or reassigned in the future.
//!
//! All storage is currently located in host memory but will eventually be a
//! hybrid host-device storage system, keeping data as close as possible to
//! it's destinations (whether those be host or device side).
//!
//!


// #![allow(dead_code, unused_imports)]

use std::ops::Range;
use std::collections::HashMap;
use cmn::{self, CmnError, CmnResult, TractDims, TractFrame, TractFrameMut, CorticalDims, MapStore};
use map::{AreaMap, LayerMapKind, LayerAddress};
use ocl::{Context, EventList};
use cortex::CorticalAreas;
use map::{AreaSchemeList, LayerMapSchemeList, /*ExecutionGraph*/};
use ::{ExternalPathway, ExternalPathwayFrame};
use tract_terminal::{SliceBufferTarget, SliceBufferSource};



/// Specifies whether or not the frame buffer for a source exists within the
/// thalamic tract or an external source itself.
#[derive(Debug)]
#[allow(dead_code)]
enum TractAreaBufferKind {
    Ocl,
    Vec,
}


#[derive(Debug)]
struct TractArea {
    src_lyr_addr: LayerAddress,
    range: Range<usize>,
    events: EventList,
    dims: TractDims,
    kind: TractAreaBufferKind,
}

impl TractArea {
    fn new(src_lyr_addr: LayerAddress, range: Range<usize>,
                dims: TractDims, kind: TractAreaBufferKind) -> TractArea {
        // println!("###### TractArea::new(): Adding area with: range: {:?}, dims: {:?}", &range, &dims);
        assert!(range.len() == dims.to_len());
        TractArea {
            src_lyr_addr: src_lyr_addr,
            range: range,
            events: EventList::new(),
            dims: dims,
            kind: kind,
        }
    }

    fn range(&self) -> &Range<usize> {
        &self.range
    }

    fn dims(&self) -> &TractDims {
        &self.dims
    }

    fn events(&self) -> &EventList {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventList {
        &mut self.events
    }
}



// [FIXME]: REPLACE STRING HASH KEY. SEE TOP OF FILE.
struct TractAreaCache {
    tract_areas: Vec<TractArea>,
    index: HashMap<LayerAddress, usize>,
}

impl TractAreaCache {
    fn new() -> TractAreaCache {
        TractAreaCache {
            tract_areas: Vec::with_capacity(32),
            index: HashMap::with_capacity(48),
        }
    }

    fn insert(&mut self, src_lyr_addr: LayerAddress, tract_area: TractArea) {
        self.tract_areas.push(tract_area);

        self.index.insert(src_lyr_addr.clone(), (self.tract_areas.len() - 1))
            .map(|_| panic!("TractAreaCache::insert(): Multiple i/o layers using the same layer \
                tags and id found. I/O layers with the same tags must have unique ids. \
                (layer address: {:?})", src_lyr_addr));
    }

    // [NOTE]: Must be `&mut self` because searching saves cache info.
    fn get(&mut self, key: &LayerAddress) -> Result<&TractArea, CmnError> {
        match self.area_search(key) {
            Ok(idx) => self.tract_areas.get(idx).ok_or(CmnError::new(format!("Index '{}' not found for \
                '{:?}'.", idx, key))),
            Err(err) => Err(err),
        }
    }

    fn get_mut(&mut self, key: &LayerAddress) -> Result<&mut TractArea, CmnError> {
        match self.area_search(key) {
            Ok(idx) => self.tract_areas.get_mut(idx).ok_or(CmnError::new(format!("Index '{}' not found for \
                '{:?}'.", idx, key))),
            Err(err) => {
                Err(err)
            },
        }
    }

    // [NOTE]: Must be `&mut self` because searching saves cache info.
    fn area_search(&mut self, key: &LayerAddress) -> Result<usize, CmnError> {
        // println!("TractAreaCache::area_search(): Searching for area: {}, tags: {:?}. ALL: {:?}",
        //     src_area_name, layer_tags, self.tract_areas);
        let area_id = self.index.get(key).map(|&idx| idx);
        // println!("   area_id: {:?}", area_id);

        match area_id {
            Some(idx) => return Ok(idx),
            None => {
                let matching_areas: Vec<usize> = self.tract_areas.iter().enumerate()
                    .filter(|&(_, ta)| ta.src_lyr_addr == *key)
                    .map(|(i, _)| i)
                    .collect();

                match matching_areas.len() {
                    0 => return Err(CmnError::new(format!("No tract areas found with \
                        layer address: '{:?}'.", key))),
                    1 => {
                        self.index.insert(key.clone(), matching_areas[0]);
                        return Ok(matching_areas[0]);
                    },
                    _ => Err(CmnError::new(format!("Multiple tract areas found with \
                        layer address: {:?}.", key))),
                }
            }
        }
    }
}



// THALAMICTRACT: A buffer for I/O between areas. Effectively analogous to the internal capsule.
pub struct ThalamicTract {
    vec_buffer: Vec<u8>,
    tract_areas: TractAreaCache,
    ttl_len: usize,
}

impl ThalamicTract {
    fn new() -> ThalamicTract {
        let vec_buffer = Vec::with_capacity(0);
        let tract_areas = TractAreaCache::new();

        ThalamicTract {
            vec_buffer: vec_buffer,
            tract_areas: tract_areas,
            ttl_len: 0,
        }
    }

    fn add_area(&mut self, src_lyr_addr: LayerAddress, layer_dims: CorticalDims) {
        // println!("###### ThalamicTract::new(): Adding tract for area: {}, tags: {}, layer_dims: {:?}",
        //     src_area_name, layer_tags, layer_dims);
        let tract_dims: TractDims = layer_dims.into();
        let len = tract_dims.to_len();
        let new_area = TractArea::new(src_lyr_addr.clone(),
            self.ttl_len..(self.ttl_len + len), tract_dims, TractAreaBufferKind::Vec);

        self.tract_areas.insert(src_lyr_addr, new_area);
        self.ttl_len += len;
    }

    fn init(mut self) -> ThalamicTract {
        self.vec_buffer.resize(self.ttl_len, 0);
        // println!("{}THALAMICTRACT::INIT(): tract_areas: {:?}", cmn::MT, self.tract_areas);
        self
    }

    fn frame<'t>(&'t mut self, key: &LayerAddress)
            -> Result<(&EventList, TractFrame<'t>), CmnError>
    {
        let ta = try!(self.tract_areas.get(key));
        let range = ta.range().clone();
        let tract = TractFrame::new(&self.vec_buffer[range], ta.dims());
        let events = ta.events();

        Ok((events, tract))
    }

    fn frame_mut<'t>(&'t mut self, key: &LayerAddress)
            -> Result<(TractFrameMut<'t>, &mut EventList), CmnError>
    {
        let ta = try!(self.tract_areas.get_mut(key));
        let range = ta.range().clone();
        let tract = TractFrameMut::new(&mut self.vec_buffer[range], ta.dims());
        let events = ta.events_mut();

        Ok((tract, events))
    }

    fn terminal_source<'t>(&'t mut self, key: &LayerAddress)
            -> CmnResult<(SliceBufferSource<'t>)>
    {
        let ta = try!(self.tract_areas.get(key));
        let range = ta.range().clone();
        let dims = ta.dims().clone();
        let events = ta.events();
        let terminal = SliceBufferSource::new(&self.vec_buffer[range], dims, Some(events));

        terminal
    }

    fn terminal_target<'t>(&'t mut self, key: &LayerAddress)
            -> CmnResult<(SliceBufferTarget<'t>)>
    {
        let ta = try!(self.tract_areas.get_mut(key));
        let range = ta.range().clone();
        let dims = ta.dims().clone();
        let events = ta.events_mut();
        let terminal = SliceBufferTarget::new(&mut self.vec_buffer[range], dims, Some(events), false);

        terminal
    }
}



// THALAMUS:
// - Input/Output is from a CorticalArea's point of view
//   - input: to layer / area
//   - output: from layer / area
pub struct Thalamus {
    tract: ThalamicTract,
    external_pathways: MapStore<String, (ExternalPathway, Vec<LayerAddress>)>,
    area_maps: MapStore<String, AreaMap>,
}

impl Thalamus {
    pub fn new(layer_map_sl: LayerMapSchemeList, mut area_sl: AreaSchemeList,
                ocl_context: &Context) -> CmnResult<Thalamus>
    {
        // [FIXME]:
        let _ = ocl_context;

        area_sl.freeze();
        let area_sl = area_sl;
        let mut tract = ThalamicTract::new();
        let mut external_pathways = MapStore::with_capacity(area_sl.areas().len());
        let mut area_maps = MapStore::with_capacity(area_sl.areas().len());

        /*=============================================================================
        ============================ THALAMIC (INPUT) AREAS ===========================
        =============================================================================*/
        for pa in area_sl.areas().iter().filter(|pa|
                    layer_map_sl[pa.layer_map_name()].kind() == &LayerMapKind::Subcortical)
        {
            let es = try!(ExternalPathway::new(pa, &layer_map_sl[pa.layer_map_name()]));
            let addrs = es.layer_addrs();
            external_pathways.insert(es.area_name().to_owned(), (es, addrs))
                .map(|es_tup| panic!("Duplicate 'ExternalPathway' keys: [\"{}\"]. \
                    Only one external (thalamic) input source per area is allowed.",
                    es_tup.0.area_name()));
        }

        /*=============================================================================
        =================================== ALL AREAS =================================
        =============================================================================*/
        for (area_id, area_s) in area_sl.areas().iter().enumerate() {
            assert!(area_s.area_id() == area_id);
            let area_map = AreaMap::new(area_id, area_s, &layer_map_sl, &area_sl, &external_pathways)?;

            println!("{mt}{mt}THALAMUS::NEW(): Area: \"{}\", Output layers (tracts): ",
                area_s.name(), mt = cmn::MT);

            {
                let output_layers = area_map.layers().iter()
                    .filter(|li| li.axn_domain().is_output()).collect::<Vec<_>>();

                for layer in output_layers.iter() {
                    // println!("###### Thalamus::new(): Processing layer {}.", layer.name());

                    // If the layer is thalamic it will have an irregular size
                    // which will need to be reflected on its tract size.
                    let layer_dims = match layer.irregular_layer_dims() {
                        Some(dims) => dims.clone(),
                        None => area_s.dims().clone_with_depth(layer.depth()),
                    };

                    println!("{mt}{mt}{mt}'{}': tags: {}, slc_range: {:?}, map_kind: {:?}, \
                        axn_kind: {:?}", layer.name(), layer.layer_tags(), layer.slc_range(),
                        layer.layer_map_kind(), layer.axn_topology(), mt = cmn::MT);

                    tract.add_area(LayerAddress::new(area_s.area_id(), layer.layer_id()),
                        layer_dims);
                }

                assert!(output_layers.len() > 0, "Areas must have at least one output layer.");
            }

            area_maps.insert(area_s.name().to_owned(), area_map);
            assert!(area_maps[area_id].area_id() == area_id);
        }

        Ok(Thalamus {
            tract: tract.init(),
            external_pathways: external_pathways,
            area_maps: area_maps,
        })
    }

    // Multiple source output areas disabled.
    pub fn cycle_external_pathways(&mut self, _: &mut CorticalAreas) {
        for &mut (ref mut src_ext_path, ref layer_addr_list) in self.external_pathways.values_mut().iter_mut() {
            src_ext_path.cycle_next();

            for layer_addr in layer_addr_list.iter() {
                let (tract_frame, events) = self.tract.frame_mut(layer_addr)
                    .expect("Thalamus::cycle_external_pathways()");

                src_ext_path.write_into(layer_addr, tract_frame, events)
            }
        }
    }

    pub fn tract_frame<'t>(&'t mut self, key: &LayerAddress)
            -> Result<(&EventList, TractFrame<'t>), CmnError>
    {
        self.tract.frame(key)
    }

    pub fn tract_frame_mut<'t>(&'t mut self, key: &LayerAddress)
            -> Result<(TractFrameMut<'t>, &mut EventList), CmnError>
    {
        self.tract.frame_mut(key)
    }

    pub fn tract_terminal_target<'t>(&'t mut self, key: &LayerAddress)
            -> CmnResult<(SliceBufferTarget<'t>)>
    {
        self.tract.terminal_target(key)
    }

    pub fn tract_terminal_source<'t>(&'t mut self, key: &LayerAddress)
            -> CmnResult<(SliceBufferSource<'t>)>
    {
        self.tract.terminal_source(key)
    }

    // pub fn area_maps(&self) -> &MapStore<String, AreaMap> {
    pub fn area_maps(&self) -> &[AreaMap] {
         &self.area_maps.values()
    }

    pub fn area_map(&self, area_id: usize) -> Option<&AreaMap> {
        self.area_maps.by_index(area_id)
    }

    pub fn area_map_by_name(&self, area_name: &str) -> Option<&AreaMap> {
        self.area_maps.by_key(area_name)
    }

    pub fn ext_pathway_idx(&self, pathway_name: &String) -> CmnResult<usize> {
        match self.external_pathways.indices().get(pathway_name) {
            Some(&idx) => Ok(idx),
            None => CmnError::err(format!("Thalamus::ext_pathway_idx(): \
                No external pathway found named: '{}'.", pathway_name)),
        }
    }

    pub fn ext_pathway(&mut self, pathway_idx: usize) -> CmnResult<&mut ExternalPathway> {
        let pathway = try!(self.external_pathways.by_index_mut(pathway_idx).ok_or(
            CmnError::new(format!("Thalamus::ext_pathway_frame(): Invalid pathway index: '{}'.",
            pathway_idx))));

        Ok(&mut pathway.0)
    }

    pub fn ext_pathway_frame(&mut self, pathway_idx: usize) -> CmnResult<ExternalPathwayFrame> {
        let pathway = try!(self.ext_pathway(pathway_idx));
        pathway.ext_frame_mut()
    }
}


#[cfg(test)]
pub mod tests {

}
