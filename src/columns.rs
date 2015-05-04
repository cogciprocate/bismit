use common;
use ocl::{ self, Ocl, WorkSize };
use ocl::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
use cortical_region_layer:: { Layer };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use cells:: { Aux };
use peak_column:: { PeakColumn };
use pyramidals::{ Pyramidals };

use std::ops;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ self, ThreadRng, Rng };
use num::{ self, Integer };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Columns {
	width: u32,
	axn_output_row: u8,
	kern_cycle: ocl::Kernel,
	kern_post_inhib: ocl::Kernel,
	kern_output: ocl::Kernel,
	kern_learn: ocl::Kernel,
	rng: rand::XorShiftRng,
	regrow_counter: usize,
	pub states: Envoy<ocl::cl_uchar>,
	pub states_raw: Envoy<ocl::cl_uchar>,
	pub cels_status: Envoy<ocl::cl_uchar>,
	pub peak_cols: PeakColumn,
	//pub syns: ColumnSynapses,
	pub syns: Synapses,
	
}

impl Columns {
	pub fn new(width: u32, region: &CorticalRegion, axons: &Axons, pyrs: &Pyramidals, aux: &Aux, ocl: &Ocl) -> Columns {
		let layer = region.col_input_layer().expect("columns::Columns::new()");
		let depth: u8 = layer.depth();

		let syns_per_den_l2: u32 = common::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
		//let syns_per_cel: u32 = 1 << syns_per_den_l2;

		let pyr_depth = region.depth_cell_kind(&CellKind::Pyramidals);
		//let pyr_axn_base_row = region.base_row_cell_kind(&CellKind::Pyramidals); // SHOULD BE SPECIFIC LAYER(S)  

		let states = Envoy::<ocl::cl_uchar>::new(width, depth, common::STATE_ZERO, ocl);
		let states_raw = Envoy::<ocl::cl_uchar>::new(width, depth, common::STATE_ZERO, ocl);
		let cels_status = Envoy::<ocl::cl_uchar>::new(width, depth, common::STATE_ZERO, ocl);
		let peak_cols = PeakColumn::new(width, depth, region, &states, ocl);
		let syns = Synapses::new(width, depth, syns_per_den_l2, syns_per_den_l2, DendriteKind::Proximal, 
			CellKind::SpinyStellate, region, axons, aux, ocl);

		let output_rows = region.col_output_rows();
		assert!(output_rows.len() == 1);
		let axn_output_row = output_rows[0];


		let kern_cycle = ocl.new_kernel("den_cycle", WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&syns.states)
			.arg_env(&syns.strengths)
			.arg_scl(syns_per_den_l2)
			.arg_scl(common::DENDRITE_INITIAL_THRESHOLD_PROXIMAL)
			.arg_env(&states_raw)
			.arg_env(&states)
		;

		let kern_post_inhib = ocl.new_kernel("col_post_inhib_unoptd", WorkSize::TwoDim(depth as usize, width as usize))
			.arg_env(&peak_cols.col_ids)
			.arg_env(&peak_cols.states)
			.arg_env(&peak_cols.wins)
			.arg_scl(layer.base_row_pos() as u32)
			.arg_env(&states)
			.arg_env(&axons.states)
		;

		let kern_output = ocl.new_kernel("col_output", WorkSize::TwoDim(depth as usize, width as usize))
			//.lws(WorkSize::TwoDim(1 as usize, common::AXONS_WORKGROUP_SIZE as usize))
			.arg_env(&states)
			.arg_env(&pyrs.depols)
			//.arg_scl(depth)
			.arg_scl(pyr_depth)
			//.arg_scl(pyr_axn_base_row)
			.arg_scl(axn_output_row)
			.arg_env(&cels_status)
			.arg_env(&axons.states)
		;


		//println!("\n*** W: {}", peak_cols.width());


		let kern_learn = ocl.new_kernel("col_learn", WorkSize::TwoDim(depth as usize, peak_cols.width() as usize))
			.arg_env(&peak_cols.col_ids)
			.arg_env(&peak_cols.states)
			.arg_env(&syns.states)
			.arg_scl(syns_per_den_l2)
			.arg_scl(0u32)
			//.arg_env(&aux.ints_0)
			.arg_env(&syns.strengths)
			//.arg_env(&axons.states)
		;

		//println!("\n***Test");

		Columns {
			width: width,
			axn_output_row: axn_output_row,
			kern_cycle: kern_cycle,
			kern_post_inhib: kern_post_inhib,
			kern_output: kern_output,
			kern_learn: kern_learn,
			rng: rand::weak_rng(),
			regrow_counter: 0usize,
			states_raw: states_raw,
			states: states,
			cels_status: cels_status,
			peak_cols: peak_cols,
			syns: syns,
		}
	}

