use std::ops;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ self, ThreadRng, Rng };
use num::{ self, Integer };
use std::default::{ Default };
use std::fmt::{ Display };

use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ Protoareas };
use proto::regions::{ Protoregion, ProtoregionKind };
use proto::layer:: { Protolayer };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use cortical_area:: { Aux };
use peak_column:: { PeakColumns };
use pyramidals::{ PyramidalCellularLayer };
use minicolumns::{ Minicolumns };




/*	Minicolumns (aka. Columns)
	- TODO:
		- Reorganization to:
			- Minicolumns
				- SpinyStellateCellularLayer
					- Dendrite

*/
pub struct SpinyStellateCellularLayer {
	dims: CorticalDimensions,
	//axn_output_slice: u8,
	//kern_cycle: ocl::Kernel,
	//kern_post_inhib: ocl::Kernel,
	//kern_output: ocl::Kernel,
	kern_ltp: ocl::Kernel,
	rng: rand::XorShiftRng,
	//regrow_counter: usize,	// SLATED FOR REMOVAL
	//pub states: Envoy<ocl::cl_uchar>,
	//pub states_raw: Envoy<ocl::cl_uchar>,
	//pub cels_status: Envoy<ocl::cl_uchar>,
	//pub best_col_den_states: Envoy<ocl::cl_uchar>,
	//pub peak_spis: PeakColumns,
	//pub syns: ColumnSynapses,
	pub dens: Dendrites,
	//pub syns: Synapses,
}

// pyrs: &PyramidalCellularLayer,
impl SpinyStellateCellularLayer {
	pub fn new(name: &'static str, dims: CorticalDimensions, region: &Protoregion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> SpinyStellateCellularLayer {
		let layer = region.col_input_layer().expect("spiny_stellates::SpinyStellateCellularLayer::new()");
		//let depth: u8 = layer.depth();

		let syns_per_den_l2 = cmn::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
		//let syns_per_cel: u32 = 1 << syns_per_den_l2;

		//let pyr_depth = region.depth_cell_kind(&ProtocellKind::Pyramidal);

		//println!("\n#######   pyr_depth: {}   ########", pyr_depth);
		//let pyr_axn_base_slice = region.base_slice_cell_kind(&ProtocellKind::Pyramidal); // SHOULD BE SPECIFIC LAYER(S)  

		//let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let states_raw = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		print!("\n##### SPINY STELLATE dims: {:?}", dims);

		let dens = Dendrites::new(dims, DendriteKind::Proximal, ProtocellKind::SpinyStellate, region, axons, aux, ocl);

		//let cels_status = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let best_col_den_states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		//let peak_spis = PeakColumns::new(dims, region, &dens.states, ocl);

		/*let syns = Synapses::new(dims, syns_per_den_l2, syns_per_den_l2, DendriteKind::Proximal, 
			ProtocellKind::SpinyStellateCellularLayer, region, axons, aux, ocl);*/

		/*let output_slices = region.col_output_slices();
		assert!(output_slices.len() == 1);
		let axn_output_slice = output_slices[0];*/


		/*let kern_cycle = ocl.new_kernel("den_cycle", WorkSize::TwoDim(depth as usize, dims.columns() as usize))
			.arg_env(&dens.syns.states)
			.arg_env(&dens.syns.strengths)
			.arg_scl(syns_per_den_l2)
			.arg_scl(cmn::DENDRITE_INITIAL_THRESHOLD_PROXIMAL)
			.arg_env(&states_raw)
			.arg_env(&states)
		;*/

		/*let kern_post_inhib = ocl.new_kernel("sst_post_inhib_unoptd", WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
			.arg_env(&peak_spis.spi_ids)
			.arg_env(&peak_spis.states)
			.arg_env(&peak_spis.wins)
			.arg_scl(layer.base_slice_pos() as u32)
			.arg_env(&dens.states)
			.arg_env(&axons.states)
		;*/

		/*let kern_output = ocl.new_kernel("col_output", WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
			//.lws(WorkSize::TwoDim(1 as usize, cmn::AXONS_WORKGROUP_SIZE as usize))
			.arg_env(&dens.states)
			.arg_env(&pyrs.preds)
			.arg_env(&pyrs.best1_den_states)
			//.arg_scl(depth)
			.arg_scl(pyr_depth)
			//.arg_scl(pyr_axn_base_slice)
			.arg_scl(axn_output_slice)
			.arg_env(&cels_status)
			.arg_env(&best_col_den_states)
			.arg_env(&axons.states)
		;*/





// ***** BRING ME BACK *****
// ***** BRING ME BACK *****
		let kern_ltp = ocl.new_kernel("sst_ltp", WorkSize::TwoDim(dims.depth() as usize, 16 as usize))
		//let kern_ltp = ocl.new_kernel("sst_ltp", WorkSize::TwoDim(dims.depth() as usize, peak_spis.dims.per_slice() as usize))
			.arg_env(&dens.syns.states)
			.arg_env(&dens.syns.states)
			.arg_env(&dens.syns.states)
			.arg_scl(syns_per_den_l2 as u32)
			.arg_scl(0u32)
			//.arg_env(&aux.ints_0)
			.arg_env(&dens.syns.strengths)
			//.arg_env(&axons.states)
		;
// ***** BRING ME BACK *****
// ***** BRING ME BACK *****




		//println!("\n***Test");

		SpinyStellateCellularLayer {
			dims: dims,
			//axn_output_slice: axn_output_slice,
			//kern_cycle: kern_cycle,
			//kern_post_inhib: kern_post_inhib,
			//kern_output: kern_output,
			kern_ltp: kern_ltp,
			rng: rand::weak_rng(),
			//regrow_counter: 0usize,
			//states_raw: states_raw,
			//states: states,
			//cels_status: cels_status,
			//best_col_den_states: best_col_den_states,
			//peak_spis: peak_spis,
			dens: dens,
		}
	}

	pub fn cycle(&mut self, ltp: bool) {
		self.dens.cycle();
		//self.kern_cycle.enqueue(); 
		//self.peak_spis.cycle(); 
		//self.kern_post_inhib.enqueue(); 
		//if ltp { self.ltp(); }
	}

	/*pub fn output(&self) {
		self.kern_output.enqueue();
	}*/

	pub fn ltp(&mut self) {
		//print!("[R:{}]", self.rng.gen::<i32>());
		self.kern_ltp.set_kernel_arg(4, self.rng.gen::<u32>());
		self.kern_ltp.enqueue(); // <<<<< PROBLEM HERE -- MAYBE SOLVED (fixed peak_col size)
	}

	pub fn regrow(&mut self, region: &Protoregion) {
		self.dens.regrow(region);
	}

	pub fn confab(&mut self) {
		//self.states.read();
		//self.states_raw.read();
		//self.cels_status.read();
		//self.peak_spis.confab();
		self.dens.confab();
	} 

	/*pub fn axn_output_range(&self) -> (usize, usize) {
		//println!("self.axn_output_slice: {}, self.dims.columns(): {}, cmn::SYNAPSE_REACH_LIN: {}", self.axn_output_slice as usize, self.dims.columns() as usize, cmn::SYNAPSE_REACH_LIN);
		let start = (self.axn_output_slice as usize * self.dims.columns() as usize) + cmn::SYNAPSE_REACH_LIN as usize;
		(start, start + (self.dims.per_slice() - 1) as usize)
	}*/
}
