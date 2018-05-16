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
    use map::{AreaMap, AreaMapTest, LayerAddress, axon_idx};
    use cmn::{self, CorticalDims, XorShiftRng, SliceDims};
    use cortex::TuftDims;
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


    /// A dendrite map.
    #[derive(Debug)]
    pub struct Dendrite<'t> {
        tuft: &'t Tuft<'t>,
        den_id: u32,
    }

    impl<'t> Dendrite<'t> {
        pub fn idx(&self) -> u32 {
            99999999
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
            assert!(den_id < self.cell.layer.tuft_dims[self.tuft_id].dens_per_tft());
            Dendrite { tuft: self, den_id }
        }
    }


    /// A cell map.
    #[derive(Debug)]
    pub struct Cell<'m> {
        layer: &'m DataCellLayerMap,
        slc_id_lyr: u8,
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
    }


    // /// Information pertaining to indexing within a tuft.
    // #[derive(Clone, Debug)]
    // struct TuftInfo {
    //     dens_per_tft: u32,
    //     syns_per_den: u32,
    // }


    /// The guts of a `DataCellLayerMap`.
    #[derive(Debug)]
    pub struct Inner {
        layer_addr: LayerAddress,
        slice_dims: SliceDims,
        depth: SlcId,
        axon_idz: u32,
        slice_idz: SlcId,
        tuft_dims: Vec<TuftDims>,
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
                .expect(&format!("DataCellLayerMap::from_names: Invalid layer name ('{}'). \
                    Layer name must be valid for the area and have an output or local axon \
                    domain (non-input).", layer_name));

            let layer_info = area_map.layer_map().layer_info(layer_addr.layer_id()).unwrap();
            let layer_slc_range = layer_info.slc_range().cloned()
                .expect(&format!("DataCellLayerMap::from_names: The specified layer ('{}') \
                    has no slices.", layer_name));

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

            let tuft_dims = cell_scheme.tft_schemes().iter().map(|ts| {
                TuftDims::new(ts.dens_per_tft(), ts.syns_per_den())
            }).collect::<Vec<_>>();

            DataCellLayerMap {
                inner: Arc::new(Inner {
                    layer_addr,
                    slice_dims,
                    depth: dims.depth(),
                    axon_idz,
                    slice_idz,
                    tuft_dims,
                })
            }
        }

        /// Returns a cell map.
        pub fn cell<'m>(&'m self, slc_id_lyr: SlcId, v_id: u32, u_id: u32) -> Cell<'m> {
            Cell {
                layer: self,
                slc_id_lyr,
                v_id,
                u_id,
            }
        }

        /// Returns the address of this layer.
        pub fn layer_addr(&self) -> LayerAddress {
            self.layer_addr
        }

        /// Returns the total number of cells in the layer.
        pub fn cell_count(&self) -> u32 {
            self.depth as u32 * self.slice_dims.columns()
        }

        pub fn tuft_count(&self) -> usize {
            self.tuft_dims.len()
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
