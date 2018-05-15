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
    #![allow(dead_code)]

    use std::ops::{Range};
    // use rand::{XorShiftRng};

    use map::{AreaMap, AreaMapTest, axon_idx};
    use cmn::{self, CorticalDims, XorShiftRng};
    use std::fmt::{Display, Formatter, Result};
    use super::DataCellLayer;
    use ::Thalamus;

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



    pub struct Cell<'l> {
        layer: &'l DataCellLayerMap,
        slc_id_lyr: u8,
        v_id: u32,
        u_id: u32,
    }

    impl<'l> Cell<'l> {
        pub fn axon_idx(&self) -> u32 {
            1000
        }
    }


    /// A stand-alone map able to resolve the index of any cell component
    /// within a data cell layer (tufts, dendrites, synapses, etc.).
    #[derive(Clone, Debug)]
    pub struct DataCellLayerMap {
        layer_dims: CorticalDims,
    }

    impl DataCellLayerMap {
        pub fn from_names(area_name: &str, layer_name: &str, thal: &mut Thalamus) -> DataCellLayerMap {
            let layer_addr = thal.layer_addr(area_name, layer_name);
            let layer_dims = thal.area_maps()[layer_addr.area_id()].layer_dims(layer_addr.layer_id())
                .expect("DataCellLayerMap::from_names: Invalid layer name. Layer name must be \
                    valid for the area and have an output or local axon domain (non-input).");



            DataCellLayerMap {
                layer_dims,
            }
        }

        pub fn cell<'l>(&'l self, coords: (u8, u32, u32)) -> Cell<'l> {
            Cell {
                layer: self,
                slc_id_lyr: coords.0,
                v_id: coords.1,
                u_id: coords.2,
            }
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
