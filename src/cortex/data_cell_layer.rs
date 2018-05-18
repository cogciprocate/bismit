//! Data Cells (Principal Neurons)
//!

use std::fmt::Debug;
use ocl::Buffer;
use cortex::{Dendrites, ControlCellLayers, Tufts};
use cmn::{CmnResult, CorticalDims};
use map::{CellScheme, ExecutionGraph, LayerAddress};

#[cfg(any(test, feature = "eval"))]
pub use self::tests::{DataCellLayerTest, CelCoords, DataCellLayerMap};

pub trait DataCellLayer: 'static + Debug + Send {
    fn layer_name<'s>(&'s self) -> &'s str;
    fn layer_addr(&self) -> LayerAddress;
    fn cycle(&mut self, &mut ControlCellLayers, &mut ExecutionGraph) -> CmnResult<()>;
    fn learn(&mut self, &mut ExecutionGraph) -> CmnResult <()> ;
    fn regrow(&mut self);
    fn soma(&self) -> &Buffer<u8>;
    fn soma_mut(&mut self) -> &mut Buffer<u8>;
    fn energies(&self) -> &Buffer<u8>;
    fn activities(&self) -> &Buffer<u8>;
    fn flag_sets(&self) -> &Buffer<u8>;
    fn dims(&self) -> &CorticalDims;
    fn axon_range(&self) -> (usize, usize);
    fn axon_slc_ids(&self) -> &[u8];
    fn base_axon_slc(&self) -> u8;
    fn tft_count(&self) -> usize;
    fn cell_scheme(&self) -> &CellScheme;
    fn tufts(&self) -> &Tufts;
    fn dens(&self) -> &Dendrites;
    fn dens_mut(&mut self) -> &mut Dendrites;
}



#[cfg(any(test, feature = "eval"))]
pub mod tests {
    #![allow(dead_code, unused_imports, unused_variables)]

    use std::sync::Arc;
    use std::ops::{Range, Deref};
    use std::fmt::{Display, Formatter, Result};
    // use rand::{XorShiftRng};
    use map::{AreaMap, AreaMapTest, LayerAddress, axon_idx, DendriteClass, DendriteKind};
    use cmn::{self, CorticalDims, XorShiftRng, SliceDims};
    use cortex::{den_idx, syn_idx, TuftDims};
    use super::DataCellLayer;
    use {Thalamus, SlcId};


    pub trait DataCellLayerTest: DataCellLayer {
        fn cycle_solo(&self);
        fn learn_solo(&mut self);
        // fn cycle_soma_only_solo(&self);
        // fn print_cel(&mut self, cel_idx: usize);
        // fn print_range(&mut self, range: Range<usize>, print_syns: bool);
        fn print_range(&self, idx_range: Option<Range<usize>>);
        // fn print_all(&mut self, print_syns: bool);
        fn print_all(&self, /*print_children: bool*/);
        fn rng(&mut self) -> &mut XorShiftRng;
        fn rand_cel_coords(&mut self) -> CelCoords;
        fn last_cel_coords(&self) -> CelCoords;
        fn cel_idx(&self, slc_id: u8, v_id: u32, u_id: u32)-> u32;
        fn celtft_idx(&self, tft_id: usize, cel_coords: &CelCoords) -> u32;
        fn set_all_to_zero(&mut self);
        // fn confab(&mut self);
    }


    #[derive(Debug, Clone)]
    pub struct CelCoords {
        pub idx: u32,
        pub slc_id_lyr: u8,
        pub axon_slc_id: u8,
        pub v_id: u32,
        pub u_id: u32,
        pub lyr_dims: CorticalDims,
        axon_idx: Option<u32>,
    }

    impl CelCoords {
        pub fn new(axon_slc_id: u8, slc_id_lyr: u8, v_id: u32, u_id: u32,
                lyr_dims: CorticalDims) -> CelCoords {
            let idx = cmn::cel_idx_3d(lyr_dims.depth(), slc_id_lyr, lyr_dims.v_size(),
                v_id, lyr_dims.u_size(), u_id);

            CelCoords {
                idx: idx,
                slc_id_lyr: slc_id_lyr,
                axon_slc_id: axon_slc_id,
                v_id: v_id,
                u_id: u_id,
                lyr_dims: lyr_dims,
                axon_idx: None,
            }
        }

