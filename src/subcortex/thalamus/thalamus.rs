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

use std::borrow::Borrow;
use futures::{Future, Async};
use cmn::{self, CmnError, CmnResult, TractDims, CorticalDims, MapStore};
use map::{AreaMap, LayerAddress, AreaSchemeList, LayerMapSchemeList};
use ocl::{Context, EventList, Buffer, RwVec, FutureReadGuard, FutureWriteGuard};
use ::{InputGenerator, WorkPool};
use subcortex::{self, Subcortex, TractSender, TractReceiver};


/// Specifies whether or not the frame buffer for a source exists within the
/// thalamic tract or an external source itself.
#[derive(Debug)]
pub enum TractBuffer {
    Ocl(Buffer<u8>),
    RwVec(RwVec<u8>),
    Vec(Vec<u8>),
}


/// An area of the thalamic tract.
#[derive(Debug)]
struct TractArea {
    src_lyr_addr: LayerAddress,
    events: EventList,
    dims: TractDims,
    buffer: TractBuffer,
}

#[allow(dead_code)]
impl TractArea {
    fn new<D>(src_lyr_addr: LayerAddress, dims: D, buffer: TractBuffer) -> TractArea
            where D: Into<TractDims> {
        TractArea {
            src_lyr_addr: src_lyr_addr,
            events: EventList::new(),
            dims: dims.into(),
            buffer: buffer,
        }
    }

    fn rw_vec(&self) -> Option<&RwVec<u8>> {
        match self.buffer {
            TractBuffer::RwVec(ref rv) => Some(rv),
            _ => None,
        }
    }

    fn dims(&self) -> &TractDims { &self.dims }
    fn events(&self) -> &EventList { &self.events }
    fn events_mut(&mut self) -> &mut EventList { &mut self.events }
    fn buffer(&self) -> &TractBuffer { &self.buffer }
}


// A buffer for I/O between areas. Effectively analogous to the internal capsule.
#[derive(Debug)]
pub struct ThalamicTract {
    tract_areas: MapStore<LayerAddress, TractArea>,
    ttl_len: usize,
}

impl ThalamicTract {
    fn new() -> ThalamicTract {

        ThalamicTract {
            tract_areas: MapStore::with_capacity(32),
            ttl_len: 0,
        }
    }

    fn add_area(&mut self, src_lyr_addr: LayerAddress, layer_dims: CorticalDims) {
        self.ttl_len += layer_dims.to_len();
        let new_area = TractArea::new(src_lyr_addr.clone(), layer_dims,
            TractBuffer::RwVec(RwVec::from(vec![0; layer_dims.to_len()])));
        self.tract_areas.insert(src_lyr_addr, new_area);

    }

    pub fn index_of<A>(&self, layer_addr: A) -> Option<usize> where A: Borrow<LayerAddress> {
        self.tract_areas.index_of(layer_addr.borrow())
    }

    pub fn read<'t>(&'t self, idx: usize) -> CmnResult<FutureReadGuard<Vec<u8>>> {
        let ta = self.tract_areas.by_index(idx).ok_or(CmnError::from("invalid tract idx"))?;
        ta.rw_vec().ok_or(CmnError::from("ThalamicTract::read")).map(|rv| rv.clone().read())
    }

    pub fn write<'t>(&'t self, idx: usize) -> CmnResult<FutureWriteGuard<Vec<u8>>> {
        let ta = self.tract_areas.by_index(idx).ok_or(CmnError::from("invalid tract idx"))?;
        ta.rw_vec().ok_or(CmnError::from("ThalamicTract::write")).map(|rv| rv.clone().write())
    }

    pub fn buffer_rwvec<'t>(&'t self, idx: usize) -> CmnResult<&RwVec<u8>> {
        let ta = self.tract_areas.by_index(idx).ok_or(CmnError::from("invalid tract idx"))?;
        ta.rw_vec().ok_or(CmnError::from("no RwVec found"))
    }

    pub fn buffer<'t>(&'t self, idx: usize) -> CmnResult<&TractBuffer> {
        self.tract_areas.by_index(idx).ok_or(CmnError::from("invalid tract idx"))
            .map(|ta| &ta.buffer)
    }
}


#[derive(Debug)]
#[allow(dead_code)]
pub enum Pathway {
    Input { tract_area_id: usize, rx: TractReceiver, wait_for_frame: bool },
    Output { tract_area_id: usize, tx: TractSender },
}



