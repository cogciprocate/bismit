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

#![allow(dead_code, unused_imports)]

use std::ops::Range;
use std::borrow::Borrow;
use std::collections::HashMap;
use cmn::{self, CmnError, CmnResult, TractDims, TractFrame, TractFrameMut, CorticalDims, MapStore};
use map::{AreaMap, LayerMapKind, LayerAddress};
use ocl::{Context, EventList, Buffer, RwVec, FutureReadGuard, FutureWriteGuard};
use map::{AreaSchemeList, LayerMapSchemeList, /*ExecutionGraph*/};
use ::{ExternalPathway, ExternalPathwayFrame};
use tract_terminal::{SliceBufferTarget, SliceBufferSource};


/// Specifies whether or not the frame buffer for a source exists within the
/// thalamic tract or an external source itself.
#[derive(Debug, Clone)]
enum BufferKind {
    Ocl(Buffer<u8>),
    RwVec(RwVec<u8>),
    Vec(Vec<u8>),
}


#[derive(Debug)]
struct TractArea {
    src_lyr_addr: LayerAddress,
    // range: Range<usize>,
    events: EventList,
    dims: TractDims,
    kind: BufferKind,
}

impl TractArea {
    fn new<D>(src_lyr_addr: LayerAddress, dims: D, kind: BufferKind) -> TractArea
            where D: Into<TractDims>
    {
        // println!("###### TractArea::new(): Adding area with: range: {:?}, dims: {:?}", &range, &dims);
        // assert!(range.len() == dims.to_len());
        TractArea {
            src_lyr_addr: src_lyr_addr,
            // range: range,
            events: EventList::new(),
            dims: dims.into(),
            kind: kind,
        }
    }

    fn rw_vec(&self) -> Option<&RwVec<u8>> {
        match self.kind {
            BufferKind::RwVec(ref rv) => Some(rv),
            _ => None,
        }
    }

    // fn range(&self) -> &Range<usize> { &self.range }
    fn dims(&self) -> &TractDims { &self.dims }
    fn events(&self) -> &EventList { &self.events }
    fn events_mut(&mut self) -> &mut EventList { &mut self.events }
    fn kind(&self) -> &BufferKind { &self.kind }

}

// A buffer for I/O between areas. Effectively analogous to the internal capsule.
pub struct ThalamicTract {
    tract_areas: MapStore<LayerAddress, TractArea>,
    vec_buffer: Vec<u8>,
    ttl_len: usize,
}

impl ThalamicTract {
    fn new() -> ThalamicTract {
        let vec_buffer = Vec::new();

        ThalamicTract {
            tract_areas: MapStore::with_capacity(32),
            vec_buffer: vec_buffer,
            ttl_len: 0,
        }
    }

    fn add_area(&mut self, src_lyr_addr: LayerAddress, layer_dims: CorticalDims) {
        // println!("###### ThalamicTract::new(): Adding tract for area: {}, tags: {}, layer_dims: {:?}",
        //     src_area_name, layer_tags, layer_dims);
        self.ttl_len += layer_dims.to_len();
        let new_area = TractArea::new(src_lyr_addr.clone(), layer_dims,
            BufferKind::RwVec(RwVec::from(vec![0; layer_dims.to_len()])));
        self.tract_areas.insert(src_lyr_addr, new_area);

    }

    fn init(mut self) -> ThalamicTract {
        self.vec_buffer.resize(self.ttl_len, 0);
        // println!("{}THALAMICTRACT::INIT(): tract_areas: {:?}", cmn::MT, self.tract_areas);
        self
    }

    pub fn index_of<A>(&self, layer_addr: A) -> Option<usize> where A: Borrow<LayerAddress> + ::std::fmt::Debug {
        self.tract_areas.index_of(layer_addr.borrow())
    }

