use ocl::{ Buffer, EventList };
use dendrites::{ Dendrites };
use cmn::{ /*self,*/ CorticalDims };
use proto::{ Protocell };

// #[cfg(test)]
// pub use self::tests::{ DataCellLayerTest };

pub trait DataCellLayer {
    fn learn(&mut self);
    fn regrow(&mut self);
    fn cycle(&self, Option<&EventList>);
    fn confab(&mut self);
    fn soma(&self) -> &Buffer<u8>;
    fn soma_mut(&mut self) -> &mut Buffer<u8>;
    fn dims(&self) -> &CorticalDims;
    fn axn_range(&self) -> (usize, usize);
    fn base_axn_slc(&self) -> u8;
    fn tfts_per_cel(&self) -> u32;
    fn layer_name(&self) -> &'static str;    
    fn protocell(&self) -> &Protocell;
    fn dens(&self) -> &Dendrites;
    fn dens_mut(&mut self) -> &mut Dendrites;
}


#[cfg(test)]
pub mod tests {
    use std::ops::{ Range };
    use rand::{ XorShiftRng };
    // use rand::distributions::{ IndependentSample, Range };

    // use super::{ DataCellLayer };
    use map::{ AreaMap, AreaMapTest };
    use cmn::{ self, CorticalDims };
    use std::fmt::{ Display, Formatter, Result };

    pub trait DataCellLayerTest {
        fn cycle_self_only(&self);
        fn print_cel(&mut self, cel_idx: usize);
        fn print_range(&mut self, range: Range<usize>, print_syns: bool);
        fn print_all(&mut self, print_syns: bool);
        fn rng(&mut self) -> &mut XorShiftRng;
        fn rand_cel_coords(&mut self) -> CelCoords;
        fn cel_idx(&self, slc_id: u8, v_id: u32, u_id: u32)-> u32;
        fn set_all_to_zero(&mut self);
    }


    #[derive(Debug, Clone)]
    pub struct CelCoords {
        pub idx: u32,
        pub slc_id_lyr: u8,
        pub axn_slc_id: u8,
        pub v_id: u32,
        pub u_id: u32,
        pub layer_dims: CorticalDims,    
        pub tfts_per_cel: u32,
        pub dens_per_tft_l2: u8,
        pub syns_per_den_l2: u8,
    }

    impl CelCoords {
        pub fn new(axn_slc_id: u8, slc_id_lyr: u8, v_id: u32, u_id: u32, 
                    dims: &CorticalDims, tfts_per_cel: u32, dens_per_tft_l2: u8,
                    syns_per_den_l2: u8) -> CelCoords 
        {
            let idx = cmn::cel_idx_3d(dims.depth(), slc_id_lyr, dims.v_size(), 
                v_id, dims.u_size(), u_id);

            CelCoords { 
                idx: idx, 
                slc_id_lyr: slc_id_lyr, 
                axn_slc_id: axn_slc_id,
                v_id: v_id, 
                u_id: u_id,
                layer_dims: dims.clone(),
                tfts_per_cel: tfts_per_cel,
                dens_per_tft_l2: dens_per_tft_l2,
                syns_per_den_l2: syns_per_den_l2,
            }
        }        

        pub fn idx(&self) -> u32 {
            self.idx
        }

        pub fn col_id(&self) -> u32 {
            // Fake a slice id of 0 with a slice depth of 1 and ignore our actual depth and id:
            cmn::cel_idx_3d(1, 0, self.layer_dims.v_size(), self.v_id, 
                self.layer_dims.u_size(), self.u_id)
        }

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
}
