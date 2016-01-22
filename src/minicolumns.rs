use std::ops::Range;
use rand;

use cmn::{ self, CorticalDims, DataCellLayer };
use map::{ AreaMap };
use ocl::{ self, ProQue, WorkSize, Envoy, OclNum, EventList };
use axon_space::{ AxonSpace };
use pyramidals::{ PyramidalLayer };
use spiny_stellates::{ SpinyStellateLayer };

#[cfg(test)]
pub use self::tests::{ MinicolumnsTest };


pub struct Minicolumns {
	dims: CorticalDims,
	aff_out_axn_slc: u8,
	aff_out_axn_idz: u32,
	ff_layer_axn_idz: usize,
	kern_output: ocl::Kernel,
	kern_activate: ocl::Kernel,
	rng: rand::XorShiftRng,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	pub best_den_states: Envoy<ocl::cl_uchar>,
}

impl Minicolumns {
	pub fn new(dims: CorticalDims, area_map: &AreaMap, axons: &AxonSpace, 
				ssts: &SpinyStellateLayer, pyrs: &PyramidalLayer, ocl_pq: &ProQue
			) -> Minicolumns 
	{
		assert!(dims.depth() == 1);
		assert!(dims.v_size() == pyrs.dims().v_size() && dims.u_size() == pyrs.dims().u_size());

		// UPDATE ME TO AREA_MAP SETUP
		let ff_layer_axn_idz = ssts.axn_range().start;
		let pyr_depth = area_map.ptal_layer().depth();

		println!("{mt}{mt}MINICOLUMNS::NEW() dims: {:?}, pyr_depth: {}", dims, pyr_depth, mt = cmn::MT);

		let flag_sets = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl_pq.queue());
		let best_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl_pq.queue());

		// [FIXME]: TEMPORARY?:
		assert!(area_map.aff_out_slcs().len() == 1, 
			"Afferent output slices currently limited to a maximum of 1.");

		let aff_out_axn_slc = area_map.aff_out_slcs()[0];
		let aff_out_axn_idz = area_map.axn_idz(aff_out_axn_slc);
		let pyr_lyr_axn_idz = area_map.axn_idz(pyrs.base_axn_slc());

		let kern_activate = ocl_pq.create_kernel("mcol_activate_pyrs",
			WorkSize::ThreeDims(pyrs.dims().depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			.arg_env(&flag_sets)
			.arg_env(&best_den_states)
			.arg_env(&pyrs.best_den_states)
			.arg_scl(ff_layer_axn_idz as u32)
			.arg_scl(pyr_lyr_axn_idz)
			.arg_scl(pyrs.protocell().dens_per_tuft_l2)
			.arg_env(&pyrs.flag_sets)
			.arg_env(&pyrs.states)
			.arg_env_named::<i32>("aux_ints_0", None)
			// .arg_env_named::<i32>("aux_ints_1", None)
			.arg_env(&axons.states)
		;


		let kern_output = ocl_pq.create_kernel("mcol_output", 
			WorkSize::TwoDims(dims.v_size() as usize, dims.u_size() as usize))
			.arg_env(&pyrs.soma())
			.arg_scl(pyrs.tfts_per_cel())
			.arg_scl(ff_layer_axn_idz as u32)
			.arg_scl(pyr_depth)
			.arg_scl(aff_out_axn_slc)
			.arg_env(&pyrs.best_den_states)
			.arg_env(&flag_sets)
			.arg_env(&best_den_states)
			.arg_env(&axons.states)
		;


		Minicolumns {
			dims: dims,
			aff_out_axn_slc: aff_out_axn_slc,
			aff_out_axn_idz: aff_out_axn_idz,
			ff_layer_axn_idz: ff_layer_axn_idz,
			kern_output: kern_output,
			kern_activate: kern_activate,
			rng: rand::weak_rng(),
			flag_sets: flag_sets,
			best_den_states: best_den_states,
		}
	}

	pub fn set_arg_env_named<T: OclNum>(&mut self, name: &'static str, env: &Envoy<T>) {
		let activate_using_aux = true;
		let output_using_aux = false;

		if activate_using_aux {
			self.kern_activate.set_arg_env_named(name, env);
		}

		if output_using_aux {
			self.kern_output.set_arg_env_named(name, env);
		}
	}

	pub fn activate(&self) {
		self.kern_activate.enqueue(None, None);
	}

	pub fn output(&self, new_events: Option<&mut EventList>) {
		match new_events {
			Some(ne) => {
				ne.release_all();
				self.kern_output.enqueue(None, Some(ne));
			},

			None => self.kern_output.enqueue(None, None),
		}
	}

	pub fn confab(&mut self) {
		self.flag_sets.read_wait();
		self.best_den_states.read_wait();
	}

	pub fn ff_layer_axn_idz(&self) -> usize {
		self.ff_layer_axn_idz
	}

	pub fn aff_out_axn_slc(&self) -> u8 {
		self.aff_out_axn_slc
	}

	// AXN_OUTPUT_RANGE(): USED FOR TESTING / DEBUGGING PURPOSES
	pub fn aff_out_axn_range(&self) -> Range<usize> {
		self.aff_out_axn_idz as usize..self.aff_out_axn_idz as usize + self.dims.per_slc() as usize
	}
}


#[cfg(test)]
pub mod tests {
	use std::ops::Range;
	use super::Minicolumns;

	pub trait MinicolumnsTest {
		fn print_range(&mut self, range: Range<usize>);
		fn print_all(&mut self);
	}

	impl MinicolumnsTest for Minicolumns {
		fn print_range(&mut self, range: Range<usize>) {
			print!("mcols.flag_sets: ");
			self.flag_sets.print(1 << 0, Some((0, 255)), 
				Some(range.clone()), false);

			print!("mcols.best_den_states: ");
			self.best_den_states.print(1 << 0, Some((0, 255)), 
				Some(range.clone()), false);
		}

		fn print_all(&mut self) {
			let range = 0..self.flag_sets.len();
			self.print_range(range);
		}
	}

}