    pub fn read<'t>(&'t self, idx: usize) -> CmnResult<FutureReadGuard<u8>> {
        let ta = self.tract_areas.by_index(idx).ok_or(CmnError::from("invalid tract idx"))?;
        // println!("Tract area: Obtaining reader for tract area: source: {:?}, dims: {:?}",
        //     ta.src_lyr_addr, ta.dims);
        ta.rw_vec().ok_or(CmnError::from("ThalamicTract::read")).map(|rv| rv.clone().read())
    }

    pub fn write<'t>(&'t self, idx: usize) -> CmnResult<FutureWriteGuard<u8>> {
        let ta = self.tract_areas.by_index(idx).ok_or(CmnError::from("invalid tract idx"))?;
        // println!("Tract area: Obtaining writer for tract area: source: {:?}, dims: {:?}",
        //     ta.src_lyr_addr, ta.dims);
        ta.rw_vec().ok_or(CmnError::from("ThalamicTract::write")).map(|rv| rv.clone().write())
    }

    pub fn buffer<'t>(&'t self, idx: usize) -> CmnResult<&RwVec<u8>> {
        let ta = self.tract_areas.by_index(idx).ok_or(CmnError::from("invalid tract idx"))?;
        ta.rw_vec().ok_or(CmnError::from("no RwVec found"))
    }

    // // pub fn terminal_source<'t>(&'t mut self, key: &LayerAddress)
    // pub fn terminal_source<'t>(&'t mut self, idx: usize)
    //         -> CmnResult<(SliceBufferSource<'t>)>
    // {
    //     // let ta = self.tract_areas.by_key(key).unwrap();
    //     let ta = self.tract_areas.by_index(idx).unwrap();
    //     let range = ta.range().clone();
    //     let dims = ta.dims().clone();
    //     let events = ta.events();
    //     SliceBufferSource::new(&self.vec_buffer[range], dims, Some(events))
    // }

    // // pub fn terminal_target<'t>(&'t mut self, key: &LayerAddress)
    // pub fn terminal_target<'t>(&'t mut self, idx: usize)
    //         -> CmnResult<(SliceBufferTarget<'t>)>
    // {
    //     // let ta = self.tract_areas.by_key_mut(key).unwrap();
    //     let ta = self.tract_areas.by_index_mut(idx).unwrap();
    //     let range = ta.range().clone();
    //     let dims = ta.dims().clone();
    //     let events = ta.events_mut();
    //     SliceBufferTarget::new(&mut self.vec_buffer[range], dims, Some(events), false)
    // }
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

            let mut output_layer_count = 0;
            for layer in area_map.layer_map().iter().filter(|li| li.axn_domain().is_output()) {
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
                output_layer_count += 1;

            }
            assert!(output_layer_count > 0, "Areas must have at least one output layer.");

            area_maps.insert(area_s.name().to_owned(), area_map);
            assert!(area_maps[area_id].area_id() == area_id);
        }

        let thal = Thalamus {
            tract: tract.init(),
            external_pathways: external_pathways,
            area_maps: area_maps,
        };

        Ok(thal)
    }

    // Multiple source output areas disabled.
    //
    // NOTE: Do not disable `RwVec` locking. A write lock must be queued each
    // cycle to prevent read locks piling up.
    pub fn cycle_external_pathways(&mut self) {
        for &mut (ref mut src_ext_path, ref layer_addr_list) in self.external_pathways.values_mut().iter_mut() {
            if src_ext_path.is_disabled() { continue; }
            src_ext_path.cycle_next();
            for &layer_addr in layer_addr_list.iter() {
                // TODO: ExternalPathway needs to store tract index.
                ////// DEBUG TEMP:
                    let tract_area_idx = self.tract.index_of(&layer_addr).unwrap();
                    let future_write = self.tract.write(tract_area_idx)
                        .expect("Thalamus::cycle_external_pathways()");
                ///////
                    src_ext_path.write_into(layer_addr, future_write)
                ///////
            }
        }
    }

    pub fn ext_pathway_idx<S: AsRef<str>>(&self, pathway_name: S) -> CmnResult<usize> {
        match self.external_pathways.indices().get(pathway_name.as_ref()) {
            Some(&idx) => Ok(idx),
            None => CmnError::err(format!("Thalamus::ext_pathway_idx(): \
                No external pathway found named: '{}'.", pathway_name.as_ref())),
        }
    }

    pub fn ext_pathway(&mut self, pathway_idx: usize) -> CmnResult<&mut ExternalPathway> {
        let pathway = try!(self.external_pathways.by_index_mut(pathway_idx).ok_or(
            CmnError::new(format!("Thalamus::ext_pathway_frame(): Invalid pathway index: '{}'.",
            pathway_idx))));
        Ok(&mut pathway.0)
    }

    // pub fn ext_pathway_frame(&mut self, pathway_idx: usize) -> CmnResult<ExternalPathwayFrame> {
    //     let pathway = try!(self.ext_pathway(pathway_idx));
    //     pathway.ext_frame_mut()
    // }

    // // [NOTE]: Incoming array values beyond the length of destination slice will
    // // be silently ignored.
    // fn intake_sensory_frame(&mut self, frame: SensoryFrame) -> CmnResult<()> {
    //     // // DEBUG:
    //     // println!("Intaking sensory frames...");

    //     match frame {
    //         SensoryFrame::F32Array16(arr) => {
    //             // println!("Intaking sensory frame [pathway id: {}]: {:?} ...",
    //             //     pathway_idx, arr);

    //             // let pathway = match try!(self.cortex.thal_mut().ext_pathway_frame(pathway_idx)) {
    //             let pathway = match self.cortex.thal_mut().ext_pathway(pathway_idx)? {
    //                 ExternalPathwayFrame::F32Slice(s) => s,
    //                 f @ _ => panic!(format!("Flywheel::intake_sensory_frames(): Unsupported \
    //                     ExternalPathwayFrame variant: {:?}", f)),
    //             };

    //             for (i, dst) in pathway.iter_mut().enumerate() {
    //                 *dst = arr[i];
    //             }
    //         },
    //         SensoryFrame::PathwayConfig(pc) => match pc {
    //             PathwayConfig::EncoderRanges(ranges) => {
    //                 // match try!(self.cortex.thal_mut().ext_pathway(pathway_idx)).encoder() {
    //                 //     &mut ExternalPathwayEncoder::VectorEncoder(ref mut v) => {
    //                 //         try!(v.set_ranges(&ranges.lock().unwrap()[..]));
    //                 //     }
    //                 //     _ => unimplemented!(),
    //                 // }

    //                 self.cortex.thal_mut().ext_pathway(pathway_idx)?
    //                     .set_encoder_ranges(ranges);
    //             }
    //         },
    //         SensoryFrame::Tract(_) => unimplemented!(),
    //     }

    //     Ok(())
    // }

    pub fn tract(&self) -> &ThalamicTract { &self.tract }
    pub fn tract_mut(&mut self) -> &mut ThalamicTract { &mut self.tract }
    pub fn area_maps(&self) -> &MapStore<String, AreaMap> { &self.area_maps }
}


#[cfg(test)]
pub mod tests {

}
