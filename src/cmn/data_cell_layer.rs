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
	fn axn_base_slc(&self) -> u8;
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