        pub fn set_axon_idx(&mut self, area_map: &AreaMap) {
            self.axon_idx = Some(area_map.axon_idx(self.axon_slc_id, self.v_id, 0, self.u_id, 0).unwrap())
        }

        pub fn idx(&self) -> u32 {
            self.idx
        }

        pub fn axon_idx(&self) -> u32 {
            self.axon_idx.expect("CelCoords::axon_idx: Axon index not set. \
                Use `::set_cel_axon_idx` first.")
        }

        // #[allow(dead_code)]
        pub fn col_id(&self) -> u32 {
            // Fake a slice id of 0 with a slice depth of 1 and ignore our actual depth and id:
            cmn::cel_idx_3d(1, 0, self.lyr_dims.v_size(), self.v_id,
                self.lyr_dims.u_size(), self.u_id)
        }

        #[deprecated(note = "Use `::set_cel_axon_idx` and `::axon_idx` instead.")]
        pub fn cel_axon_idx(&self, area_map: &AreaMap) -> u32 {
            area_map.axon_idx(self.axon_slc_id, self.v_id, 0, self.u_id, 0).unwrap()
        }
    }

    impl Display for CelCoords {
        fn fmt(&self, fmtr: &mut Formatter) -> Result {
            write!(fmtr, "CelCoords {{ idx: {}, slc_id_lyr: {}, axon_slc_id: {}, v_id: {}, u_id: {} }}",
                self.idx, self.slc_id_lyr, self.axon_slc_id, self.v_id, self.u_id)
        }
    }


    /// A data cell layer map error.
    #[derive(Debug, Fail)]
    pub enum DataCellLayerMapError {
        // #[fail(display = "{}", _0)]
        // OclCore(OclCoreError),
        #[fail(display = "Multiple matching tufts found.")]
        MultipleMatchingTufts,
    }



