use ocl;
//use std;
//use std::io;
use rand;
use rand::distributions::{IndependentSample, Range};
use std::ptr;
use std::default::Default;

pub const KERNELS_FILE_NAME: &'static str = "bismit.cl";

pub const COLUMNS_PER_HYPERCOLUMN: uint = 64u;
pub const COLUMNS_PER_ADDRESS_BLOCK: uint = 16u;
pub const CELLS_PER_COLUMN: uint = 16u;
pub const AXONS_PER_CELL: uint = 256u;
pub const DENDRITES_PER_CELL: uint = 16u;
pub const SYNAPSES_PER_DENDRITE: uint = 16u;

pub const HYPERCOLUMNS_TOTAL: uint = 16u;

pub const SYNAPSE_WEIGHT_ZERO: u8 = 16;
pub const SYNAPSE_WEIGHT_INITIAL_DEVIATION: u8 = 3;
pub const DENDRITE_INITIAL_THRESHOLD_PROXIMAL: u16 = 128;

pub const COLUMNS_TOTAL: uint = COLUMNS_PER_HYPERCOLUMN * HYPERCOLUMNS_TOTAL;
pub const CELLS_TOTAL: uint = CELLS_PER_COLUMN * COLUMNS_TOTAL;
pub const AXONS_TOTAL: uint = AXONS_PER_CELL * CELLS_TOTAL;
pub const DENDRITES_TOTAL: uint = DENDRITES_PER_CELL * CELLS_TOTAL;
pub const SYNAPSES_TOTAL: uint = SYNAPSES_PER_DENDRITE * DENDRITES_TOTAL; 


pub struct Cortex {
	pub ocl: ocl::Ocl,

	pub synapses: Vec<ocl::cl_uchar>,
	pub dendrite_thresholds: Vec<ocl::cl_ushort>,
	pub dendrite_states: Vec<ocl::cl_ushort>,
	pub axons: Vec<ocl::cl_ushort>,
	pub column_states: Vec<ocl::cl_uint>,
	pub hypercolumn_states: Vec<ocl::cl_uint>,

	pub pseudo_columns: [f32, ..COLUMNS_TOTAL],

	pub synapses_buff: ocl::cl_mem,
	pub dendrite_thresholds_buff: ocl::cl_mem,
	pub dendrite_states_buff: ocl::cl_mem,
	pub axons_buff: ocl::cl_mem,
	pub column_states_buff: ocl::cl_mem,
	pub hypercolumn_states_buff: ocl::cl_mem,

}

impl Cortex {
	pub fn new() -> Cortex {
		println!("Initializing Cortex...");

		Cortex {
			ocl: ocl::Ocl::new(),

			synapses : Vec::with_capacity(self::SYNAPSES_TOTAL),
			dendrite_thresholds : Vec::with_capacity(self::DENDRITES_TOTAL),
			dendrite_states : Vec::with_capacity(self::DENDRITES_TOTAL),
			axons : Vec::with_capacity(self::AXONS_TOTAL),
			column_states : Vec::with_capacity(self::COLUMNS_TOTAL),
			hypercolumn_states : Vec::with_capacity(self::HYPERCOLUMNS_TOTAL),

			pseudo_columns: [0.0f32, ..COLUMNS_TOTAL],

			synapses_buff: ptr::null_mut(),
			dendrite_thresholds_buff: ptr::null_mut(),
			dendrite_states_buff: ptr::null_mut(),
			axons_buff: ptr::null_mut(),
			column_states_buff: ptr::null_mut(),
			hypercolumn_states_buff: ptr::null_mut(),
		}

	}

	pub fn init(&mut self) {
		self.init_data_vectors();
		self.init_pseudo_columns();
		self.init_buffers();
		self.write_buffers();
	}

