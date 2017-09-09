//! Data Cells (Principal Neurons)
//!

use std::collections::BTreeMap;
use std::fmt::Debug;
use ocl::Buffer;
use cortex::{Dendrites, ControlCellLayer};
use cmn::{CmnResult, CorticalDims};
use map::{CellScheme, ExecutionGraph, LayerAddress};

#[cfg(test)]
pub use self::tests::{DataCellLayerTest, CelCoords};

pub trait DataCellLayer: 'static + Debug + Send {
    fn layer_name(&self) -> &'static str;
    fn layer_addr(&self) -> LayerAddress;
    fn cycle(&mut self, &mut BTreeMap<(LayerAddress, usize), Box<ControlCellLayer>>, &mut ExecutionGraph) -> CmnResult<()>;
    fn learn(&mut self, &mut ExecutionGraph) -> CmnResult <()> ;
    fn regrow(&mut self);
    fn soma(&self) -> &Buffer<u8>;
    fn soma_mut(&mut self) -> &mut Buffer<u8>;
    fn energies(&self) -> &Buffer<u8>;
    fn activities(&self) -> &Buffer<u8>;
    fn dims(&self) -> &CorticalDims;
    fn axn_range(&self) -> (usize, usize);
    fn axn_slc_ids(&self) -> &[u8];
    fn base_axn_slc(&self) -> u8;
    fn tft_count(&self) -> usize;
    fn cell_scheme(&self) -> &CellScheme;
    fn dens(&self) -> &Dendrites;
    fn dens_mut(&mut self) -> &mut Dendrites;
}


#[cfg(test)]
pub mod tests {
    use std::ops::{Range};
    // use rand::{XorShiftRng};

    use map::{AreaMap, AreaMapTest};
    use cmn::{self, CorticalDims, XorShiftRng};
    use std::fmt::{Display, Formatter, Result};

    pub trait DataCellLayerTest {
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
        pub axn_slc_id: u8,
        pub v_id: u32,
        pub u_id: u32,
        pub lyr_dims: CorticalDims,
        // pub tfts_per_cel: u32,
        // pub dens_per_tft_l2: u8,
        // pub syns_per_den_l2: u8,
    }

    impl CelCoords {
        pub fn new(axn_slc_id: u8, slc_id_lyr: u8, v_id: u32, u_id: u32,
                    lyr_dims: CorticalDims, /*tfts_per_cel: u32, dens_per_tft_l2: u8,
                    syns_per_den_l2: u8*/) -> CelCoords
        {
            let idx = cmn::cel_idx_3d(lyr_dims.depth(), slc_id_lyr, lyr_dims.v_size(),
                v_id, lyr_dims.u_size(), u_id);


            CelCoords {
                idx: idx,
                slc_id_lyr: slc_id_lyr,
                axn_slc_id: axn_slc_id,
                v_id: v_id,
                u_id: u_id,
                lyr_dims: lyr_dims,
                // tfts_per_cel: tfts_per_cel,
                // dens_per_tft_l2: dens_per_tft_l2,
                // syns_per_den_l2: syns_per_den_l2,
            }
        }

        pub fn idx(&self) -> u32 {
            self.idx
        }

        #[allow(dead_code)]
        pub fn col_id(&self) -> u32 {
            // Fake a slice id of 0 with a slice depth of 1 and ignore our actual depth and id:
            cmn::cel_idx_3d(1, 0, self.lyr_dims.v_size(), self.v_id,
                self.lyr_dims.u_size(), self.u_id)
        }

        #[allow(dead_code)]
        pub fn cel_axn_idx(&self, area_map: &AreaMap) -> u32 {
            area_map.axn_idx(self.axn_slc_id, self.v_id, 0, self.u_id, 0).unwrap()
        }
    }

    impl Display for CelCoords {
        fn fmt(&self, fmtr: &mut Formatter) -> Result {
            write!(fmtr, "CelCoords {{ idx: {}, slc_id_lyr: {}, axn_slc_id: {}, v_id: {}, u_id: {} }}",
                self.idx, self.slc_id_lyr, self.axn_slc_id, self.v_id, self.u_id)
        }
    }


    // #[derive(Debug, Clone)]
    // pub struct TftCoords {
    //     pub idx: u32,
    //     pub tft_id: usize,
    //     pub slc_id_lyr: u8,
    //     pub axn_slc_id: u8,
    //     pub v_id: u32,
    //     pub u_id: u32,
    //     pub lyr_dims: CorticalDims,
    //     // pub tfts_per_cel: u32,
    //     // pub dens_per_tft_l2: u8,
    //     // pub syns_per_den_l2: u8,
    // }

    // impl TftCoords {
    //     pub fn new(tft_id: usize, axn_slc_id: u8, slc_id_lyr: u8, v_id: u32, u_id: u32,
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
    //             axn_slc_id: axn_slc_id,
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

    //     pub fn cel_axn_idx(&self, area_map: &AreaMap) -> u32 {
    //         area_map.axn_idx(self.axn_slc_id, self.v_id, 0, self.u_id, 0).unwrap()
    //     }
    // }

    // impl Display for TftCoords {
    //     fn fmt(&self, fmtr: &mut Formatter) -> Result {
    //         write!(fmtr, "TftCoords {{ idx: {}, slc_id_lyr: {}, axn_slc_id: {}, v_id: {}, u_id: {} }}",
    //             self.idx, self.slc_id_lyr, self.axn_slc_id, self.v_id, self.u_id)
    //     }
    // }
}
