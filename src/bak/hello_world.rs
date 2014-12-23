use ocl;

pub fn run() {
	println!("=== hello_world::run() ===");

	let ocl = ocl::Ocl::new();

	let kernel_name: &str = "hello";
	let input_data: Vec<f32> = vec!(1f32, 2f32, 3f32, 4f32, 5f32, 6f32, 7f32, 8f32, 9f32, 0f32);
	let results: Vec<f32> = Vec::from_elem(10, 0f32);

	let hello_kernel: ocl::cl_kernel = ocl::new_kernel(ocl.program, kernel_name);

	let input_data_buff = ocl.new_write_buffer(&input_data);
	ocl.enqueue_write_buffer(&input_data, input_data_buff);

	let results_buff = ocl.new_write_buffer(&results);


	ocl::set_kernel_arg(0, input_data_buff, hello_kernel);
	ocl::set_kernel_arg(1, results_buff, hello_kernel);

	ocl.enqueue_kernel(hello_kernel, input_data.len());
	println!("Finished with Return Code: {}", ocl::cl_finish(ocl.command_queue));

	ocl.enqueue_read_buffer(&results, results_buff);

	println!("Results: {}", results);


	ocl::release_mem_object(input_data_buff);
	ocl::release_mem_object(results_buff);

	ocl.release_components();
}