// THALAMUS:
// - Input/Output is from a CorticalArea's point of view
//   - input: to layer / area
//   - output: from layer / area
pub struct Thalamus {
    tract: ThalamicTract,
    pathways: MapStore<LayerAddress, Pathway>,
    area_maps: MapStore<String, AreaMap>,
}

impl Thalamus {
    pub fn new(layer_map_schemes: LayerMapSchemeList, mut area_schemes: AreaSchemeList,
            subcortex: &Subcortex, _ocl_context: &Context) -> CmnResult<Thalamus> {
        area_schemes.freeze();
        let area_schemes = area_schemes;
        let mut tract = ThalamicTract::new();
        let mut area_maps = MapStore::with_capacity(area_schemes.areas().len());

        /*=============================================================================
        =================================== ALL AREAS =================================
        =============================================================================*/
        for (area_id, area_s) in area_schemes.areas().iter().enumerate() {
            assert!(area_s.area_id() == area_id);
            let area_map = AreaMap::new(area_id, area_s, &layer_map_schemes, &area_schemes,
                subcortex)?;

            println!("{mt}{mt}THALAMUS::NEW(): Area: \"{}\", Output layers (tracts): ",
                area_s.name(), mt = cmn::MT);

            // Output layer tracts:
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
            assert!(output_layer_count > 0, "Thalamus::new: Area \"{}\" has no output layers. \
                Areas must have at least one output (AxonDomain::Output) layer.", area_s.name());

            // // Input layer tracts:
            // for layer in area_map.layer_map().iter().filter(|li| li.axn_domain().is_input()) {

            // }

            area_maps.insert(area_s.name().to_owned(), area_map);
            assert!(area_maps[area_id].area_id() == area_id);
        }

        let thal = Thalamus {
            tract,
            pathways: MapStore::with_capacity(16),
            area_maps: area_maps,
        };

        Ok(thal)
    }

    /// Cycles thalamic tract pathways.
    pub fn cycle_pathways(&mut self, _work_pool: &mut WorkPool) {
        // Cycle all input pathways first.
        for pathway in self.pathways.values_mut().iter_mut() {
            if let Pathway::Input { ref mut rx, wait_for_frame, .. } = *pathway {
                //////// KEEPME: There may be some reason to asynchronously queue this:
                // let future_read_guard = rx.recv(wait_for_frame)
                //     .map(|buf_opt| buf_opt.map(|buf| buf.read_u8()))
                //     .flatten()
                //     .map(|_guard_opt| ())
                //     .map_err(|err| panic!("{}", err));

                // work_pool.complete(future_read_guard)
                //     .expect("Thalamus::cycle_pathways")
                //////// KEEPME

                //////// KEEPME: This will send the `FutureReadGuard` to the pool.
                //////// We may want to just wait after all for some reason.
                // if let Some(read_buffer) = rx.recv(wait_for_frame).wait().unwrap() {
                //     let future_read_guard = read_buffer.read_u8()
                //         .map(|_read_guard| ())
                //         .map_err(|err| panic!("{}", err));

                //     work_pool.complete(future_read_guard)
                //         .expect("Thalamus::cycle_pathways")
                // }
                //////// KEEPME

                match rx.recv(wait_for_frame).poll() {
                    Ok(Async::Ready(None)) => (),
                    Ok(Async::Ready(_)) => panic!("Thalamus::cycle_pathways: \
                        Nothing to receive. `Pathway::Input` should contain a send \
                        only tract channel "),
                    Ok(Async::NotReady) => panic!("Thalamus::cycle_pathways: \
                        Cycling pathways for input pathways (`Pathway::Input`) \
                        should never have to wait."),
                    Err(err) => panic!("{:?}", err),
                }
            }
        }

        // Then cycle output pathways.
        for pathway in self.pathways.values_mut().iter_mut() {
            if let Pathway::Output { ref mut tx, .. } = *pathway {
                tx.send().wait().expect("error cycling thalamic output pathway");
            }
        }
    }

