use std::ops;
// use std::mem;
// use std::collections::{ HashMap };
use rand::distributions::{ /*Normal, IndependentSample,*/ Range };
use rand::{ self, /*ThreadRng, Rng*/ };
// use num::{ self, Integer };
// use std::default::{ Default };
// use std::fmt::{ Display };

use cmn::{ self, CorticalDimensions, DataCellLayer };
use map::{ AreaMap };
use ocl::{ self, OclProgQueue, WorkSize, Envoy, OclNum };
use proto::{ /*ProtoLayerMap, RegionKind, ProtoAreaMaps,*/ ProtocellKind, /*Protocell, DendriteKind*/ };
// use synapses::{ Synapses };
// use dendrites::{ Dendrites };
use axon_space::{ AxonSpace };
// use cortical_area:: { Aux };
// use iinn:: { InhibitoryInterneuronNetwork };
use pyramidals::{ PyramidalLayer };
use spiny_stellates::{ SpinyStellateLayer };

#[cfg(test)]
pub use self::tests::{ MinicolumnsTest };



/*	Minicolumns (aka. Columns)
	- TODO:
		- Reorganization to:
			- Minicolumns
				- SpinyStellate
					- Dendrite

*/
pub struct Minicolumns {
	dims: CorticalDimensions,
	aff_out_axn_slc: u8,
	aff_out_axn_idz: u32,
	//hrz_demarc: u8,		// TEMPORARY
	ff_layer_axn_idz: usize,
	//kern_cycle: ocl::Kernel,
	//kern_post_inhib: ocl::Kernel,
	kern_output: ocl::Kernel,
	kern_activate: ocl::Kernel,
	//kern_ltp: ocl::Kernel,
	rng: rand::XorShiftRng,
	//regrow_counter: usize,	// SLATED FOR REMOVAL
	//pub states: Envoy<ocl::cl_uchar>,
	//pub states_raw: Envoy<ocl::cl_uchar>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	pub best_pyr_den_states: Envoy<ocl::cl_uchar>,
	//pub iinn: InhibitoryInterneuronNetwork,
	//pub syns: ColumnSynapses,
	//pub dens: Dendrites,
	//pub syns: Synapses,
}

impl Minicolumns {
	pub fn new(dims: CorticalDimensions, area_map: &AreaMap, axons: &AxonSpace, 

					/*ssts_map: &HashMap<&str, Box<SpinyStellateLayer>>, pyrs_map: &HashMap<&str, Box<PyramidalLayer>>, */

					ssts: &SpinyStellateLayer, 
					pyrs: &PyramidalLayer,

					/*aux: &Aux,*/ ocl: &OclProgQueue) -> Minicolumns {

		assert!(dims.depth() == 1);
		assert!(dims.v_size() == pyrs.dims().v_size() && dims.u_size() == pyrs.dims().u_size());

		/*let psal_name = cortex.area_mut("v1").psal_name();
		let ptal_name = cortex.area_mut("v1").ptal_name();*/

		let layer = area_map.proto_layer_map().spt_asc_layer().expect("minicolumns::Minicolumns::new()");
		//let depth: u8 = layer.depth();

		/*let ssts = ssts_map.get(psal_name).expect("minicolumns.rs");
		let pyrs = pyrs_map.get(ptal_name).expect("minicolumns.rs");*/
		//let syns_per_den_l2 = cmn::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
		//let syns_per_tuft: u32 = 1 << syns_per_den_l2;

		let ff_layer_axn_idz = ssts.axn_range().start;
		let pyr_depth = area_map.proto_layer_map().depth_cell_kind(&ProtocellKind::Pyramidal);

		//let pyr_base_axn_slc = area_map.proto_layer_map().base_slc_cell_kind(&ProtocellKind::Pyramidal); // SHOULD BE SPECIFIC LAYER(S)  

		//let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let states_raw = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		println!("      MINICOLUMNS::NEW() dims: {:?}, pyr_depth: {}", dims, pyr_depth);

		//let dens = Dendrites::new(dims, DendriteKind::Proximal, ProtocellKind::SpinyStellate, area_map.proto_layer_map(), axons, aux, ocl);

		let flag_sets = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best_pyr_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);

