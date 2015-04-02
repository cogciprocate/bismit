/*
==== Log ====

14-20-13:
	-Test persistence of data loaded to memory




Things to test:

*/

use cortex;
//use std;
//use std::io;
//use rand;
//use rand::distributions::{IndependentSample, Range};

/*

pub const COLUMNS_PER_HYPERCOLUMN: uint = 64u;
pub const COLUMNS_PER_ADDRESS_BLOCK: uint = 16u;
pub const CELLS_PER_COLUMN: uint = 16u;
pub const AXONS_PER_CELL: uint = 256u;
pub const DENDRITES_PER_CELL: uint = 16u;
pub const SYNAPSES_PER_DENDRITE: uint = 16u;

pub const HYPERCOLUMNS_TOTAL: uint = 16u;

pub const COLUMNS_TOTAL: uint = COLUMNS_PER_HYPERCOLUMN * HYPERCOLUMNS_TOTAL;
pub const CELLS_TOTAL: uint = CELLS_PER_COLUMN * COLUMNS_TOTAL;
pub const AXONS_TOTAL: uint = AXONS_PER_CELL * CELLS_TOTAL;
pub const DENDRITES_TOTAL: uint = DENDRITES_PER_CELL * CELLS_TOTAL;
pub const SYNAPSES_TOTAL: uint = SYNAPSES_PER_DENDRITE * DENDRITES_TOTAL; 

pub const SYNAPSE_WEIGHT_ZERO: u8 = 16;
pub const SYNAPSE_WEIGHT_INITIAL_DEVIATION: u8 = 3;
pub const DENDRITE_INITIAL_THRESHOLD: u16 = 128;

*/

//pub const KERNELS_FILE_NAME: &'static str = "bismit.cl";

pub fn run() {

	//println!("=== Test 2 (SYNAPSES_TOTAL: {}) ===", SYNAPSES_TOTAL);

/*
	//Read Kernel File
	let file_path: std::path::Path = std::path::Path::new(format!("{}/{}/{}", env!("P"), "bismit/src", KERNELS_FILE_NAME));
	let kern_str = io::File::open(&file_path).read_to_end().unwrap();
	let kern_c_str = std::str::from_utf8(kern_str.as_slice()).unwrap().to_c_str();
	let x: int;

	
	
	// Create Parts and Pieces
	let platform = ocl::new_platform() as ocl::cl_platform_id;
	let device: ocl::cl_device_id = ocl::new_device(platform);
	let context: ocl::cl_context = ocl::new_context(device);
	let program: ocl::cl_program = ocl::new_program(kern_c_str.as_ptr(), context, device);
	let command_queue: ocl::cl_command_queue = ocl::new_command_queue(context, device);
*/

	//let my_ocl = ocl::Ocl::new(); 

	let mut my_cortex = cortex::Cortex::new();

	

/*
	// Initialize Data
	let mut my_cortex.my_cortex.synapses: Vec<ocl::cl_uchar> = Vec::with_capacity(self::SYNAPSES_TOTAL);
	let mut my_cortex.dendrite_thresholds: Vec<ocl::cl_ushort> = Vec::with_capacity(self::DENDRITES_TOTAL);
	let mut dendrite_states: Vec<ocl::cl_ushort> = Vec::with_capacity(self::DENDRITES_TOTAL);
	let mut axons: Vec<ocl::cl_ushort> = Vec::with_capacity(self::AXONS_TOTAL);
	let mut column_states: Vec<ocl::cl_uint> = Vec::with_capacity(self::COLUMNS_TOTAL);
	let mut hypercolumn_states: Vec<ocl::cl_uint> = Vec::with_capacity(self::HYPERCOLUMNS_TOTAL);

	let mut pseudo_columns: [f32, ..COLUMNS_TOTAL] = [0.0f32, ..COLUMNS_TOTAL];

	println!("Initializing Vectors...");
	initialize_data_vectors(&mut my_cortex.my_cortex.synapses, &mut my_cortex.dendrite_thresholds, &mut dendrite_states, &mut axons, &mut column_states, &mut hypercolumn_states);

	println!("Initializing Pseudo Columns...");
	initialize_pseudo_columns(&mut pseudo_columns);

	
	// Create Buffers for Kernel Arguments
	println!("Creating Buffers...");
	let synapses_buff: ocl::cl_mem = ocl::new_write_buffer(&synapses, my_cortex.ocl.context);
	let dendrite_thresholds_buff: ocl::cl_mem = ocl::new_write_buffer(&dendrite_thresholds, my_cortex.ocl.context);
	let dendrite_states_buff: ocl::cl_mem = ocl::new_write_buffer(&dendrite_states, my_cortex.ocl.context);
	let axons_buff: ocl::cl_mem = ocl::new_write_buffer(&axons, my_cortex.ocl.context);
	let column_states_buff: ocl::cl_mem = ocl::new_write_buffer(&column_states, my_cortex.ocl.context);
	let hypercolumn_states_buff: ocl::cl_mem = ocl::new_write_buffer(&hypercolumn_states, my_cortex.ocl.context);


	// Fill Write Buffers
	println!("Writing Buffers...");
	ocl::enqueue_write_buffer(&synapses, synapses_buff, command_queue);
	ocl::enqueue_write_buffer(&dendrite_thresholds, dendrite_thresholds_buff, my_cortex.ocl.command_queue);
	ocl::enqueue_write_buffer(&dendrite_states, dendrite_states_buff, my_cortex.ocl.command_queue);
	ocl::enqueue_write_buffer(&axons, axons_buff, my_cortex.ocl.command_queue);
	ocl::enqueue_write_buffer(&column_states, column_states_buff, my_cortex.ocl.command_queue);
	ocl::enqueue_write_buffer(&hypercolumn_states, hypercolumn_states_buff, my_cortex.ocl.command_queue);
*/
	
	my_cortex.init();

/*
	println!("Performing Simple Test 1 (Synapses)...");

	simple_test(&my_cortex.synapses, my_cortex.synapses_buff, cortex::SYNAPSES_TOTAL, 0, "test_synapse", my_cortex.ocl.program, my_cortex.ocl.context, my_cortex.ocl.command_queue);

	println!("Performing Simple Test 2 (Axons)...");

	simple_test(&my_cortex.axons, my_cortex.axons_buff, cortex::AXONS_TOTAL, 0, "test_axon", my_cortex.ocl.program, my_cortex.ocl.context, my_cortex.ocl.command_queue);

*/

	my_cortex.readback_test(&my_cortex.synapses, my_cortex.synapses_buff, "test_synapse");
	my_cortex.readback_test(&my_cortex.axons, my_cortex.axons_buff, "test_axon");

// Free CL Memory

	my_cortex.release_components();
	
	
}

