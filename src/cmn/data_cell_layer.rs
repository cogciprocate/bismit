use ocl::{ Envoy };
use dendrites::{ Dendrites };
use cmn::{ /*self,*/ CorticalDimensions };
use proto::{ Protocell };

// #[cfg(test)]
// pub use self::tests::{ DataCellLayerTest };

pub trait DataCellLayer {
	fn learn(&mut self);
	fn regrow(&mut self);
	fn cycle(&mut self);
	fn confab(&mut self);
	fn soma(&self) -> &Envoy<u8>;
	fn soma_mut(&mut self) -> &mut Envoy<u8>;
	fn dims(&self) -> &CorticalDimensions;
	fn axn_range(&self) -> (usize, usize);
	fn base_axn_slc(&self) -> u8;
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
	use cmn::{ self, CorticalDimensions };
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
		pub slc_id_axn: u8,
		pub v_id: u32,
		pub u_id: u32,
		pub layer_dims: CorticalDimensions,	
	}

	impl CelCoords {
		pub fn new(slc_id_axn: u8, slc_id_lyr: u8, v_id: u32, u_id: u32, 
				dims: &CorticalDimensions) -> CelCoords 
		{
			let idx = cmn::cel_idx_3d(dims.depth(), slc_id_lyr, dims.v_size(), 
				v_id, dims.u_size(), u_id);

			CelCoords { 
				idx: idx, 
				slc_id_lyr: slc_id_lyr, 
				slc_id_axn: slc_id_axn,
				v_id: v_id, 
				u_id: u_id,
				layer_dims: dims.clone() }
		}		

		pub fn idx(&self) -> u32 {
			self.idx
		}

		pub fn col_id(&self) -> u32 {
			cmn::cel_idx_3d(1, 0, self.layer_dims.v_size(), self.v_id, 
				self.layer_dims.u_size(), self.u_id)
		}

		pub fn print(&self) {
			
		}
	}

	impl Display for CelCoords {
	    fn fmt(&self, fmtr: &mut Formatter) -> Result {
	        write!(fmtr, "CelCoords {{ idx: {}, slc_id_lyr: {}, slc_id_axn: {}, v_id: {}, u_id: {} }}", 
				self.idx, self.slc_id_lyr, self.slc_id_axn, self.v_id, self.u_id)
	    }
	}
}
