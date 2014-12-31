use ocl;
use common;
use cortical_component::{ CorticalComponent };
use neurons_scalar;
use neurons_binary;
//use std;
//use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;
use std::default::Default;

pub struct Columns {
	pub states: CorticalComponent<ocl::cl_uint>,
	pub axons: neurons_scalar::Axons,
	pub dendrites: neurons_scalar::Dendrites,
	pub synapses: neurons_scalar::Synapses,
}
impl Columns {
	pub fn new(hcols: uint, ocl: &ocl::Ocl) -> Columns {
		Columns {
			states: CorticalComponent::<ocl::cl_uint>::new(common::COLUMNS_PER_SEGMENT, 0u32, ocl),
			axons:	neurons_scalar::Axons::new(common::COLUMN_AXONS_PER_SEGMENT, ocl),
			dendrites: neurons_scalar::Dendrites::new(common::COLUMN_DENDRITES_PER_SEGMENT, ocl),
			synapses: neurons_scalar::Synapses::new(common::COLUMN_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}


pub struct Cells {
	pub states: CorticalComponent<ocl::cl_uint>,
	pub axons: neurons_binary::Axons,
	pub dendrites: neurons_binary::Dendrites,
	pub synapses: neurons_binary::Synapses,
}
impl Cells {
	pub fn new(hcols: uint, ocl: &ocl::Ocl) -> Cells {
		Cells {
			states: CorticalComponent::<ocl::cl_uint>::new(common::CELLS_PER_SEGMENT, 0u32, ocl),
			axons:	neurons_binary::Axons::new(common::CELL_AXONS_PER_SEGMENT, ocl),
			dendrites: neurons_binary::Dendrites::new(common::CELL_DENDRITES_PER_SEGMENT, ocl),
			synapses: neurons_binary::Synapses::new(common::CELL_SYNAPSES_PER_SEGMENT, ocl),
		}
	}
}

pub struct HyperColumns {
	pub qty: uint,
	pub states: CorticalComponent<ocl::cl_uint>,
}
impl HyperColumns {
	pub fn new(qty: uint, ocl: &ocl::Ocl) -> HyperColumns {
		HyperColumns {
			qty: common::HYPERCOLUMNS_PER_SEGMENT,
			states: CorticalComponent::<ocl::cl_uint>::new(common::HYPERCOLUMNS_PER_SEGMENT, 0u32, ocl),
		}
	}
}


pub struct CortexSegment {
	pub hypercolumns: HyperColumns,
	pub columns: Columns,
	pub cells: Cells,
}
impl CortexSegment {
	pub fn new(hcols: uint, ocl: &ocl::Ocl) -> CortexSegment {

		CortexSegment {

			hypercolumns: HyperColumns::new(hcols, ocl),
			columns: Columns::new(hcols, ocl),
			cells: Cells::new(hcols, ocl),

			
			/*
			hypcol_states: CorticalComponent::<ocl::cl_uint>::new(Vec::from_elem(common::HYPERCOLUMNS_PER_SEGMENT, 0u32), ocl),

			col_states: CorticalComponent::<ocl::cl_uint>::new(Vec::from_elem(common::COLUMNS_PER_SEGMENT, 0u32)), ocl),
			col_tar_cols: CorticalComponent::<ocl::cl_ushort>::new(Vec::with_capacity(common::COLUMN_TARGETS_PER_SEGMENT), ocl),
			col_tar_syns: CorticalComponent::<ocl::cl_uchar>::new(Vec::with_capacity(common::COLUMN_TARGETS_PER_SEGMENT), ocl),
			col_den_values: CorticalComponent::<ocl::cl_uchar>::new(Vec::from_elem(common::COLUMN_DENDRITES_PER_SEGMENT, 0u8)), ocl),
			col_den_thresholds: CorticalComponent::<ocl::cl_uchar>::new(Vec::from_elem(common::COLUMN_DENDRITES_PER_SEGMENT, 256u16)), ocl),
			col_syn_values: CorticalComponent::<ocl::cl_uchar>::new(Vec::from_elem(common::COLUMN_SYNAPSES_PER_SEGMENT, 0u8)), ocl),
			col_syn_strengths: CorticalComponent::<ocl::cl_uchar>::new(Vec::from_elem(common::COLUMN_SYNAPSES_PER_SEGMENT, 16u8)), ocl),

			cel_states: CorticalComponent::<ocl::cl_uint>::new(Vec::from_elem(common::CELLS_PER_SEGMENT, 0u32)), ocl),
			cel_tar_cels: CorticalComponent::<ocl::cl_ushort>::new(Vec::with_capacity(common::CELL_TARGETS_PER_SEGMENT), ocl),
			cel_tar_syns: CorticalComponent::<ocl::cl_uchar>::new(Vec::with_capacity(common::CELL_TARGETS_PER_SEGMENT), ocl),
			cel_denstates: CorticalComponent::<ocl::cl_ushort>::new(Vec::from_elem(common::CELL_DENDRITES_PER_SEGMENT, 0u16)), ocl),
			cel_den_thresholds: CorticalComponent::<ocl::cl_uchar>::new(Vec::from_elem(common::CELL_DENDRITES_PER_SEGMENT, 16u8)), ocl),
			cel_den_synstates: CorticalComponent::<ocl::cl_ushort>::new(Vec::from_elem(common::CELL_DENDRITES_PER_SEGMENT, 0u16)), ocl),
			cel_syn_strengths: CorticalComponent::<ocl::cl_uchar>::new(Vec::from_elem(common::CELL_SYNAPSES_PER_SEGMENT, 16u8)), ocl),
			*/

			/*
			synapse_values: CorticalComponent::<ocl::cl_uchar>::new(Vec::with_capacity(common::SYNAPSES_PER_SEGMENT), ocl),
			synapse_weights : CorticalComponent::<ocl::cl_uchar>::new(Vec::with_capacity(common::SYNAPSES_PER_SEGMENT), ocl),
			dendrite_thresholds : CorticalComponent::<ocl::cl_ushort>::new(Vec::with_capacity(common::DENDRITES_PER_SEGMENT), ocl),
			axon_targets : CorticalComponent::<ocl::cl_ushort>::new(Vec::with_capacity(common::TARGETS_PER_SEGMENT), ocl),

			column_states : CorticalComponent::<ocl::cl_uint>::new(Vec::from_elem(common::COLUMNS_PER_SEGMENT, 0u32), ocl),
			hypercolumn_states : CorticalComponent::<ocl::cl_uint>::new(Vec::from_elem(common::HYPERCOLUMNS_PER_SEGMENTMENT, 0u32), ocl),
			dendrite_values : CorticalComponent::<ocl::cl_ushort>::new(Vec::from_elem(common::DENDRITES_PER_SEGMENT, 0u16), ocl),

			column_axon_col_targets: CorticalComponent::<ocl::cl_ushort>::new(Vec::with_capacity(common::COLUMNS_PER_SEGMENT), ocl),
			column_axon_syn_targets: CorticalComponent::<ocl::cl_uchar>::new(Vec::with_capacity(common::COLUMNS_PER_SEGMENT), ocl),
			*/

		}
	}

	pub fn init(&mut self, ocl: &ocl::Ocl) {

		/*

		let rng_range = Range::new(common::SYNAPSE_WEIGHT_ZERO - (common::SYNAPSE_WEIGHT_INITIAL_DEVIATION), common::SYNAPSE_WEIGHT_ZERO + (common::SYNAPSE_WEIGHT_INITIAL_DEVIATION) + 1);
		let mut rng = rand::task_rng();
		for i in range(0u, self.synapse_values.vec.capacity()) {
			self.synapse_values.vec.push(rng_range.ind_sample(&mut rng));
		}
		self.synapse_values.init();

		
		for i in range(0u, self.synapse_weights.vec.capacity()) {
			self.synapse_weights.vec.push(rng_range.ind_sample(&mut rng));
		}
		self.synapse_weights.init();

		
		for i in range(0u, self.dendrite_thresholds.vec.capacity()) {
			self.dendrite_thresholds.vec.push(common::DENDRITE_INITIAL_THRESHOLD);
		}
		self.dendrite_thresholds.init();


		self.dendrite_values.init();


		let rng_range = Range::new(0u16, 0xFFFEu16);
		let mut rng = rand::task_rng();
		for i in range(0u, self.axon_targets.vec.capacity()) {
			self.axon_targets.vec.push(rng_range.ind_sample(&mut rng));
		}
		self.axon_targets.init();


		self.column_states.init();


		self.hypercolumn_states.init();
		*/

	}
}

pub struct Cortex {
	pub ocl: ocl::Ocl,
	pub cortex_segments: Vec<CortexSegment>,
	pub sensory_segments: Vec<(CorticalComponent<ocl::cl_ushort>, CorticalComponent<ocl::cl_uchar>)>,
	// ADD ME:  pub motor_segments: Vec<MotorSegment>,

	
}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex...");

		let ocl = ocl::Ocl::new();

		let mut cs = Vec::with_capacity(common::CORTICAL_SEGMENTS_TOTAL);
		for i in range(0u, common::CORTICAL_SEGMENTS_TOTAL) {
			let mut seg = CortexSegment::new(common::HYPERCOLUMNS_PER_SEGMENT, &ocl);
			seg.init(&ocl);
			cs.push(seg);
		}

		let mut ss = Vec::with_capacity(common::SENSORY_SEGMENTS_TOTAL);
		for i in range(0u, common::SENSORY_SEGMENTS_TOTAL) {
			let mut col = CorticalComponent::<ocl::cl_ushort>::new(common::SENSORY_CHORD_WIDTH, 0u16, &ocl);
			let mut syn = CorticalComponent::<ocl::cl_uchar>::new(common::SENSORY_CHORD_WIDTH, 0u8, &ocl);
			col.init();
			syn.init();
			ss.push((col, syn));
		}

		Cortex {
			sensory_segments: ss,
			cortex_segments: cs,
			ocl: ocl,
		}
	}

	pub fn init(&mut self) {

	}



	pub fn release_components(&mut self) {
		println!("Releasing OCL Components...")

		self.ocl.release_components();
		
	}

	pub fn readback_test<T: ToPrimitive + Clone + Default>(
				&self,
				test_source: &Vec<T>,
				test_source_buff: ocl::cl_mem, 
				test_kernel_name: &str, 

	) {
		println!("Performing Readback Test ({})...", test_kernel_name);

		// Create Vec for output
		let mut test_out: Vec<T> = Vec::with_capacity(test_source.len());
		for i in range(0u, test_out.capacity()) {
			test_out.push(Default::default());
		}

		// Create output buffer to read into output Vec
		let test_out_buff: ocl::cl_mem = ocl::new_read_buffer(&test_out, self.ocl.context);

		// Test Kernels simply clone the vector right now
		let test_out_kernel: ocl::cl_kernel = ocl::new_kernel(self.ocl.program, test_kernel_name);
		ocl::set_kernel_arg(0, test_source_buff, test_out_kernel);
		ocl::set_kernel_arg(1, test_out_buff, test_out_kernel);

		// Run Kernel then read from output buffer
		ocl::enqueue_kernel(test_out_kernel, self.ocl.command_queue, test_source.len());
		ocl::enqueue_read_buffer(&test_out, test_out_buff, self.ocl.command_queue);

		// Calculate sum total for output vector
		print_vec_info(test_source, test_kernel_name);
		print_vec_info(&test_out, test_kernel_name);
		
		// Run again 5 times using output as source
		ocl::set_kernel_arg(0, test_out_buff, test_out_kernel);

		for i in range(0u, 5) {
			//println!("... {}", i + 1);
			ocl::enqueue_kernel(test_out_kernel, self.ocl.command_queue, test_source.len());
			ocl::enqueue_read_buffer(&test_out, test_out_buff, self.ocl.command_queue);
			print_vec_info(&test_out, test_kernel_name);
		}

		//print_vec_info(test_source, test_kernel_name);
		//print_vec_info(&test_out, test_kernel_name);
		
		ocl::release_kernel(test_out_kernel);
	}

}


pub fn print_vec_info<T: ToPrimitive + Clone + Default>(my_vec: &Vec<T>, info: &str) {
	let mut total = 0u;
	for x in range(0u, my_vec.len()) {
		total += my_vec[x].to_uint().unwrap();
	}
	println!("*** {} *** Total: {}; Len: {}; Avg: {}", info, total, my_vec.len(), total/my_vec.len());

}

