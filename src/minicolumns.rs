use std::ops;
use std::mem;
use std::collections::{ HashMap };
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ self, ThreadRng, Rng };
use num::{ self, Integer };
use std::default::{ Default };
use std::fmt::{ Display };

use cmn;
use ocl::{ self, OclProgQueue, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ Protoareas };
use proto::regions::{ Protoregion, ProtoregionKind };
use proto::layer:: { Protolayer };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use cortical_area:: { Aux };
use iinn:: { InhibitoryInterneuronNetwork };
use pyramidals::{ PyramidalCellularLayer };
use spiny_stellates::{ SpinyStellateCellularLayer };




/*	Minicolumns (aka. Columns)
	- TODO:
		- Reorganization to:
			- Minicolumns
				- SpinyStellate
					- Dendrite

*/
pub struct Minicolumns {
	dims: CorticalDimensions,
	axn_output_slc: u8,
	ff_layer_axn_idz: usize,
	//kern_cycle: ocl::Kernel,
	//kern_post_inhib: ocl::Kernel,
	kern_output: ocl::Kernel,
	//kern_ltp: ocl::Kernel,
	rng: rand::XorShiftRng,
	//regrow_counter: usize,	// SLATED FOR REMOVAL
	//pub states: Envoy<ocl::cl_uchar>,
	//pub states_raw: Envoy<ocl::cl_uchar>,
	pub pred_totals: Envoy<ocl::cl_uchar>,
	pub best_pyr_den_states: Envoy<ocl::cl_uchar>,
	//pub iinn: InhibitoryInterneuronNetwork,
	//pub syns: ColumnSynapses,
	//pub dens: Dendrites,
	//pub syns: Synapses,
}

impl Minicolumns {
	pub fn new(dims: CorticalDimensions, protoregion: &Protoregion, axons: &Axons, 

					/*ssts_map: &HashMap<&str, Box<SpinyStellateCellularLayer>>, pyrs_map: &HashMap<&str, Box<PyramidalCellularLayer>>, */

					sstl: &SpinyStellateCellularLayer, 
					pyrs: &PyramidalCellularLayer,

					aux: &Aux, ocl: &OclProgQueue) -> Minicolumns {

		assert!(dims.depth() == 1);

		/*let psal_name = cortex.area_mut("v1").psal_name();
		let ptal_name = cortex.area_mut("v1").ptal_name();*/

		let layer = protoregion.spt_asc_layer().expect("minicolumns::Minicolumns::new()");
		//let depth: u8 = layer.depth();

		/*let ssts = ssts_map.get(psal_name).expect("minicolumns.rs");
		let pyrs = pyrs_map.get(ptal_name).expect("minicolumns.rs");*/
		//let syns_per_den_l2 = cmn::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
		//let syns_per_tuft: u32 = 1 << syns_per_den_l2;

		let (ff_layer_axn_idz, _) = sstl.axn_range();

		let pyr_depth = protoregion.depth_cell_kind(&ProtocellKind::Pyramidal);

		//let pyr_axn_base_slc = protoregion.base_slc_cell_kind(&ProtocellKind::Pyramidal); // SHOULD BE SPECIFIC LAYER(S)  

		//let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let states_raw = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		print!("\n      MINICOLUMNS::NEW() dims: {:?}, pyr_depth: {}", dims, pyr_depth);

		//let dens = Dendrites::new(dims, DendriteKind::Proximal, ProtocellKind::SpinyStellate, protoregion, axons, aux, ocl);

		let pred_totals = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let best_pyr_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);

		//let iinn = InhibitoryInterneuronNetwork::new(dims, protoregion, &sstl.soma(), ocl);

		/*let syns = Synapses::new(dims, syns_per_den_l2, syns_per_den_l2, DendriteKind::Proximal, 
			ProtocellKind::SpinyStellate, protoregion, axons, aux, ocl);*/

		let (sstl_axn_idz, _) = sstl.axn_range();
		//let axn_output_slc = sstl.base_axn_slc();


		let axn_output_slc = protoregion.aff_out_slcs()[0];


		/*let output_slcs = protoregion.aff_out_slcs();
		assert!(output_slcs.len() == 1);
		let axn_output_slc = output_slcs[0];
		let ssts_slc_ids = protoregion.slc_ids(vec!["iv_old"]);
		let ssts_axn_base_slc = ssts_slc_ids[0];
		let ssts_axn_idz_old = cmn::axn_idx_2d(ssts_axn_base_slc, dims.columns(), protoregion.hrz_demarc());
		assert!(ssts_axn_idz == ssts_axn_idz_old as usize);*/

		//println!("\n ##### ssts_axn_idz: {}", ssts_axn_idz);

		let kern_output = ocl.new_kernel("col_output".to_string(), 
			WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
			//.lws(WorkSize::TwoDim(1 as usize, cmn::AXONS_WORKGROUP_SIZE as usize))
			//.arg_env(&sstl.soma())
			.arg_env(&pyrs.soma())
			.arg_env(&pyrs.best_den_states)
			//.arg_scl(depth)
			.arg_scl(sstl_axn_idz as u32)
			.arg_scl(pyr_depth)
			//.arg_scl(pyr_axn_base_slc)
			.arg_scl(axn_output_slc)
			.arg_env(&pred_totals)
			.arg_env(&best_pyr_den_states)
			.arg_env(&axons.states)
		;


		Minicolumns {
			dims: dims,
			axn_output_slc: axn_output_slc,
			ff_layer_axn_idz: ff_layer_axn_idz,
			//kern_cycle: kern_cycle,
			//kern_post_inhib: kern_post_inhib,
			kern_output: kern_output,
			//kern_ltp: kern_ltp,
			rng: rand::weak_rng(),
			//regrow_counter: 0usize,
			//states_raw: states_raw,
			//states: states,
			pred_totals: pred_totals,
			best_pyr_den_states: best_pyr_den_states,
			//iinn: iinn,
			//dens: dens,
		}
	}

	/*pub fn cycle(&mut self, ltp: bool) {
		self.iinn.cycle();  
		self.kern_post_inhib.enqueue(); 
	}*/

	pub fn output(&self) {
		self.kern_output.enqueue();
	}

	pub fn confab(&mut self) {
		//self.states.read();
		//self.states_raw.read();
		self.pred_totals.read();
		//self.iinn.confab();
		//self.sstl.dens.confab();
	}

	pub fn ff_layer_axn_idz(&self) -> usize {
		self.ff_layer_axn_idz
	}

	pub fn axn_output_range(&self) -> (usize, usize) {
		//println!("self.axn_output_slc: {}, self.dims.columns(): {}, cmn::SYNAPSE_REACH_LIN: {}", self.axn_output_slc as usize, self.dims.columns() as usize, cmn::SYNAPSE_REACH_LIN);
		let start = (self.axn_output_slc as usize * self.dims.columns() as usize) + cmn::SYNAPSE_REACH_LIN as usize;
		(start, start + self.dims.per_slc() as usize)
	}
}
