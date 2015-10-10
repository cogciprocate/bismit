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
	fn cycle_self_only(&self);
	fn dims(&self) -> &CorticalDimensions;
	fn axn_range(&self) -> (usize, usize);
	fn base_axn_slc(&self) -> u8;
	fn layer_name(&self) -> &'static str;
	fn print_cel(&mut self, cel_idx: usize);
	fn set_all_to_zero(&mut self);
	fn protocell(&self) -> &Protocell;
	fn dens(&self) -> &Dendrites;
	fn dens_mut(&mut self) -> &mut Dendrites;
}


#[cfg(test)]
pub mod tests {
	use rand::{ XorShiftRng };
	// use rand::distributions::{ IndependentSample, Range };

	// use super::{ DataCellLayer };
	use cmn::{ self, CorticalDimensions };

	pub trait DataCellLayerTest {
		fn rng(&mut self) -> &mut XorShiftRng;
		fn rand_cel_coords(&mut self) -> CelCoords;
		fn cel_idx(&self, slc_id: u8, v_id: u32, u_id: u32)-> u32;
	}


	#[derive(Debug)]
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
	}
}
