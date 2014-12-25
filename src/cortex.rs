use ocl;
//use std;
//use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};
use std::ptr;
use std::default::Default;

pub const KERNELS_FILE_NAME: &'static str = "bismit.cl";

pub const CORTICAL_SEGMENTS_TOTAL: uint = 1;
pub const HYPERCOLUMNS_PER_SEGMENT: uint = 16;
pub const SENSORY_CHORD_WIDTH: uint = 2048;
pub const SENSORY_SEGMENTS_TOTAL: uint = 2;

pub const SYNAPSE_WEIGHT_ZERO: u8 = 16;
pub const SYNAPSE_WEIGHT_INITIAL_DEVIATION: u8 = 3;
pub const DENDRITE_INITIAL_THRESHOLD: u16 = 128;

pub const COLUMNS_PER_HYPERCOLUMN: uint = 64u;
pub const COLUMNS_PER_ADDRESS_BLOCK: uint = 16u;
pub const CELLS_PER_COLUMN: uint = 16u;
pub const DENDRITES_PER_CELL: uint = 16u;
pub const SYNAPSES_PER_DENDRITE: uint = 16u;
pub const AXONS_PER_CELL: uint = (DENDRITES_PER_CELL * SYNAPSES_PER_DENDRITE);

pub const COLUMNS_TOTAL: uint = COLUMNS_PER_HYPERCOLUMN * HYPERCOLUMNS_PER_SEGMENT;
pub const CELLS_TOTAL: uint = CELLS_PER_COLUMN * COLUMNS_TOTAL;
pub const AXONS_TOTAL: uint = AXONS_PER_CELL * CELLS_TOTAL;
pub const DENDRITES_TOTAL: uint = DENDRITES_PER_CELL * CELLS_TOTAL;
pub const SYNAPSES_TOTAL: uint = SYNAPSES_PER_DENDRITE * DENDRITES_TOTAL; 

pub struct CorticalComponent<T> {
	pub vec: Vec<T>,
	pub buff: ocl::cl_mem,
	pub context: ocl::cl_context,
	pub command_queue: ocl::cl_command_queue,
}
impl <T> CorticalComponent<T> {
	pub fn new<T>(v: Vec<T>, ocl: &ocl::Ocl) -> CorticalComponent<T> {
		CorticalComponent {
			vec: v,
			buff: ptr::null_mut(),
			context: ocl.context,
			command_queue: ocl.command_queue,
		}
	}

	pub fn init(&mut self) {
		self.buff = ocl::new_write_buffer(&self.vec, self.context);
		ocl::enqueue_write_buffer(&self.vec, self.buff, self.command_queue);
	}

	pub fn release(&mut self) {
		ocl::release_mem_object(self.buff);
	}
}

pub struct CortexSegment {
	size_hcols: uint,

	pub synapse_values: CorticalComponent<ocl::cl_uchar>,
	pub synapse_weights: CorticalComponent<ocl::cl_uchar>,
	pub dendrite_thresholds: CorticalComponent<ocl::cl_ushort>,
	pub dendrite_values: CorticalComponent<ocl::cl_ushort>,
	pub axon_targets: CorticalComponent<ocl::cl_ushort>,
	pub column_states: CorticalComponent<ocl::cl_uint>,
	pub column_axon_col_targets: CorticalComponent<ocl::cl_ushort>,
	pub column_axon_syn_targets: CorticalComponent<ocl::cl_uchar>,
	pub hypercolumn_states: CorticalComponent<ocl::cl_uint>,
}
impl CortexSegment {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> CortexSegment {

