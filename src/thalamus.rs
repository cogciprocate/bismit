//! [FIXME]: Every single hash lookup requires a heap allocation (`String`).
//! This is obviously very wasteful and is temporary until one of the
//! following can be implemented:
//! - Convert area names to indexes (preferred). Use a table stored
//!     somewhere to look up names for display.
//!     - Use a hashmap to resolve area ids to area names in the event of an
//!       error. Store this hashmap both on `TractAreaCache`. Each
//!       `CorticalArea` will, of course, also have a copy of its own area id.
//!     - Possibly have `AreaSchemeList` initially create the id list.
//! - Precompute hash.
//! - Store strings in a separate vector (stored in cortex) and put a
//!   reference in the key.
//!     - Will need some sort of global lookup system. Bleh.
//! - Think of something else (but top opt looks good).
//!
//!

use std::ops::Range;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use cmn::{self, CmnError, CmnResult, TractDims, TractFrame, TractFrameMut, CorticalDims};
use map::{self, AreaMap, LayerTags, LayerMapKind};
use ocl::EventList;
use area::CorticalAreas;
use map::{AreaSchemeList, LayerMapSchemeList};
use external_source::ExternalSource;


// /// Specifies whether or not the frame buffer for a source exists within the
// /// thalamic tract or an external source itself.
// ///
// /// `External` is used when the source implements a special buffering strategy
// /// such as double-buffering.
// enum FrameBufferKind<'t> {
//     Internal(&'t mut [u8]),
//     External,
// }


// THALAMICTRACT: A buffer for I/O between areas. Effectively analogous to the internal capsule.
pub struct ThalamicTract {
    ganglion: Vec<u8>,
    tract_areas: TractAreaCache,
    ttl_len: usize,
}

impl ThalamicTract {
    fn new() -> ThalamicTract {
        let ganglion = Vec::with_capacity(0);
        let tract_areas = TractAreaCache::new();

        ThalamicTract {
            ganglion: ganglion,
            tract_areas: tract_areas,
            ttl_len: 0,
        }
    }

    fn add_area(&mut self, src_area_name: String, layer_tags: LayerTags, layer_dims: &CorticalDims) {
        let tract_dims: TractDims = layer_dims.into();
        let len = tract_dims.to_len();
        self.tract_areas.insert(src_area_name.clone(), layer_tags,
            TractArea::new(src_area_name, layer_tags, self.ttl_len..(self.ttl_len + len),
            tract_dims));
        self.ttl_len += len;
    }

    fn init(mut self) -> ThalamicTract {
        self.ganglion.resize(self.ttl_len, 0);
        // println!("{}THALAMICTRACT::INIT(): tract_areas: {:?}", cmn::MT, self.tract_areas);
        self
    }

