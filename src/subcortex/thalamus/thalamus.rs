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
//! [FIXME]: Every single hash lookup requires a heap allocation (`String`).
//! This is obviously very wasteful and is temporary until one of the
//! following can be implemented:
//! - Convert area names to indexes (preferred). Use a table stored
//!     somewhere to look up names for display.
//!     - Use a hashmap to resolve area ids to area names in the event of an
//!       error. Store this hashmap both on `TractAreaCache`. Each
//!       `CorticalArea` will, of course, also have a copy of its own area id.
//!     - Possibly have `AreaSchemeList` initially create the id list.
//!     - [UPDATE]: Use the new `cmn::MapStore`.
//! - Precompute hash.
//! - Store strings in a separate vector (stored in cortex) and put a
//!   reference in the key.
//!     - Will need some sort of global lookup system. Bleh.
//! - Think of something else (but top opt looks good).
//!
//!

#![allow(dead_code, unused_imports)]

use std::ops::Range;
use std::collections::HashMap;
// use std::collections::hash_map::Entry;

use cmn::{self, CmnError, CmnResult, TractDims, TractFrame, TractFrameMut, CorticalDims, MapStore};
use map::{self, AreaMap, LayerTags, LayerMapKind};
use ocl::{Context, EventList, Buffer};
use cortex::CorticalAreas;
use map::{AreaSchemeList, LayerMapSchemeList};
use thalamus::{ExternalPathway, ExternalPathwayFrame};
use tract_terminal::{SliceBufferTarget, SliceBufferSource};



/// Specifies whether or not the frame buffer for a source exists within the
/// thalamic tract or an external source itself.
#[derive(Debug)]
enum TractAreaBufferKind {
    Ocl,
    Vec,
}


#[derive(Debug)]
struct TractArea {
    src_area_name: String,
    layer_tags: LayerTags,
    range: Range<usize>,
    events: EventList,
    dims: TractDims,
    kind: TractAreaBufferKind,
}