	pub fn cycle(&mut self, learn: bool) {
		self.syns.cycle();
		self.kern_cycle.enqueue();
		self.peak_cols.cycle(); // *****
		self.kern_post_inhib.enqueue(); // *****
		if learn { self.learn(); }
	}

	pub fn output(&self) {
		self.kern_output.enqueue();
	}

	pub fn learn(&mut self) {
		//print!("[R:{}]", self.rng.gen::<i32>());
		self.kern_learn.set_kernel_arg(4, self.rng.gen::<u32>());
		self.kern_learn.enqueue();

		self.regrow_counter += 1;

		if self.regrow_counter >= 1000 {
				self.syns.regrow();
			self.regrow_counter = 0;
		}

	}

	pub fn confab(&mut self) {
		self.states.read();
		self.states_raw.read();
		self.cels_status.read();
		//self.peak_cols.confab();
		self.syns.confab();
	} 

	pub fn axn_output_range(&self) -> (usize, usize) {
		let start = (self.axn_output_row as usize * self.width as usize) + common::SYNAPSE_REACH as usize;
		(start, start + (self.width - 1) as usize)
	}
}


/*pub struct ColumnSynapses {
	width: u32,
	depth: u8,
	per_cell: u32,
	src_row_ids_list: Vec<u8>,
	kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_char>,
	pub src_ofs: Envoy<ocl::cl_char>,
	pub src_row_ids: Envoy<ocl::cl_uchar>,
}

impl ColumnSynapses {
	pub fn new(width: u32, depth: u8, per_cell: u32, layer: &Layer, 
					region: &CorticalRegion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> ColumnSynapses {

		let syns_per_row = width * per_cell;
		let src_row_ids_list: Vec<u8> = region.src_row_ids(layer.name, DendriteKind::Proximal);
		let src_rows_len = src_row_ids_list.len() as u8;
		//let depth = src_rows_len;
		let wg_size = common::SYNAPSES_WORKGROUP_SIZE;
		//let dens_per_wg: u32 = wg_size / (common::SYNAPSES_PER_DENDRITE_PROXIMAL);
		let syns_per_den_l2: u32 = common::SYNAPSES_PER_CELL_PROXIMAL_LOG2;
		//let dens_per_wg: u32 = 1;

		print!("\nNew Proximal Synapses with: depth: {}, syns_per_row: {}, src_rows_len: {}", depth, syns_per_row, src_rows_len);

		let states = Envoy::<ocl::cl_uchar>::new(syns_per_row, depth, common::STATE_ZERO, ocl);
		let strengths = Envoy::<ocl::cl_char>::new(syns_per_row, depth, 1i8, ocl);
		let src_ofs = Envoy::<ocl::cl_char>::shuffled(syns_per_row, depth, -128, 127, ocl);
		let src_row_ids= Envoy::<ocl::cl_uchar>::new(syns_per_row, depth, 0u8, ocl);

		let mut kern_cycle = ocl.new_kernel("syns_cycle", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.lws(WorkSize::TwoDim(1 as usize, wg_size as usize));
		kern_cycle.new_arg_envoy(&axons.states);
		kern_cycle.new_arg_envoy(&src_ofs);
		kern_cycle.new_arg_envoy(&src_row_ids);
		kern_cycle.new_arg_scalar(syns_per_den_l2);
		//kern_cycle.new_arg_envoy(&aux.ints_0);
		//kern_cycle.new_arg_envoy(&aux.ints_1);
		kern_cycle.new_arg_envoy(&states);
		
		//println!("src_row_ids_list[0]: {}", src_row_ids_list[0]);
		
		let mut syns = ColumnSynapses {
			width: width,
			depth: depth,
			per_cell: per_cell,
			src_row_ids_list: src_row_ids_list,
			states: states,
			strengths: strengths,
			src_ofs: src_ofs,
			src_row_ids: src_row_ids,
			kern_cycle: kern_cycle,
		};

		syns.init(region);

		syns
	}

	fn init(&mut self, region: &CorticalRegion) {
		let len = self.width * self.per_cell * self.depth as u32;
		let mut rng = rand::weak_rng();
		let ei_start = 0usize;
		let ei_end = ei_start + len as usize;
		let src_row_idx_range: Range<usize> = Range::new(0, self.src_row_ids_list.len());
		//println!("\nInitializing Column Synapses: ei_start: {}, ei_end: {}, self.src_row_ids: {:?}, self.src_row_ids.len(): {}", ei_start, ei_end, self.src_row_ids_list, self.src_row_ids_list.len());

		for ref i in ei_start..ei_end {
			self.src_row_ids[i] = self.src_row_ids_list[src_row_idx_range.ind_sample(&mut rng)];
		}
		self.src_row_ids.write();
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue();
	}
}*/