    /// A synapse map.
    #[derive(Debug)]
    pub struct Synapse<'d> {
        den: &'d Dendrite<'d>,
        syn_id_den: u32,
    }

    impl<'d> Synapse<'d> {
        /// Returns the index of this synapse within its layer.
        pub fn idx(&self) -> u32 {
            let cell = &self.den.tuft.cell;
            let tuft_info = &cell.layer.tuft_info[self.den.tuft.tuft_id];
            syn_idx(&cell.layer.dims(), cell.slc_id_lyr, cell.v_id, cell.u_id,
                tuft_info.tft_syn_idz, &tuft_info.dims, self.den.den_id, self.syn_id_den)
        }
    }


    /// A dendrite map.
    #[derive(Debug)]
    pub struct Dendrite<'t> {
        tuft: &'t Tuft<'t>,
        den_id: u32,
    }

    impl<'t> Dendrite<'t> {
        /// Returns the index of this dendrite within its layer.
        pub fn idx(&self) -> u32 {
            let cell = &self.tuft.cell;
            let tuft_info = &cell.layer.tuft_info[self.tuft.tuft_id];
            den_idx(&cell.layer.dims(), cell.slc_id_lyr, cell.v_id, cell.u_id,
                tuft_info.tft_den_idz, &tuft_info.dims, self.den_id)
        }

        /// Returns a synapse map corresponding to the synapse within the
        /// dendrite specified by `syn_id`.
        pub fn synapse<'d>(&'d self, syn_id_den: u32) -> Synapse<'d> {
            assert!(syn_id_den < self.tuft.cell.layer.tuft_info[self.tuft.tuft_id].dims.syns_per_den());
            Synapse { den: self, syn_id_den }
        }

        /// Returns a synapse map corresponding to the synapse within the
        /// dendrite without checking whether or not `syn_id_den` is valid.
        #[inline]
        pub unsafe fn synapse_unchecked<'d>(&'d self, syn_id_den: u32) -> Synapse<'d> {
            Synapse { den: self, syn_id_den }
        }
    }


    /// A tuft map.
    #[derive(Debug)]
    pub struct Tuft<'c> {
        cell: &'c Cell<'c>,
        tuft_id: usize,
    }

    impl<'c> Tuft<'c> {
        /// Returns the index of this cell-tuft within its layer.
        pub fn idx(&self) -> u32 {
            (self.tuft_id as u32 * self.cell.layer.cell_count()) + self.cell.idx()
        }

        /// Returns a dendrite map corresponding to the dendrite within the
        /// cell-tuft specified by `den_id`.
        pub fn dendrite<'t>(&'t self, den_id: u32) -> Dendrite<'t> {
            assert!(den_id < self.cell.layer.tuft_info[self.tuft_id].dims.dens_per_tft());
            Dendrite { tuft: self, den_id }
        }

        /// Returns a dendrite map corresponding to the dendrite within the
        /// cell-tuft without checking whether or not `den_id` is valid.
        #[inline]
        pub unsafe fn dendrite_unchecked<'t>(&'t self, den_id: u32) -> Dendrite<'t> {
            Dendrite { tuft: self, den_id }
        }

        // /// Iterate through all synapses (with custom iterator).
        // pub fn synapses()

        /// Returns the dimensions of this tuft.
        pub fn dims(&self) -> &TuftDims {
            &self.cell.layer.tuft_info[self.tuft_id].dims
        }

        /// Returns the tuft id.
        pub fn tuft_id(&self) -> usize {
            self.tuft_id
        }
    }


    /// A cell map.
    #[derive(Debug)]
    pub struct Cell<'m> {
        layer: &'m DataCellLayerMap,
        slc_id_lyr: SlcId,
        v_id: u32,
        u_id: u32,
    }

    impl<'m> Cell<'m> {
        /// Returns the index of the cell within its layer.
        pub fn idx(&self) -> u32 {
            cmn::cel_idx_3d(self.layer.depth, self.slc_id_lyr, self.layer.slice_dims.v_size(),
                self.v_id, self.layer.slice_dims.u_size(), self.u_id)
        }

        /// Returns the index of the cell's axon within axon space.
        pub fn axon_idx(&self) -> u32 {
            let slc_axon_idz = (self.slc_id_lyr as u32 * self.layer.slice_dims.columns()) + self.layer.axon_idz;
            axon_idx(slc_axon_idz, self.layer.depth, self.layer.slice_idz,
                self.layer.slice_dims.v_size(), self.layer.slice_dims.v_scale(), self.v_id, 0,
                self.layer.slice_dims.u_size(), self.layer.slice_dims.u_scale(), self.u_id, 0).unwrap()
        }

        /// Returns a tuft map.
        pub fn tuft<'c>(&'c self, tuft_id: usize) -> Tuft<'c> {
            assert!(tuft_id < self.layer.tuft_count());
            Tuft { cell: self, tuft_id, }
        }

        /// Returns a tuft map without checking whether or not `tuft_id` is valid.
        #[inline]
        pub unsafe fn tuft_unchecked<'c>(&'c self, tuft_id: usize) -> Tuft<'c> {
            Tuft { cell: self, tuft_id, }
        }

        /// Returns a proximal (basal) tuft.
        ///
        /// If multiple proximal (basal) tufts are defined, the tuft returned
        /// could be any one of them.
        pub fn tuft_proximal<'c>(&'c self) -> Option<Tuft<'c>> {
            self.layer.tuft_ids.proximal.map(|tuft_id| self.tuft(tuft_id))
        }

        /// Returns a distal (basal) tuft.
        ///
        /// If multiple distal (basal) tufts are defined, the tuft returned
        /// could be any one of them.
        pub fn tuft_distal<'c>(&'c self) -> Option<Tuft<'c>> {
            self.layer.tuft_ids.distal.map(|tuft_id| self.tuft(tuft_id))
        }

        /// Returns an apical (distal) tuft.
        ///
        /// If multiple apical (distal) tufts are defined, the tuft returned
        /// could be any one of them.
        pub fn tuft_apical<'c>(&'c self) -> Option<Tuft<'c>> {
            self.layer.tuft_ids.apical.map(|tuft_id| self.tuft(tuft_id))
        }

        /// Returns the tuft info for this cellular layer.
        pub fn tuft_info(&self) -> &[TuftInfo] {
            self.layer.tuft_info()
        }

        /// Returns this cell's slice id *within* its layer.
        pub fn slc_id_lyr(&self) -> SlcId {
            self.slc_id_lyr
        }

        /// Returns this cell's 'v' coordinate.
        pub fn v_id(&self) -> u32 {
            self.v_id
        }

        /// Returns this cell's 'u' coordinate.
        pub fn u_id(&self) -> u32 {
            self.u_id
        }
    }


    /// Information pertaining to indexing within a tuft.
    #[derive(Clone, Debug)]
    pub struct TuftInfo {
        dims: TuftDims,
        tft_den_idz: u32,
        tft_syn_idz: u32,
        den_class: DendriteClass,
        den_kind: DendriteKind,
    }

    impl TuftInfo {
        pub fn den_class(&self) -> DendriteClass {
            self.den_class
        }

        pub fn den_kind(&self) -> DendriteKind {
            self.den_kind
        }

        pub fn dims(&self) -> &TuftDims {
            &self.dims
        }
    }


    #[derive(Debug)]
    struct TuftIds {
        proximal: Option<usize>,
        distal: Option<usize>,
        apical: Option<usize>,
    }


    /// The guts of a `DataCellLayerMap`.
    #[derive(Debug)]
    pub struct Inner {
        layer_addr: LayerAddress,
        slice_dims: SliceDims,
        depth: SlcId,
        axon_idz: u32,
        slice_idz: SlcId,
        tuft_info: Vec<TuftInfo>,
        tuft_ids: TuftIds,
        den_count: u32,
        syn_count: u32,
    }


    /// A stand-alone map able to resolve the index of any cell component
    /// within a data cell layer (tufts, dendrites, synapses, etc.).
    #[derive(Clone, Debug)]
    pub struct DataCellLayerMap {
        inner: Arc<Inner>,
    }

    impl DataCellLayerMap {
        /// Creates and returns a new `DataCellLayerMap`.
        pub fn from_names(area_name: &str, layer_name: &str, thal: &mut Thalamus) -> DataCellLayerMap {
            let layer_addr = thal.layer_addr(area_name, layer_name);
            let area_map = &thal.area_maps()[layer_addr.area_id()];
            let dims = area_map.layer_dims(layer_addr.layer_id())
                .expect(&format!("DataCellLayerMap::from_names: Invalid data cell layer ('{}'). \
                    Layer must have an output or local axon domain (non-input).", layer_addr));

            let layer_info = area_map.layer_map().layer_info(layer_addr.layer_id()).unwrap();
            let layer_slc_range = layer_info.slc_range().cloned()
                .expect(&format!("DataCellLayerMap::from_names: The specified layer ('{}') \
                    has no slices.", layer_addr));

            debug_assert!(layer_slc_range.start <= SlcId::max_value() as usize);
            let slice_idz = layer_slc_range.start as SlcId;
            let axon_idz = area_map.slice_map().axon_idzs()[slice_idz as usize];
            let mut slice_dims = None;

            for (i, slc_id) in layer_slc_range.clone().enumerate() {
                let sd_i = &area_map.slice_map().dims()[slc_id as usize];

                match slice_dims {
                    // Ensure each slice in the layer is equal:
                    Some(ref sd_0) => debug_assert!(sd_0 == sd_i),
                    None => slice_dims = Some(sd_i.clone()),
                }

                // Ensure axon idz calculations for each slice are correct:
                debug_assert!(area_map.slice_map().axon_idzs()[slc_id] ==
                    (dims.columns() * i as u32) + axon_idz);
            }

            let slice_dims = slice_dims.unwrap();
            debug_assert!(slice_dims.v_size() == dims.v_size() &&
                slice_dims.u_size() == dims.u_size());

            let cell_scheme = layer_info.kind().cell_scheme().unwrap();
            let tuft_count = cell_scheme.tft_count();
            let mut den_count_ttl = 0u32;
            let mut syn_count_ttl = 0u32;

            let mut tuft_id_proximal = None;
            let mut tuft_id_distal = None;
            let mut tuft_id_apical = None;

            // Determine tuft dims and den/syn idzs:
            let tuft_info = cell_scheme.tft_schemes().iter().enumerate().map(|(t_id, ts)| {
                debug_assert!(t_id == ts.tft_id());

                let tft_den_idz = den_count_ttl;
                let tft_den_count = dims.cells() * ts.dens_per_tft();
                debug_assert!(tft_den_count > 0);
                den_count_ttl += tft_den_count;

                let tft_syn_idz = syn_count_ttl;
                let tft_syn_count = dims.cells() * ts.syns_per_tft();
                debug_assert!(tft_syn_count > 0);
                syn_count_ttl += tft_syn_count;

                let den_class = ts.den_class();
                let den_kind = ts.den_kind();

                match den_class {
                    DendriteClass::Basal => match den_kind {
                        DendriteKind::Proximal => tuft_id_proximal = Some(ts.tft_id()),
                        DendriteKind::Distal => tuft_id_distal = Some(ts.tft_id()),
                        _ => unimplemented!(),
                    },
                    DendriteClass::Apical => match den_kind {
                        DendriteKind::Proximal => panic!("Unable to handle proximal apical tufts."),
                        DendriteKind::Distal => tuft_id_apical = Some(ts.tft_id()),
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                }

                TuftInfo {
                    dims: TuftDims::new(ts.dens_per_tft(), ts.syns_per_den()),
                    tft_den_idz,
                    tft_syn_idz,
                    den_class,
                    den_kind,
                }
            }).collect::<Vec<_>>();

            DataCellLayerMap {
                inner: Arc::new(Inner {
                    layer_addr,
                    slice_dims,
                    depth: dims.depth(),
                    axon_idz,
                    slice_idz,
                    tuft_info,
                    tuft_ids: TuftIds {
                        proximal: tuft_id_proximal,
                        distal: tuft_id_distal,
                        apical: tuft_id_apical,
                    },
                    den_count: den_count_ttl,
                    syn_count: syn_count_ttl,
                })
            }
        }

        // /// Returns the first tuft that matches the specified parameters.
        // fn matching_tuft<'c>(&'c self, class: DendriteClass, kind: DendriteKind) -> Option<Tuft<'c>> {
        //     self.tuft_info().iter().enumerate().find(|(tuft_id, tuft_info)| {
        //         tuft_info.den_class() == class && tuft_info.den_kind() == kind
        //     }).map(|(tuft_id, _)| self.tuft(tuft_id))
        // }

        /// Returns a cell map.
        pub fn cell<'m>(&'m self, slc_id_lyr: SlcId, v_id: u32, u_id: u32) -> Cell<'m> {
            assert!(slc_id_lyr < self.depth && v_id < self.slice_dims.v_size() &&
                u_id < self.slice_dims.u_size(), "Cell coordinates out of range: \
                slc_id_lyr: {} ({}), v_id: {} ({}), u_id: {} ({})", slc_id_lyr, self.depth,
                v_id, self.slice_dims.v_size(), u_id, self.slice_dims.u_size());
            Cell { layer: self, slc_id_lyr, v_id, u_id, }
        }

        /// Returns a cell map without checking whether or not the coordinates
        /// given are valid.
        #[inline]
        pub unsafe fn cell_unchecked<'m>(&'m self, slc_id_lyr: SlcId, v_id: u32, u_id: u32) -> Cell<'m> {
            Cell { layer: self, slc_id_lyr, v_id, u_id, }
        }

        /// Returns the address of this layer.
        pub fn layer_addr(&self) -> LayerAddress {
            self.layer_addr
        }

        /// Returns the tuft info for cells in this layer.
        pub fn tuft_info(&self) -> &[TuftInfo] {
            &self.tuft_info
        }

        pub fn cell_count(&self) -> u32 {
            self.depth as u32 * self.slice_dims.v_size() * self.slice_dims.u_size()
        }

        /// Returns the number of tufts for cells in this layer.
        pub fn tuft_count(&self) -> usize {
            self.tuft_info.len()
        }

        /// Returns the layer dimensions.
        pub fn dims(&self) -> CorticalDims {
            CorticalDims::new(self.depth, self.slice_dims.v_size(), self.slice_dims.u_size())
        }

        /// Returns the index of the 0th cell's axon within axon space.
        pub fn axon_idz(&self) -> u32 {
            self.axon_idz
        }

        /// Returns the total number of dendrites in every slice, cell, and tuft.
        pub fn den_count(&self) -> u32 {
            self.den_count
        }

        /// Returns the total number of synapses in every slice, cell, and tuft.
        pub fn syn_count(&self) -> u32 {
            self.syn_count
        }
    }

    impl Deref for DataCellLayerMap {
        type Target = Arc<Inner>;

        /// Implemented for convenience (to avoid having to `.inner`
        /// everywhere).
        fn deref(&self) -> &Arc<Inner> {
            &(*self).inner
        }
    }


    // #[derive(Debug, Clone)]
    // pub struct TftCoords {
    //     pub idx: u32,
    //     pub tft_id: usize,
    //     pub slc_id_lyr: u8,
    //     pub axon_slc_id: u8,
    //     pub v_id: u32,
    //     pub u_id: u32,
    //     pub lyr_dims: CorticalDims,
    //     // pub tfts_per_cel: u32,
    //     // pub dens_per_tft_l2: u8,
    //     // pub syns_per_den_l2: u8,
    // }

    // impl TftCoords {
    //     pub fn new(tft_id: usize, axon_slc_id: u8, slc_id_lyr: u8, v_id: u32, u_id: u32,
    //                 lyr_dims: CorticalDims, /*tfts_per_cel: u32, dens_per_tft_l2: u8,
    //                 syns_per_den_l2: u8*/) -> TftCoords
    //     {
    //         let idx_tft = cmn::cel_idx_3d(dims.depth(), slc_id_lyr, dims.v_size(),
    //             v_id, dims.u_size(), u_id);

    //         let idx = ((tft_id as u32) * dims.cells()) + idx_tft;

    //         TftCoords {
    //             idx: idx,
    //             tft_id: tft_id,
    //             slc_id_lyr: slc_id_lyr,
    //             axon_slc_id: axon_slc_id,
    //             v_id: v_id,
    //             u_id: u_id,
    //             lyr_dims: dims,
    //             // tfts_per_cel: tfts_per_cel,
    //             // dens_per_tft_l2: dens_per_tft_l2,
    //             // syns_per_den_l2: syns_per_den_l2,
    //         }
    //     }

    //     pub fn idx(&self) -> u32 {
    //         self.idx
    //     }

    //     pub fn col_id(&self) -> u32 {
    //         // Fake a slice id of 0 with a slice depth of 1 and ignore our actual depth and id:
    //         cmn::cel_idx_3d(1, 0, self.lyr_dims.v_size(), self.v_id,
    //             self.lyr_dims.u_size(), self.u_id)
    //     }

    //     pub fn cel_axon_idx(&self, area_map: &AreaMap) -> u32 {
    //         area_map.axon_idx(self.axon_slc_id, self.v_id, 0, self.u_id, 0).unwrap()
    //     }
    // }

    // impl Display for TftCoords {
    //     fn fmt(&self, fmtr: &mut Formatter) -> Result {
    //         write!(fmtr, "TftCoords {{ idx: {}, slc_id_lyr: {}, axon_slc_id: {}, v_id: {}, u_id: {} }}",
    //             self.idx, self.slc_id_lyr, self.axon_slc_id, self.v_id, self.u_id)
    //     }
    // }
}