/*

fn simple_test<T: ToPrimitive + Clone>(
				test_source: &Vec<T>,
				test_source_buff: ocl::cl_mem, 
				len: uint, 
				zero_value: T,
				test_kernel_name: &str, 
				program : ocl::cl_program,
				context: ocl::cl_context, 
				command_queue: ocl::cl_command_queue,
			) {
	// test_synapse()
	let mut test_out: Vec<T> = Vec::with_capacity(len);
	for i in range(0u, test_out.capacity()) {
		test_out.push(zero_value.clone());
	}
	let test_out_buff: ocl::cl_mem = ocl::new_read_buffer(&test_out, context);
	let test_out_kernel: ocl::cl_kernel = ocl::new_kernel(program, test_kernel_name);
	ocl::set_kernel_arg(0, test_source_buff, test_out_kernel);
	ocl::set_kernel_arg(1, test_out_buff, test_out_kernel);
	ocl::enqueue_kernel(test_out_kernel, command_queue, test_source.len());
	ocl::enqueue_read_buffer(&test_out, test_out_buff, command_queue);
	let mut total = 0u;
	for x in range(0u, test_out.len()) {
		total += test_out[x].to_uint().unwrap();
	}
	println!("*** {} *** Total: {}; Len: {}; Avg: {}", test_kernel_name, total, test_out.len(), total/test_out.len());
}



fn initialize_pseudo_columns(pseudo_columns: &mut [f32, ..COLUMNS_TOTAL]) {
	let rng_range = Range::new(1f32, 500f32);
	let mut rng = rand::task_rng();

	for i in range(0u, COLUMNS_TOTAL) {
		pseudo_columns[i] = rng_range.ind_sample(&mut rng);
	}

}


fn initialize_data_vectors(
				my_cortex.synapses: &mut Vec<ocl::cl_uchar>,
				my_cortex.dendrite_thresholds: &mut Vec<ocl::cl_ushort>,
				dendrite_states: &mut Vec<ocl::cl_ushort>,
				axons: &mut Vec<ocl::cl_ushort>,
				column_states: &mut Vec<ocl::cl_uint>,
				hypercolumn_states: &mut Vec<ocl::cl_uint>,
) {
	//SYNAPSES
	let rng_range = Range::new(SYNAPSE_WEIGHT_ZERO - (SYNAPSE_WEIGHT_INITIAL_DEVIATION), SYNAPSE_WEIGHT_ZERO + (SYNAPSE_WEIGHT_INITIAL_DEVIATION) + 1);
	let mut rng = rand::task_rng();
	for i in range(0u, my_cortex.synapses.capacity()) {
		my_cortex.synapses.push(rng_range.ind_sample(&mut rng));
	}

	//DENDRITES
	for i in range(0u, my_cortex.dendrite_thresholds.capacity()) {
		my_cortex.dendrite_thresholds.push(DENDRITE_INITIAL_THRESHOLD);
	}

	for i in range(0u, dendrite_states.capacity()) {
		dendrite_states.push(0u16);
	}

	let rng_range = Range::new(0u16, 0xFFFEu16);
	let mut rng = rand::task_rng();
	for i in range(0u, axons.capacity()) {
		axons.push(rng_range.ind_sample(&mut rng));
	}

	for i in range(0u, column_states.capacity()) {
		column_states.push(0u32);
	}

	for i in range(0u, hypercolumn_states.capacity()) {
		hypercolumn_states.push(0u32);
	}
	

}
*/


