
extern crate libc;

use ocl;
use std;
use std::io;

pub const KERNELS_FILE_NAME: &'static str = "bismit.cl";

pub static VEC_SIZE: uint = 100u;

pub fn run_kernel() {

//Read Kernel File
	let file_path: std::path::Path = std::path::Path::new(format!("{}/{}/{}", env!("P"), "bismit/src", KERNELS_FILE_NAME));
	let kern_str = io::File::open(&file_path).read_to_end().unwrap();
	let kern_c_str = std::str::from_utf8(kern_str.as_slice()).unwrap().to_c_str();

// Create Parts and Pieces
	let platform: ocl::cl_platform_id = ocl::new_platform();

	let device: ocl::cl_device_id = ocl::new_device(platform);

	let context: ocl::cl_context = ocl::new_context(device);

	let program: ocl::cl_program = ocl::new_program(kern_c_str.as_ptr(), context, device);

	let kernel: ocl::cl_kernel = ocl::new_kernel(program, "my_kernel_func");

	let command_queue: ocl::cl_command_queue = ocl::new_command_queue(context, device);

// Initialize Data

	let mut a: Vec<ocl::cl_float> = Vec::with_capacity(self::VEC_SIZE);
	let mut b: Vec<ocl::cl_float> = Vec::with_capacity(self::VEC_SIZE);
	let mut c: Vec<ocl::cl_float> = Vec::with_capacity(self::VEC_SIZE);

	println!("Generating Data...");
	for x in range(0u, self::VEC_SIZE) {
		a.push(2f32 * x.to_f32().unwrap());
		b.push(1f32 * x.to_f32().unwrap());
		c.push(0f32);
	}

// Create Buffers for Kernel Arguments
	println!("Creating Buffers...");
	let a_buff: ocl::cl_mem = ocl::new_write_buffer(&a, context);

	let b_buff: ocl::cl_mem = ocl::new_write_buffer(&b, context);

	let c_buff: ocl::cl_mem = ocl::new_read_buffer(&c, context);


// Fill Write Buffers
	println!("Writing Buffers...");
	ocl::enqueue_write_buffer(&a, a_buff, command_queue);

	ocl::enqueue_write_buffer(&b, b_buff, command_queue);
	

// Set Kernel Arguments
	ocl::set_kernel_arg(0, a_buff, kernel);

	ocl::set_kernel_arg(1, b_buff, kernel);

	ocl::set_kernel_arg(2, c_buff, kernel);


// Execute Kernel
	println!("Enqueuing Kernel...");
	ocl::enqueue_kernel(kernel, command_queue, VEC_SIZE);	
// Read Results	
	ocl::enqueue_read_buffer(&c, c_buff, command_queue);
	
	
// Print Results
	println!("Totaling Results...");
	let mut total = 0f32;
	for x in range(0u, self::VEC_SIZE) {
		//println!("Results: {}: {} - {} = {}", x, a[x], b[x], c[x]);
		total += c[x];
	}

	println!("Total: {}",total);

// Free CL Memory
	
	ocl::release_mem_object(a_buff);
	ocl::release_mem_object(b_buff);
	ocl::release_mem_object(c_buff);
	ocl::release_components(kernel, command_queue, program, context);
	
}