	pub fn init_data_vectors(&mut self) {
		println!("Initializing Cortical Data Vectors...");

		//SYNAPSES
		let rng_range = Range::new(SYNAPSE_WEIGHT_ZERO - (SYNAPSE_WEIGHT_INITIAL_DEVIATION), SYNAPSE_WEIGHT_ZERO + (SYNAPSE_WEIGHT_INITIAL_DEVIATION) + 1);
		let mut rng = rand::task_rng();
		for i in range(0u, self.synapses.capacity()) {
			self.synapses.push(rng_range.ind_sample(&mut rng));
		}

		//DENDRITES
		for i in range(0u, self.dendrite_thresholds.capacity()) {
			self.dendrite_thresholds.push(DENDRITE_INITIAL_THRESHOLD_PROXIMAL);
		}

		for i in range(0u, self.dendrite_states.capacity()) {
			self.dendrite_states.push(0u16);
		}

		let rng_range = Range::new(0u16, 0xFFFEu16);
		let mut rng = rand::task_rng();
		for i in range(0u, self.axons.capacity()) {
			self.axons.push(rng_range.ind_sample(&mut rng));
		}

		for i in range(0u, self.column_states.capacity()) {
			self.column_states.push(0u32);
		}

		for i in range(0u, self.hypercolumn_states.capacity()) {
			self.hypercolumn_states.push(0u32);
		}

		/*
		= Vec::with_capacity(self::DENDRITES_TOTAL);
		= Vec::with_capacity(self::AXONS_TOTAL);
		= Vec::with_capacity(self::COLUMNS_TOTAL);
		= Vec::with_capacity(self::HYPERCOLUMNS_TOTAL) 
		*/

	}

	pub fn init_pseudo_columns(&mut self) {
		println!("Initializing Cortical Pseudo Columns...");
		let rng_range = Range::new(1f32, 500f32);
		let mut rng = rand::task_rng();

		for i in range(0u, COLUMNS_TOTAL) {
			self.pseudo_columns[i] = rng_range.ind_sample(&mut rng);
		}

	}

	pub fn init_buffers(&mut self) {
		println!("Initializing Cortical Buffers...");
		self.synapses_buff = ocl::new_write_buffer(&self.synapses, self.ocl.context);
		self.dendrite_thresholds_buff = ocl::new_write_buffer(&self.dendrite_thresholds, self.ocl.context);
		self.dendrite_states_buff = ocl::new_write_buffer(&self.dendrite_states, self.ocl.context);
		self.axons_buff = ocl::new_write_buffer(&self.axons, self.ocl.context);
		self.column_states_buff = ocl::new_write_buffer(&self.column_states, self.ocl.context);
		self.hypercolumn_states_buff = ocl::new_write_buffer(&self.hypercolumn_states, self.ocl.context);

	}

	pub fn write_buffers(&mut self) {
		println!("Writing Buffers...");
		ocl::enqueue_write_buffer(&self.synapses, self.synapses_buff, self.ocl.command_queue);
		ocl::enqueue_write_buffer(&self.dendrite_thresholds, self.dendrite_thresholds_buff, self.ocl.command_queue);
		ocl::enqueue_write_buffer(&self.dendrite_states, self.dendrite_states_buff, self.ocl.command_queue);
		ocl::enqueue_write_buffer(&self.axons, self.axons_buff, self.ocl.command_queue);
		ocl::enqueue_write_buffer(&self.column_states, self.column_states_buff, self.ocl.command_queue);
		ocl::enqueue_write_buffer(&self.hypercolumn_states, self.hypercolumn_states_buff, self.ocl.command_queue);
	}

	pub fn release_components(&mut self) {
		println!("Releasing OCL Components...")
		
		ocl::release_mem_object(self.synapses_buff);
		ocl::release_mem_object(self.dendrite_thresholds_buff);
		ocl::release_mem_object(self.dendrite_states_buff);
		ocl::release_mem_object(self.axons_buff);
		ocl::release_mem_object(self.column_states_buff);
		ocl::release_mem_object(self.hypercolumn_states_buff);

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