		CortexSegment {
			size_hcols: size,

			synapse_values: CorticalComponent::<ocl::cl_uchar>::new(Vec::with_capacity(self::SYNAPSES_TOTAL), ocl),
			synapse_weights : CorticalComponent::<ocl::cl_uchar>::new(Vec::with_capacity(self::SYNAPSES_TOTAL), ocl),
			dendrite_thresholds : CorticalComponent::<ocl::cl_ushort>::new(Vec::with_capacity(self::DENDRITES_TOTAL), ocl),
			axon_targets : CorticalComponent::<ocl::cl_ushort>::new(Vec::with_capacity(self::AXONS_TOTAL), ocl),

			column_states : CorticalComponent::<ocl::cl_uint>::new(Vec::from_elem(self::COLUMNS_TOTAL, 0u32), ocl),
			hypercolumn_states : CorticalComponent::<ocl::cl_uint>::new(Vec::from_elem(self::HYPERCOLUMNS_PER_SEGMENT, 0u32), ocl),
			dendrite_values : CorticalComponent::<ocl::cl_ushort>::new(Vec::from_elem(self::DENDRITES_TOTAL, 0u16), ocl),

			column_axon_col_targets: CorticalComponent::<ocl::cl_ushort>::new(Vec::with_capacity(self::COLUMNS_TOTAL), ocl),
			column_axon_syn_targets: CorticalComponent::<ocl::cl_uchar>::new(Vec::with_capacity(self::COLUMNS_TOTAL), ocl),

		}
	}

	pub fn init(&mut self, ocl: &ocl::Ocl) {

		//SYNAPSES
		let rng_range = Range::new(SYNAPSE_WEIGHT_ZERO - (SYNAPSE_WEIGHT_INITIAL_DEVIATION), SYNAPSE_WEIGHT_ZERO + (SYNAPSE_WEIGHT_INITIAL_DEVIATION) + 1);
		let mut rng = rand::task_rng();
		for i in range(0u, self.synapse_values.vec.capacity()) {
			self.synapse_values.vec.push(rng_range.ind_sample(&mut rng));
		}
		self.synapse_values.init();

		
		for i in range(0u, self.synapse_weights.vec.capacity()) {
			self.synapse_weights.vec.push(rng_range.ind_sample(&mut rng));
		}
		self.synapse_weights.init();

		//DENDRITES
		
		for i in range(0u, self.dendrite_thresholds.vec.capacity()) {
			self.dendrite_thresholds.vec.push(DENDRITE_INITIAL_THRESHOLD);
		}
		self.dendrite_thresholds.init();

		/*
		for i in range(0u, self.dendrite_values.vec.capacity()) {
			self.dendrite_values.vec.push(0u16);
		}
		*/
		self.dendrite_values.init();

		let rng_range = Range::new(0u16, 0xFFFEu16);
		let mut rng = rand::task_rng();
		for i in range(0u, self.axon_targets.vec.capacity()) {
			self.axon_targets.vec.push(rng_range.ind_sample(&mut rng));
		}
		self.axon_targets.init();

		/*
		for i in range(0u, self.column_states.vec.capacity()) {
			self.column_states.vec.push(0u32);
		}
		*/
		self.column_states.init();

		/*
		for i in range(0u, self.hypercolumn_states.vec.capacity()) {
			self.hypercolumn_states.vec.push(0u32);
		}
		*/
		self.hypercolumn_states.init();

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

		let mut cs = Vec::with_capacity(CORTICAL_SEGMENTS_TOTAL);
		for i in range(0u, CORTICAL_SEGMENTS_TOTAL) {
			let mut seg = CortexSegment::new(HYPERCOLUMNS_PER_SEGMENT, &ocl);
			seg.init(&ocl);
			cs.push(seg);
		}

		let mut ss = Vec::with_capacity(SENSORY_SEGMENTS_TOTAL);
		for i in range(0u, SENSORY_SEGMENTS_TOTAL) {
			let mut col = CorticalComponent::<ocl::cl_ushort>::new(Vec::from_elem(SENSORY_CHORD_WIDTH, 0u16), &ocl);
			let mut syn = CorticalComponent::<ocl::cl_uchar>::new(Vec::from_elem(SENSORY_CHORD_WIDTH, 0u8), &ocl);
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