		//let iinn = InhibitoryInterneuronNetwork::new(dims, area_map.proto_layer_map(), &ssts.soma(), ocl);

		/*let syns = Synapses::new(dims, syns_per_den_l2, syns_per_den_l2, DendriteKind::Proximal, 
			ProtocellKind::SpinyStellate, area_map.proto_layer_map(), axons, aux, ocl);*/

		let aff_out_axn_slc = area_map.proto_layer_map().aff_out_slcs()[0];
		let aff_out_axn_idz = area_map.axn_idz(aff_out_axn_slc);

		/*let output_slcs = area_map.proto_layer_map().aff_out_slcs();
		assert!(output_slcs.len() == 1);
		let aff_out_axn_slc = output_slcs[0];
		let ssts_slc_ids = area_map.proto_layer_map().slc_ids(vec!["iv_old"]);
		let ssts_base_axn_slc = ssts_slc_ids[0];
		let ff_layer_axn_idz_old = cmn::axn_idz_2d(ssts_base_axn_slc, dims.columns(), area_map.proto_layer_map().hrz_demarc());
		assert!(ff_layer_axn_idz == ff_layer_axn_idz_old as usize);*/

		// REPLACE ME WITH AREAMAP GOODNESS
		//let (ff_layer_axn_idz, _) = ssts.axn_range();		