impl TractArea {
    fn new(src_area_name: String, layer_tags: LayerTags, range: Range<usize>,
                dims: TractDims, kind: TractAreaBufferKind) -> TractArea {
        // println!("###### TractArea::new(): Adding area with: range: {:?}, dims: {:?}", &range, &dims);
        assert!(range.len() == dims.to_len());
        TractArea {
            src_area_name: src_area_name,
            layer_tags: layer_tags,
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
    areas: Vec<TractArea>,
    index: HashMap<(String, LayerTags), usize>,
}

impl TractAreaCache {
    fn new() -> TractAreaCache {
        TractAreaCache {
            areas: Vec::with_capacity(32),
            index: HashMap::with_capacity(48),
        }
    }

    fn insert(&mut self, src_area_name: String, layer_tags: LayerTags, tract_area: TractArea) {
        self.areas.push(tract_area);

        self.index.insert((src_area_name.clone(), layer_tags), (self.areas.len() - 1))
            .map(|_| panic!("TractAreaCache::insert(): Multiple i/o layers using the same layer \
                tags and id found. I/O layers with the same tags must have unique ids. \
                (area: \"{}\", tags: {})", src_area_name, layer_tags));
    }

    // [NOTE]: Must be `&mut self` because searching saves cache info.
    fn get(&mut self, key: &(String, LayerTags)) -> Result<&TractArea, CmnError> {
        match self.area_search(key) {
            Ok(idx) => self.areas.get(idx).ok_or(CmnError::new(format!("Index '{}' not found for '{}' \
                with tags {}", idx, key.0, key.1))),

            Err(err) => Err(err),
        }
    }

    fn get_mut(&mut self, key: &(String, LayerTags)) -> Result<&mut TractArea, CmnError> {
        match self.area_search(key) {
            Ok(idx) => self.areas.get_mut(idx).ok_or(CmnError::new(format!("Index '{}' not \
                found for '{}' with tags {}", idx, key.0, key.1))),

            Err(err) => {
                Err(err)
            },
        }
    }

    // [NOTE]: Must be `&mut self` because searching saves cache info.
    fn area_search(&mut self, key: &(String, LayerTags)) -> Result<usize, CmnError> {
        // println!("TractAreaCache::area_search(): Searching for area: {}, tags: {:?}. ALL: {:?}",
        //     src_area_name, layer_tags, self.areas);
        let area_id = self.index.get(key).map(|&idx| idx);
        // println!("   area_id: {:?}", area_id);

        let mut matching_areas: Vec<usize> = Vec::with_capacity(4);

        match area_id {
            Some(idx) => return Ok(idx),
            None => {
                for i in 0..self.areas.len() {
                    if self.areas[i].layer_tags.meshes(key.1)
                        && self.areas[i].src_area_name == key.0
                    {
                        matching_areas.push(i);
                    }
                }

                match matching_areas.len() {
                    0 => return Err(CmnError::new(format!("No areas found with name: '{}' \
                        and tags: '{:?}'", key.0, key.1))),
                    1 => {
                        self.index.insert((key.0.clone(), key.1), matching_areas[0]);
                        return Ok(matching_areas[0]);
                    },
                    _ => Err(CmnError::new(format!("Multiple tract areas found for area: '{}' \
                        with tags: '{:?}'. Please use additional tags to specify tract area more \
                        precisely", key.0, key.1))),
                }
            }
        }
    }
}



// THALAMICTRACT: A buffer for I/O between areas. Effectively analogous to the internal capsule.
pub struct ThalamicTract {
    vec_buffer: Vec<u8>,
    // ocl_buffer: Buffer<u8>,
    tract_areas: TractAreaCache,
    ttl_len: usize,
}

impl ThalamicTract {
    fn new() -> ThalamicTract {
        let vec_buffer = Vec::with_capacity(0);
        // let ocl_buffer = Buffer::new(queue: Queue, flags: Option<MemFlags>, dims: D,
        //         data: Option<&[T]>
        let tract_areas = TractAreaCache::new();

        ThalamicTract {
            vec_buffer: vec_buffer,
            tract_areas: tract_areas,
            ttl_len: 0,
        }
    }

    fn add_area(&mut self, src_area_name: String, layer_tags: LayerTags, layer_dims: CorticalDims) {
        // println!("###### ThalamicTract::new(): Adding tract for area: {}, tags: {}, layer_dims: {:?}",
        //     src_area_name, layer_tags, layer_dims);
        let tract_dims: TractDims = layer_dims.into();
        let len = tract_dims.to_len();
        let new_area = TractArea::new(src_area_name.clone(), layer_tags,
            self.ttl_len..(self.ttl_len + len), tract_dims, TractAreaBufferKind::Vec);

        self.tract_areas.insert(src_area_name, layer_tags, new_area);
        self.ttl_len += len;
    }

    fn init(mut self) -> ThalamicTract {
        self.vec_buffer.resize(self.ttl_len, 0);
        // println!("{}THALAMICTRACT::INIT(): tract_areas: {:?}", cmn::MT, self.tract_areas);
        self
    }

    // fn frame(&mut self, src_area_name: &str, layer_tags: LayerTags)
    fn frame<'t>(&'t mut self, key: &(String, LayerTags))
            -> Result<(&EventList, TractFrame<'t>), CmnError>
    {
        let ta = try!(self.tract_areas.get(key));
        let range = ta.range().clone();
        let tract = TractFrame::new(&self.vec_buffer[range], ta.dims());
        let events = ta.events();

        Ok((events, tract))
    }

    fn frame_mut<'t>(&'t mut self, key: &(String, LayerTags))
            -> Result<(TractFrameMut<'t>, &mut EventList), CmnError>
    {
        let ta = try!(self.tract_areas.get_mut(key));
        let range = ta.range().clone();
        let tract = TractFrameMut::new(&mut self.vec_buffer[range], ta.dims());
        let events = ta.events_mut();

        Ok((tract, events))
    }

    fn terminal_source<'t>(&'t mut self, key: &(String, LayerTags))
            -> CmnResult<(SliceBufferSource<'t>)>
    {
        let ta = try!(self.tract_areas.get(key));
        let range = ta.range().clone();
        let dims = ta.dims().clone();
        // let tract = TractFrame::new(&self.vec_buffer[range], ta.dims());
        let events = ta.events();
        let terminal = SliceBufferSource::new(&self.vec_buffer[range], dims, Some(events));

        terminal
    }

    fn terminal_target<'t>(&'t mut self, key: &(String, LayerTags))
            -> CmnResult<(SliceBufferTarget<'t>)>
    {
        let ta = try!(self.tract_areas.get_mut(key));
        let range = ta.range().clone();
        // let tract = TractFrameMut::new(&mut self.vec_buffer[range], ta.dims());
        let dims = ta.dims().clone();
        let events = ta.events_mut();
        let terminal = SliceBufferTarget::new(&mut self.vec_buffer[range], dims, Some(events), false);
        // let events = ta.events_mut();

        terminal
    }

    // fn verify_range(&self, range: &Range<usize>, area_name: &'static str) -> Result<(), CmnError> {
    //     if range.end > self.vec_buffer.len() {
    //         Err(CmnError::new(format!("ThalamicTract::vec_buffer_mut(): Index range for target area: '{}' \
    //             exceeds the boundaries of the input tract (length: {})", area_name,
    //             self.vec_buffer.len())))
    //     } else {
    //         Ok(())
    //     }
    // }
}



// THALAMUS:
// - Input/Output is from a CorticalArea's point of view
//   - input: to layer / area
//   - output: from layer / area
pub struct Thalamus {
    tract: ThalamicTract,
    // [TODO]: Redesign this with something other than `String` key (use a separate vec & hashmap).
    // external_pathways: HashMap<String, (ExternalPathway, Vec<LayerTags>)>,
    external_pathways: MapStore<String, (ExternalPathway, Vec<LayerTags>)>,
    // area_maps: HashMap<&'static str, AreaMap>,
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
        // let mut area_maps = HashMap::with_capacity(area_sl.areas().len());
        let mut area_maps = MapStore::with_capacity(area_sl.areas().len());

        /*=============================================================================
        ============================ THALAMIC (INPUT) AREAS ===========================
        =============================================================================*/
        for pa in area_sl.areas().iter().filter(|pa|
                    layer_map_sl[pa.layer_map_name].kind() == &LayerMapKind::Subcortical)
        {
            let es = try!(ExternalPathway::new(pa, &layer_map_sl[pa.layer_map_name]));
            let tags = es.layer_tags();
            external_pathways.insert(es.area_name().to_owned(), (es, tags))
                .map(|es_tup| panic!("Duplicate 'ExternalPathway' keys: [\"{}\"]. \
                    Only one external (thalamic) input source per area is allowed.",
                    es_tup.0.area_name()));
        }

        /*=============================================================================
        =================================== ALL AREAS =================================
        =============================================================================*/
        for (area_id, area_s) in area_sl.areas().iter().enumerate() {
            let area_map = AreaMap::new(area_id, area_s, &layer_map_sl, &area_sl, &external_pathways);

            println!("{mt}{mt}THALAMUS::NEW(): Area: \"{}\", Output layers (tracts): ",
                area_s.name(), mt = cmn::MT);

            {
                let output_layers = area_map.layers().layers_containing_tags(map::OUTPUT);

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
                        layer.layer_map_kind(), layer.axn_kind(), mt = cmn::MT);

                    tract.add_area(area_s.name().to_owned(), layer.layer_tags(), layer_dims);
                }

                assert!(output_layers.len() > 0, "Areas must have at least one afferent or efferent area.");
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
        // for (area_name, &mut (ref mut src_area, ref layer_tags_list)) in self.external_pathways.iter_mut() {
        for &mut (ref mut src_area, ref layer_tags_list) in self.external_pathways.values_mut().iter_mut() {
            src_area.cycle_next();

            for &layer_tags in layer_tags_list.iter() {
                let (tract_frame, events) = self.tract.frame_mut(&(src_area.area_name().to_owned(), layer_tags))
                    .expect("Thalamus::cycle_external_pathways()");

                // match tract_frame {
                //     FrameBufferKind::Internal(frame) => src_area.read_into(layer_tags, frame, events),
                //     FrameBufferKind::External => (),
                // }
                src_area.write_into(layer_tags, tract_frame, events)
            }
        }
    }

    pub fn tract_frame<'t>(&'t mut self, key: &(String, LayerTags))
            -> Result<(&EventList, TractFrame<'t>), CmnError>
    {
        self.tract.frame(key)
    }

    pub fn tract_frame_mut<'t>(&'t mut self, key: &(String, LayerTags))
            -> Result<(TractFrameMut<'t>, &mut EventList), CmnError>
    {
        self.tract.frame_mut(key)
    }

    pub fn tract_terminal_target<'t>(&'t mut self, key: &(String, LayerTags))
            -> CmnResult<(SliceBufferTarget<'t>)>
    {
        self.tract.terminal_target(key)
    }

    pub fn tract_terminal_source<'t>(&'t mut self, key: &(String, LayerTags))
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
        // match self.external_pathways.indices().entry(pathway_name.clone()) {
        //     Entry::Occupied(entry) => {
        //         Ok(*entry.get())
        //     },
        //     Entry::Vacant(_) => {
        //         CmnError::err(format!("Thalamus::ext_pathway_idx(): \
        //             No external pathway found named: '{}'.", pathway_name))
        //     },
        // }
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