    /// Creates a thalamic tract input pathway.
    pub fn input_pathway(&mut self, src_lyr_addr: LayerAddress, wait_for_frame: bool)
            -> TractSender {
        let tract_area_id = self.tract.index_of(&src_lyr_addr)
            .expect(&format!("Thalamus::input_pathway: \
                No thalamic tract area with layer address: '{:?}'.", &src_lyr_addr));

        let buffer = match self.tract.buffer(tract_area_id) {
            Ok(&TractBuffer::RwVec(ref rw_vec)) => rw_vec.clone(),
            Ok(tb @ _) => panic!("Thalamus::input_pathway: \
                Unsupported tract buffer type: '{:?}'.", tb),
            Err(err) => panic!("Thalamus::input_pathway: \
                (tract area id: {}): {}", tract_area_id, err),
        };

        let (tx, rx) = subcortex::tract_channel_single_u8_send_only(buffer, None, true);

        // Send a dummy/init frame, dropping the guard immediately:
        tx.send().wait().unwrap().unwrap().write_u8().wait().unwrap();

        let pathway = Pathway::Input { tract_area_id, rx, wait_for_frame };
        self.pathways.insert(src_lyr_addr, pathway);

        tx
    }

    /// Creates a thalamic tract output pathway.
    pub fn output_pathway(&mut self, src_lyr_addr: LayerAddress) -> TractReceiver {
        let tract_area_id = self.tract.index_of(&src_lyr_addr)
            .expect(&format!("Thalamus::output_pathway: \
                No thalamic tract area with layer address: '{:?}'.", &src_lyr_addr));

        let buffer = match self.tract.buffer(tract_area_id) {
            Ok(&TractBuffer::RwVec(ref rw_vec)) => rw_vec.clone(),
            Ok(tb @ _) => panic!("Thalamus::input_pathway: \
                Unsupported tract buffer type: '{:?}'.", tb),
            Err(err) => panic!("Thalamus::input_pathway: \
                (tract area id: {}): {}", tract_area_id, err),
        };

        let (tx, rx) = subcortex::tract_channel_single_u8_recv_only(buffer, None, true);

        let pathway = Pathway::Output { tract_area_id, tx };
        self.pathways.insert(src_lyr_addr, pathway);

        rx
    }


    // [REMOVE ME]
    //
    // Multiple source output areas disabled.
    //
    // NOTE: Do not disable `RwVec` locking. A write lock must be queued each
    // cycle to prevent read locks piling up. [NOTE: 2017-Oct-14: This may
    // have been corrected by ocl patch -- Verify]
    #[deprecated]
    pub fn cycle_input_generators(&mut self) {
        // for &mut (ref mut input_gen, ref layer_addr_list) in self.input_generators.values_mut().iter_mut() {
        //     if input_gen.is_disabled() { continue; }
        //     input_gen.cycle_next();
        //     for &layer_addr in layer_addr_list.iter() {
        //         // TODO: InputGenerator needs to store tract index.
        //         let tract_area_idx = self.tract.index_of(&layer_addr).unwrap();
        //         let future_write = self.tract.write(tract_area_idx)
        //             .expect("Thalamus::cycle_input_generators()");
        //         input_gen.write_into(layer_addr, future_write)
        //     }
        // }
        unimplemented!();
    }

    #[deprecated]
    pub fn input_generator_idx<S: AsRef<str>>(&self, _pathway_name: S) -> CmnResult<usize> {
        // match self.input_generators.indices().get(pathway_name.as_ref()) {
        //     Some(&idx) => Ok(idx),
        //     None => CmnError::err(format!("Thalamus::input_generator_idx(): \
        //         No external pathway found named: '{}'.", pathway_name.as_ref())),
        // }
        unimplemented!();
    }

    #[deprecated]
    pub fn input_generator(&mut self, _pathway_idx: usize) -> CmnResult<&mut InputGenerator> {
        // let pathway = try!(self.input_generators.by_index_mut(pathway_idx).ok_or(
        //     CmnError::new(format!("Thalamus::input_generator_frame(): Invalid pathway index: '{}'.",
        //     pathway_idx))));
        // Ok(&mut pathway.0)
        unimplemented!();
    }

    pub fn tract(&self) -> &ThalamicTract { &self.tract }
    pub fn tract_mut(&mut self) -> &mut ThalamicTract { &mut self.tract }
    pub fn area_maps(&self) -> &MapStore<String, AreaMap> { &self.area_maps }
}


#[cfg(test)]
pub mod tests {

}
