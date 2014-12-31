use cortex::{ Cortex };
use ocl;
use std::default::Default;

pub fn readback_test<T: ToPrimitive + Clone + Default>(
			cortex: &Cortex,
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
	let test_out_buff: ocl::cl_mem = ocl::new_read_buffer(&test_out, cortex.ocl.context);

	// Test Kernels simply clone the vector right now
	let test_out_kernel: ocl::cl_kernel = ocl::new_kernel(cortex.ocl.program, test_kernel_name);
	ocl::set_kernel_arg(0, test_source_buff, test_out_kernel);
	ocl::set_kernel_arg(1, test_out_buff, test_out_kernel);

	// Run Kernel then read from output buffer
	ocl::enqueue_kernel(test_out_kernel, cortex.ocl.command_queue, test_source.len());
	ocl::enqueue_read_buffer(&test_out, test_out_buff, cortex.ocl.command_queue);

	// Calculate sum total for output vector
	print_vec_info(test_source, test_kernel_name);
	print_vec_info(&test_out, test_kernel_name);
	
	// Run again 5 times using output as source
	ocl::set_kernel_arg(0, test_out_buff, test_out_kernel);

	for i in range(0u, 5) {
		//println!("... {}", i + 1);
		ocl::enqueue_kernel(test_out_kernel, cortex.ocl.command_queue, test_source.len());
		ocl::enqueue_read_buffer(&test_out, test_out_buff, cortex.ocl.command_queue);
		print_vec_info(&test_out, test_kernel_name);
	}

	//print_vec_info(test_source, test_kernel_name);
	//print_vec_info(&test_out, test_kernel_name);
	
	ocl::release_kernel(test_out_kernel);
}
 

pub fn print_vec_info<T: ToPrimitive + Clone + Default>(my_vec: &Vec<T>, info: &str) {
	let mut total = 0u;
	for x in range(0u, my_vec.len()) {
		total += my_vec[x].to_uint().unwrap();
	}
	println!("*** {} *** Total: {}; Len: {}; Avg: {}", info, total, my_vec.len(), total/my_vec.len());

}