		let kern_activate = ocl.new_kernel("mcol_activate_pyrs".to_string(),
			WorkSize::ThreeDim(pyrs.dims().depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			.arg_env(&flag_sets)
			.arg_env(&best_pyr_den_states)
			.arg_env(&pyrs.tft_best_den_ids)
			.arg_env(&pyrs.dens.states)
			.arg_scl(ff_layer_axn_idz as u32)
			.arg_scl(pyrs.base_axn_slc())
			.arg_scl(pyrs.protocell().dens_per_tuft_l2)
			.arg_env(&pyrs.flag_sets)
			.arg_env(&pyrs.states)
			// .arg_env_named::<i32>("aux_ints_0", None)
			// .arg_env_named::<i32>("aux_ints_1", None)
			.arg_env(&axons.states)
		;

		//println!("\n ##### ff_layer_axn_idz: {}", ff_layer_axn_idz);

		let kern_output = ocl.new_kernel("mcol_output".to_string(), 
			// WorkSize::ThreeDim(1 as usize, dims.v_size() as usize, dims.u_size() as usize))
			WorkSize::TwoDim(dims.v_size() as usize, dims.u_size() as usize))
			//.arg_env(&ssts.soma())
			.arg_env(&pyrs.soma())
			.arg_env(&pyrs.tft_best_den_states)
			.arg_scl(pyrs.tfts_per_cel())
			.arg_scl(ff_layer_axn_idz as u32)
			.arg_scl(pyr_depth)
			//.arg_scl(pyr_base_axn_slc)
			.arg_scl(aff_out_axn_slc)
			.arg_env(&flag_sets)
			.arg_env(&best_pyr_den_states)
			// .arg_env_named::<i32>("aux_ints_0", None)
			// .arg_env_named::<i32>("aux_ints_1", None)
			.arg_env(&axons.states)
		;


		Minicolumns {
			dims: dims,
			aff_out_axn_slc: aff_out_axn_slc,
			aff_out_axn_idz: aff_out_axn_idz,
			//hrz_demarc: area_map.proto_layer_map().hrz_demarc(),
			ff_layer_axn_idz: ff_layer_axn_idz,
			//kern_cycle: kern_cycle,
			//kern_post_inhib: kern_post_inhib,
			kern_output: kern_output,
			kern_activate: kern_activate,
			//kern_ltp: kern_ltp,
			rng: rand::weak_rng(),
			//regrow_counter: 0usize,
			//states_raw: states_raw,
			//states: states,
			flag_sets: flag_sets,
			best_pyr_den_states: best_pyr_den_states,
			//iinn: iinn,
			//dens: dens,
		}
	}


	// pub fn init_kernels(&mut self, mcols: &Minicolumns, ssts: &Box<SpinyStellateLayer>, axns: &AxonSpace, aux: &Aux) {
	// 	let (ff_layer_axn_idz, _) = ssts.axn_range();
	// 	//println!("\n##### Pyramidals::init_kernels(): ff_layer_axn_idz: {}", ff_layer_axn_idz as u32);

	// 	println!("   PYRAMIDALS::INIT_KERNELS()[ACTIVATE]: ssts_axn_range(): {:?}", ssts.axn_range());

	// 	//self.kern_activate.new_arg_envoy(Some(&ssts.soma()));
	// 	self.kern_activate.new_arg_envoy(Some(&mcols.flag_sets));
	// 	self.kern_activate.new_arg_envoy(Some(&mcols.best_pyr_den_states));
	// 	self.kern_activate.new_arg_envoy(Some(&self.best_den_ids));
	// 	self.kern_activate.new_arg_envoy(Some(&self.dens.states));

	// 	self.kern_activate.new_arg_scalar(Some(ff_layer_axn_idz as u32));
	// 	self.kern_activate.new_arg_scalar(Some(self.base_axn_slc));
	// 	self.kern_activate.new_arg_scalar(Some(self.protocell.dens_per_tuft_l2));

	// 	//self.kern_activate.new_arg_envoy(&self.energies);
	// 	self.kern_activate.new_arg_envoy(Some(&self.flag_sets));
	// 	self.kern_activate.new_arg_envoy(Some(&self.preds));	
	// 	//self.kern_activate.new_arg_envoy(Some(&aux.ints_0));
	// 	self.kern_activate.new_arg_envoy(Some(&axns.states));
	// }

	/*pub fn cycle(&mut self, ltp: bool) {
		self.iinn.cycle();  
		self.kern_post_inhib.enqueue(); 
	}*/

	pub fn set_arg_env_named<T: OclNum>(&mut self, name: &'static str, env: &Envoy<T>) {
		let activate_using_aux = false;
		let output_using_aux = false;

		if activate_using_aux {
			self.kern_activate.set_arg_env_named(name, env);
		}

		if output_using_aux {
			self.kern_output.set_arg_env_named(name, env);
		}
	}

	pub fn activate(&self) {
		self.kern_activate.enqueue();
	}

	pub fn output(&self) {
		self.kern_output.enqueue();
	}

	pub fn confab(&mut self) {
		//self.states.read();
		//self.states_raw.read();
		self.flag_sets.read();
		//self.iinn.confab();
		//self.ssts.dens.confab();
	}

	pub fn ff_layer_axn_idz(&self) -> usize {
		self.ff_layer_axn_idz
	}

	// AXN_OUTPUT_RANGE(): USED FOR TESTING / DEBUGGING PURPOSES
	pub fn aff_out_axn_range(&self) -> ops::Range<usize> {
		//	println!("self.aff_out_axn_slc: {}, self.dims.columns(): {}, cmn::AXON_MAR__GIN_SIZE: {}", 
		//		self.aff_out_axn_slc as usize, self.dims.columns() as usize, cmn::AXON_MAR__GIN_SIZE);
		//let start = (self.aff_out_axn_slc as usize * self.dims.columns() as usize) + cmn::AXON_MAR__GIN_SIZE as usize;
		//let start = cmn::axn_idz_2d(self.aff_out_axn_slc, self.dims.columns(), self.hrz_demarc) as usize;		
		self.aff_out_axn_idz as usize..self.aff_out_axn_idz as usize + self.dims.per_slc() as usize
	}
}


#[cfg(test)]
pub mod tests {
	use std::ops::{ Range };
	use super::{ Minicolumns };

	pub trait MinicolumnsTest {
		fn print_range(&mut self, range: Range<usize>);
		fn print_all(&mut self);
	}

	impl MinicolumnsTest for Minicolumns {
		fn print_range(&mut self, range: Range<usize>) {
			print!("mcols.flag_sets: ");
			self.flag_sets.print(1 << 0, Some((0, 255)), 
				Some(range.clone()), false);

			print!("mcols.best_pyr_den_states: ");
			self.best_pyr_den_states.print(1 << 0, Some((0, 255)), 
				Some(range.clone()), false);
		}

		fn print_all(&mut self) {
			let range = 0..self.flag_sets.len();
			self.print_range(range);
		}
	}

}