    // fn frame(&mut self, src_area_name: &str, layer_tags: LayerTags)
    fn frame<'t>(&'t mut self, key: &(String, LayerTags))
                -> Result<(&EventList, TractFrame<'t>), CmnError> {
        let ta = try!(self.tract_areas.get(key));
        let range = ta.range().clone();
        let tract = TractFrame::new(&self.ganglion[range], ta.dims());
        let events = ta.events();

        Ok((events, tract))
    }

    fn frame_mut<'t>(&'t mut self, key: &(String, LayerTags))
                -> Result<(TractFrameMut<'t>, &mut EventList), CmnError> {
        let ta = try!(self.tract_areas.get_mut(key));
        let range = ta.range().clone();
        let tract = TractFrameMut::new(&mut self.ganglion[range], ta.dims());
        let events = ta.events_mut();

        Ok((tract, events))
    }

    // fn verify_range(&self, range: &Range<usize>, area_name: &'static str) -> Result<(), CmnError> {
    //     if range.end > self.ganglion.len() {
    //         Err(CmnError::new(format!("ThalamicTract::ganglion_mut(): Index range for target area: '{}' \
    //             exceeds the boundaries of the input tract (length: {})", area_name,
    //             self.ganglion.len())))
    //     } else {
    //         Ok(())
    //     }
    // }
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
            .map(|_| panic!("Duplicate 'TractAreaCache' keys: (area: \"{}\", tags: '{:?}')",
                src_area_name, layer_tags));
    }

    fn get(&mut self, key: &(String, LayerTags)) -> Result<&TractArea, CmnError> {
        match self.area_search(key) {
            Ok(idx) => self.areas.get(idx).ok_or(CmnError::new(format!("Index '{}' not found for '{}' \
                with tags '{:?}'", idx, key.0, key.1))),

            Err(err) => Err(err),
        }
    }

    // fn get_mut(&mut self, src_area_name: &str, layer_tags: LayerTags
    fn get_mut(&mut self, key: &(String, LayerTags)) -> Result<&mut TractArea, CmnError> {
        match self.area_search(key) {
            Ok(idx) => self.areas.get_mut(idx).ok_or(CmnError::new(format!("Index '{}' not \
                found for '{}' with tags '{:?}'", idx, key.0, key.1))),

            Err(err) => {
                Err(err)
            },
        }
    }

    // fn area_search(&mut self, src_area_name: &str, layer_tags: LayerTags)
    fn area_search(&mut self, key: &(String, LayerTags)) -> Result<usize, CmnError> {
        // println!("TractAreaCache::area_search(): Searching for area: {}, tags: {:?}. ALL: {:?}",
        //     src_area_name, layer_tags, self.areas);
        let area_idx = self.index.get(key).map(|&idx| idx);

        // println!("   area_idx: {:?}", area_idx);

        let mut matching_areas: Vec<usize> = Vec::with_capacity(4);

        match area_idx {
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


// /// Specifies whether or not the frame buffer for a source exists within the
// /// thalamic tract or an external source itself.
// #[derive(Debug)]
// enum TractAreaKind {
//     Internal,
//     External,
// }


#[derive(Debug)]
struct TractArea {
    src_area_name: String,
    layer_tags: LayerTags,
    range: Range<usize>,
    events: EventList,
    dims: TractDims,
}

impl TractArea {
    fn new(src_area_name: String, layer_tags: LayerTags, range: Range<usize>,
                dims: TractDims) -> TractArea {
        println!("###### TractArea::new(): Adding area with: range: {:?}, dims: {:?}", &range, &dims);
        assert!(range.len() == dims.to_len());
        TractArea {
            src_area_name: src_area_name,
            layer_tags: layer_tags,
            range: range,
            events: EventList::new(),
            dims: dims,
        }
    }

    fn range(&self) -> &Range<usize> {
        &self.range
    }

    // #[allow(dead_code)]
    // fn len(&self) -> usize {
    //     self.range.len()
    // }

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


//    THALAMUS:
//    - Input/Output is from a CorticalArea's point of view
//         - input: to layer / area
//         - output: from layer / area
pub struct Thalamus {
    tract: ThalamicTract,
    // [TODO]: Redesign this with something other than `String` key (use a separate vec & hashmap).
    external_sources: HashMap<String, (ExternalSource, Vec<LayerTags>)>,
    area_maps: HashMap<&'static str, AreaMap>,
}

impl Thalamus {
    pub fn new(plmaps: LayerMapSchemeList, mut pamaps: AreaSchemeList) -> Thalamus {
        pamaps.freeze();
        let pamaps = pamaps;
        // let area_count = pamaps.maps().len();

        let mut tract = ThalamicTract::new();
        let mut external_sources = HashMap::with_capacity(pamaps.maps().len());
        let mut area_maps = HashMap::with_capacity(pamaps.maps().len());

        /*=============================================================================
        ============================ THALAMIC (INPUT) AREAS ===========================
        =============================================================================*/
        for (&_, pa) in pamaps.maps().iter().filter(|&(_, pa)|
                    &plmaps[pa.layer_map_name].kind == &LayerMapKind::Thalamic)
        {
            let es = ExternalSource::new(pa, &plmaps[pa.layer_map_name]);
            let tags = es.layer_tags();
            external_sources.insert(es.area_name().to_owned(), (es, tags))
                .map(|es_tup| panic!("Duplicate 'ExternalSource' keys: [\"{}\"]. \
                    Only one external (thalamic) input source per area is allowed.",
                    es_tup.0.area_name()));
        }

        /*=============================================================================
        =================================== ALL AREAS =================================
        =============================================================================*/
        for (&area_name, pamap) in pamaps.maps().iter() {
            let area_map = AreaMap::new(pamap, &plmaps, &pamaps, &external_sources);

            println!("{mt}{mt}THALAMUS::NEW(): Area: \"{}\", Output layers (tracts): ", area_name, mt = cmn::MT);

            {
                let output_layers = area_map.layers().layers_containing_tags(map::OUTPUT);

                for layer in output_layers.iter() {
                    // If the layer is thalamic is will have an irregular size
                    // which will need to be reflected on its tract size.
                    let layer_dims = match layer.irregular_layer_dims() {
                        Some(dims) => dims,
                        None => pamap.dims(),
                    };

                    println!("{mt}{mt}{mt}'{}': tags: {}, slc_range: {:?}, map_kind: {:?}, \
                        axn_kind: {:?}", layer.name(), layer.tags(), layer.slc_range(),
                        layer.layer_map_kind(), layer.axn_kind(), mt = cmn::MT);

                    tract.add_area(area_name.to_owned(), layer.tags(), layer_dims);
                }

                assert!(output_layers.len() > 0, "Areas must have at least one afferent or efferent area.");
            }

            area_maps.insert(area_name, area_map);

        }

        Thalamus {
            tract: tract.init(),
            external_sources: external_sources,
            area_maps: area_maps,
        }
    }

    // Multiple source output areas disabled.
    pub fn cycle_external_tracts(&mut self, _: &mut CorticalAreas) {
        for (area_name, &mut (ref mut src_area, ref layer_tags_list)) in self.external_sources.iter_mut() {
            for &layer_tags in layer_tags_list.iter() {
                let (tract_frame, events) = self.tract.frame_mut(&(area_name.to_owned(), layer_tags))
                    .expect("Thalamus::cycle_external_tracts()");

                // match tract_frame {
                //     FrameBufferKind::Internal(frame) => src_area.read_into(layer_tags, frame, events),
                //     FrameBufferKind::External => (),
                // }
                src_area.write_into(layer_tags, tract_frame, events)
            }
            src_area.cycle_next();
        }
    }

    pub fn tract<'t>(&'t mut self, key: &(String, LayerTags))
            -> Result<(&EventList, TractFrame<'t>), CmnError>
    {
        self.tract.frame(key)
    }

    pub fn tract_mut<'t>(&'t mut self, key: &(String, LayerTags))
            -> Result<(TractFrameMut<'t>, &mut EventList), CmnError>
    {
        self.tract.frame_mut(key)
    }

     pub fn area_maps(&self) -> &HashMap<&'static str, AreaMap> {
         &self.area_maps
    }

     pub fn area_map(&self, area_name: &'static str) -> &AreaMap {
         &self.area_maps[area_name]
    }

    pub fn external_tract_mut(&mut self, tract_name: String) -> CmnResult<TractFrameMut> {
        match self.external_sources.entry(tract_name.clone()) {
            Entry::Occupied(entry) => {
                entry.into_mut().0.buf_mut()
            },
            Entry::Vacant(_) => {
                Err(CmnError::new(format!("Thalamus::external_tract_mut(): \
                    No external tract found named: '{}'.", tract_name)))
            },
        }
    }
}


#[cfg(test)]
pub mod tests {

}
