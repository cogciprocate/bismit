use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };


use std::num;
use std::ops;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };


pub struct Columns {
	width: u32,
	kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_char>,
	pub syns: ColumnSynapses,
}

impl Columns {
	pub fn new(width: u32, region: &CorticalRegion, axons: &Axons, ocl: &Ocl) -> Columns {
		let height: u8 = 1;
		let syns_per_cell = common::DENDRITES_PER_CELL_PROXIMAL * common::SYNAPSES_PER_DENDRITE_PROXIMAL;

		let states = Envoy::<ocl::cl_char>::new(width, height, 0i8, ocl);
		let syns = ColumnSynapses::new(width, syns_per_cell, region, axons, ocl);

		let kern_cycle = ocl.new_kernel("col_cycle", WorkSize::TwoDim(height as usize, width as usize))
			.arg(&syns.states)
			.arg(&states)
		;

		Columns {
			width: width,
			kern_cycle: kern_cycle,
			states: states,
			syns: syns,
		}
	}

	pub fn cycle(&self, axons: &Axons, ocl: &Ocl) {
		self.kern_cycle.enqueue();
		self.syns.cycle(axons, ocl);
	}
}


pub struct ColumnSynapses {
	width: u32,
	height: u8,
	per_cell: u32,
	src_row_ids_list: Vec<u8>,
	kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_char>,
	pub strengths: Envoy<ocl::cl_char>,
	pub src_ofs: Envoy<ocl::cl_char>,
	pub src_row_ids: Envoy<ocl::cl_uchar>,
}

impl ColumnSynapses {
	pub fn new(width: u32, per_cell: u32, region: &CorticalRegion, axons: &Axons, ocl: &Ocl) -> ColumnSynapses {
		let syns_per_row = width * per_cell;
		let src_row_ids_list: Vec<u8> = region.src_row_ids(region.col_input_row(), DendriteKind::Proximal);
		let src_rows_len = src_row_ids_list.len() as u8;

		let height = src_rows_len;

		//println!("New Column Synapses with: height: {}, syns_per_row: {},", height, syns_per_row);

		let states = Envoy::<ocl::cl_char>::new(syns_per_row, height, 0i8, ocl);
		let strengths = Envoy::<ocl::cl_char>::new(syns_per_row, height, 1i8, ocl);
		let src_ofs = Envoy::<ocl::cl_char>::shuffled(syns_per_row, height, -128, 127, ocl);
		let src_row_ids= Envoy::<ocl::cl_uchar>::new(syns_per_row, height, 0u8, ocl);

		let kern_cycle = ocl.new_kernel("col_syns_cycle", 
			WorkSize::TwoDim(height as usize, width as usize))
			.arg(&axons.states)
			.arg(&src_ofs)
			.arg(&src_row_ids)
			.arg(&states)
			//.arg_scalar(src_row_ids_list[0])	// FIX THIS TO BE AN ENVOY
			//.arg_local(0u8, common::SYNAPSE_WORKGROUP_SIZE + per_cell as usize)
		;
		//println!("src_row_ids_list[0]: {}", src_row_ids_list[0]);
		
		let mut syns = ColumnSynapses {
			width: width,
			height: height,
			per_cell: per_cell,
			src_row_ids_list: src_row_ids_list,
			states: states,
			strengths: strengths,
			src_ofs: src_ofs,
			src_row_ids: src_row_ids,
			kern_cycle: kern_cycle,
		};

		//common::print_vec(&syns.src_ofs.vec, 1 << 12, true, Some(ops::Range{ start: -128, end: 127 }));
		//syns.src_ofs.print_val_range(1 << 12, -128, 127);
		syns.init(region);

		syns
	}

	fn init(&mut self, region: &CorticalRegion) {
		let row_len = self.width * self.per_cell;
		let mut rng = rand::weak_rng();

		let ei_start = 0usize;
		let ei_end = ei_start + row_len as usize;
		let src_row_idx_range: Range<usize> = Range::new(0, self.src_row_ids_list.len());
		//println!("\nInitializing Column Synapses: ei_start: {}, ei_end: {}, self.src_row_ids: {:?}, self.src_row_ids.len(): {}", ei_start, ei_end, self.src_row_ids_list, self.src_row_ids_list.len());
		//let col_off_range: Range<i8> = Range::new(-126, 127);

		for i in range(ei_start, ei_end) {
			//self.strengths[i] = common::DST_SYNAPSE_STRENGTH_DEFAULT;
			self.src_row_ids[i] = self.src_row_ids_list[src_row_idx_range.ind_sample(&mut rng)];
			//self.src_row_ids[i] = 5;
			//self.axn_col_offs[i] = col_off_range.ind_sample(&mut rng);
		}

		self.src_row_ids.write();
		//self.src_row_ids.print(8);
	}

	/*fn init_kern_cycle(&mut self, 
				axons: &Axons, 
				states: &Envoy<ocl::cl_char>, 
				strengths: &Envoy<ocl::cl_char>,
				src_ofs: &Envoy<ocl::cl_char>,
				src_row_ids: &Envoy<ocl::cl_uchar>, 
				src_row_ids_list: Vec<u8>,
				ocl: &Ocl) -> ocl::Kernel {
		ocl.new_kernel(
			"col_syns_cycle", 
			WorkSize::TwoDim(self.height as usize, self.width as usize)
		)
		//.lws(WorkSize::TwoDim(1 as usize, common::SYNAPSE_WORKGROUP_SIZE as usize))
		.arg(states)
		.arg(src_ofs)
		.arg(src_row_ids)
		.arg(states)
		.arg_scalar(self.src_row_ids_list[0])	// FIX THIS TO BE AN ENVOY
		.arg_local(0u8, common::SYNAPSE_WORKGROUP_SIZE + self.per_cell as usize)
	}*/

	pub fn cycle(&self, axons: &Axons, ocl: &Ocl) {
		/*ocl.new_kernel(
			"col_syns_cycle", 
			WorkSize::TwoDim(self.height as usize, self.width as usize)
		)
		//.lws(WorkSize::TwoDim(1 as usize, common::SYNAPSE_WORKGROUP_SIZE as usize))
		.arg(&axons.states)
		.arg(&self.src_ofs)
		.arg(&self.src_row_ids)
		.arg(&self.states)
		.arg_scalar(self.src_row_ids_list[0])	// FIX THIS TO BE AN ENVOY
		.arg_local(0u8, common::SYNAPSE_WORKGROUP_SIZE + self.per_cell as usize)*/
		self.kern_cycle.enqueue();


		/*let kern = ocl::new_kernel(ocl.program, "col_syns_cycle");
		ocl::set_kernel_arg(0, axons.states.buf, kern);
		ocl::set_kernel_arg(1, self.src_ofs.buf, kern);
		ocl::set_kernel_arg(2, self.src_row_ids.buf, kern);
		ocl::set_kernel_arg(3, self.states.buf, kern);
		//ocl::set_kernel_arg(4, 1, kern);

		let gws = (self.height as usize, self.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);*/
	}
}
