use ocl::{ Envoy };
use dendrites::{ Dendrites };
use cmn::{ self, CorticalDimensions };
use proto::{ Protocell };

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
	fn axn_slc_base(&self) -> u8;
	fn layer_name(&self) -> &'static str;
	fn print_cel(&mut self, cel_idx: usize);
	fn set_all_to_zero(&mut self);
	fn protocell(&self) -> &Protocell;
	fn dens(&self) -> &Dendrites;
	fn dens_mut(&mut self) -> &mut Dendrites;

	fn cel_idx(&self, slc_id: u8, v_id: u32, u_id: u32)-> u32 {
		cmn::cel_idx_3d(self.dims().depth(), slc_id, self.dims().v_size(), v_id, self.dims().u_size(), u_id)
	}
}


#[cfg(test)]
pub mod tests {
	//use super::{ PyramidalLayer };
	use rand::distributions::{ IndependentSample, Range };
	use super::{ DataCellLayer };
	use rand;
	//use tests::{ testbed };
	//use cortex::{ Cortex };
	//use synapses::tests as syn_tests; 

	#[derive(Debug)]
	pub struct CelCoords {
		pub idx: u32,
		pub slc_id_layer: u8,
		pub v_id: u32,
		pub u_id: u32,		
	}

	impl CelCoords {
		pub fn new<D: DataCellLayer>(slc_id_layer: u8, v_id: u32, u_id: u32, cels: &Box<D>) -> CelCoords {
			let idx = cels.cel_idx(slc_id_layer, v_id, u_id);
			CelCoords { idx: idx, slc_id_layer: slc_id_layer, v_id: v_id, u_id: u_id }
		}

		pub fn new_random<D: DataCellLayer>(pyrs: &Box<D>) -> CelCoords {
			let slc_range = Range::new(0, pyrs.dims().depth());
			let v_range = Range::new(0, pyrs.dims().v_size());
			let u_range = Range::new(0, pyrs.dims().u_size());

			let mut rng = rand::weak_rng();

			let slc_id_layer = slc_range.ind_sample(&mut rng);
			let u_id = u_range.ind_sample(&mut rng);
			let v_id = v_range.ind_sample(&mut rng);

			CelCoords::new(slc_id_layer, v_id, u_id, pyrs)
		}

		pub fn idx(&self) -> u32 {
			self.idx
		}
	}
}
