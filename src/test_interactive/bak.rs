


/*let peek_vec = world.peek_from(worm.uid).unfold();

	cmn::print_vec_simple(&peek_vec);

	world.entities().get_mut(worm.uid).turn(0.0004f32);*/


	//let mut worm_brain = EntityBrain::new(worm.uid, &world);
	//let mut snake_brain = SnakeBrain::new(snake.uid);

	//let chord = render_peek(world.peek_from(worm.uid));
	//chord.print();
	//chord.unfold().print();

	// for i in range(0, 100000) {
	/*for i in 0..5 {
		if worm_brain.act(&mut world) == Option::None {
			println!("");
			println!("Everything eaten after {} iterations.", i);
			break
		}
	}*/



	/*cortex.sense_vec(0, "pre-thal", &mut vec1);
	cortex.sense_vec(0, "post-thal", &mut vec1);*/







/*pub fn pe<T: Integer + Copy + Clone + NumCast + Default + Display + FromPrimitive + ToPrimitive, V>(label: &'static str, env: &Envoy<T>, scale: usize, 
				val_range: Option<(V, V)>, 
				idx_range: Option<(usize, usize)>
) {
	print!("\n{}: ", label);
	env.len();
	//env.print(scale, val_range, idx_range);
}*/


		//
		// 128:1 RATIO FOR PRINTING IS COOL (100% ACTIVITY)
		// 512:1 (25% ACTIVITY, 262144 len)
		//

		//	println!("\n tmp_out: ");
		//	cortex.sensory_segments[0].tmp_out.print(1000);


		/* SYNAPSE COL_OFS (SRC_OFS) */

		/*print!("\ncols.syns.src_ofs:");
		cortex.region_cells.cols.syns.src_ofs.print_val_range(1 << 12, -128, 127);*/

		/*if false {
			print!("\nsoma.bsl_dst_dens.syns.src_col_x_offs:");
			cortex.region_cells.soma.bsl_dst_dens.syns.src_col_x_offs.print(1 << 14);		// 16384

			print!("\ncols.bsl_prx_dens.syns.src_col_x_offs:");
			cortex.region_cells.cols.bsl_prx_dens.syns.src_col_x_offs.print(1 << 16);
		}*/

		/* SYNAPSE AXN_ROW_IDS */

		/*if false {
			print!("\nsoma.bsl_dst_dens.syns.src_row_ids:");
			cortex.region_cells.soma.bsl_dst_dens.syns.src_row_ids.print(1 << 14);		// 16384
		}

		if false {
			print!("\ncols.bsl_prx_dens.syns.src_row_ids:");
			cortex.region_cells.cols.bsl_prx_dens.syns.src_row_ids.print(1 << 10);
		}*/


		/* SYNAPSE STRENGTHS */

		/*if false {		
			println!("\nsoma.bsl_dst_dens.syns.strengths: ");
			cortex.region_cells.soma.bsl_dst_dens.syns.strengths.print_val_range(1 << 6, 17, 127);
		}

		if false {
			print!("\ncols.bsl_prx_dens.syns.strengths: ");
			cortex.region_cells.cols.bsl_prx_dens.syns.strengths.print_val_range(1 << 4, 17, 127);
		}*/

		/*if true {	
			print!("\nsoma.dst_dens.syns.states: ");
			cortex.region_cells.soma.dst_dens.syns.states.print(1 << 14);
		}*/

		/*if true {
			print!("\ncols.bsl_prx_dens.syns.states: ");
			cortex.region_cells.cols.bsl_prx_dens.syns.states.print(1 << 10);
		}*/

		/* DENDRITE STATES */

		/*if true {
			print!("\nsoma.bsl_dst_dens.states: ");
			cortex.region_cells.soma.bsl_dst_dens.states.print(1 << 10);
		}

		if true {
			print!("\ncols.bsl_prx_dens.states: ");
			cortex.region_cells.cols.bsl_prx_dens.states.print(1 << 6);
		}*/


		/* AUX VALS */

		/*if true {
			print!("\naux.chars_0: ");
			cortex.region_cells.aux.chars_0.print(1 << 0);
		}

		if true {
			print!("\naux.chars_1: ");
			cortex.region_cells.aux.chars_1.print(1 << 0);
		}*/



	/*if false {
		println!("\n region_cells.somata.states: ");
		cortex.region_cells.somata.states.print(1 << 8);
	}*/


	/*let mut t3: Envoy<i8> = Envoy::new(64, 0i8, &cortex.ocl);

	println!("t3.vec.len(): {}", t3.vec.len());


	let tkern = ocl::new_kernel(cortex.ocl.program, "test_int_shift");

	ocl::set_kernel_arg(0, t3.buf, tkern);
	ocl::set_kernel_arg(1, -96i8, tkern);

	ocl::enqueue_kernel(tkern, cortex.ocl.command_queue, t3.vec.len());

	t3.read();

	cmn::print_vec(&t3.vec, 1, true);*/


	/***** Testing Axon Stuff *****
		let mut vec2: Vec<u8> = iter::repeat(0).take(1024 * 256).collect();

		let mut tar_idxs: Vec<usize> = iter::repeat(0).take(1024 * 256).collect();


		for i in range(0, vec2.len()) {

			tar_idxs[i] = ((cortex.sensory_segments[0].target_idxesses.target_column_bodies.vec[i] as usize) << 8 ) + cortex.sensory_segments[0].target_idxesses.target_column_dst_dens.syns.vec[i] as usize;

			//	print!("[{}: {}]", i, tar_idx);

			vec2[tar_idxs[i]] = vec1[i as usize >> 8us];
			

			//	*x = (tcb_vec[i] << 8) as u32 + tcs_vec[i] as u32;

		}

		//cmn::dup_check(&tar_idxs);

		//println!("First 3: {}, {}, {}", tar_idxs[0], tar_idxs[1], tar_idxs[2]);
		//println!("Last 3: {}, {}, {}", tar_idxs[tar_idxs.len() - 1], tar_idxs[tar_idxs.len() - 2], tar_idxs[tar_idxs.len() - 3]);

	*******************/

	



/*pub fn buffer_test() {
	println!("--- test4::buffer_test() ---");
	let mut cortex = cortex::Cortex::new();

	let mut vec1: Vec<u32> = Vec::new();
	for i in range(0, 1024) {
		vec1.push(i as u32);
	}

	let init_val = 0u32;
	let size = 50;

	let mut vec2: Vec<u32> = iter::repeat(init_val).take(size).collect();
	let buf2: ocl::cl_mem = ocl::new_write_buffer(&mut vec2, cortex.ocl.context);

	//cmn::print_vec(&vec2, 10);

	for i in range(0, 5) {
		for x in vec2.iter_mut() {
			*x += 1000;
		}
		//cmn::print_vec(&vec2, 10);

		cortex.ocl.enqueue_write_buffer(&mut vec2, buf2, cortex.ocl.command_queue);
	}

	

	_buffer_test(&cortex, &vec2, buf2, "buffer_test");

	for i in range(0, 5) {
		for x in vec2.iter_mut() {
			*x += 1000;
		}
		//cmn::print_vec(&vec2, 10);

		ocl::enqueue_write_buffer(&mut vec2, buf2, cortex.ocl.command_queue);
	}

	//cmn::print_vec(&vec2, 10);
	_buffer_test(&cortex, &vec2, buf2, "buffer_test");
	

	cortex.release_components();

	println!("--- test4::buffer_test() complete ---");

}

fn _buffer_test<T: Clone + Default + Integer>(
			cortex: &Cortex,
			test_source: &Vec<T>,
			test_source_buff: ocl::cl_mem, 
			test_kernel_name: &str, 

) {
	//print_vec_info(test_source, test_kernel_name);
	println!("Performing buffer test with {} iterations.", READBACK_TEST_ITERATIONS);

	let time_start = time::get_time().sec;
	//println!("Timer Started at: {:?}...", time::now().to_timespec());

	// Create Vec for output
	let mut test_out: Vec<T> = Vec::with_capacity(test_source.len());
	for i in range(0, test_out.capacity()) {
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

	//println!("");
	//print_vec_info(&test_out, test_kernel_name);
	
	// Run again 5 times using output as source
	ocl::set_kernel_arg(0, test_out_buff, test_out_kernel);

	

	for i in range(0, READBACK_TEST_ITERATIONS) {
		//println!("... {}", i + 1);
		ocl::enqueue_kernel(test_out_kernel, cortex.ocl.command_queue, test_source.len());
		//ocl::enqueue_read_buffer(&test_out, test_out_buff, cortex.ocl.command_queue);
		//print_vec_info(&test_out, test_kernel_name);

	}

	//println!("Readback loops complete, reading buffer...");

	ocl::enqueue_read_buffer(&test_out, test_out_buff, cortex.ocl.command_queue);

	let time_stop = time::get_time().sec;
	let time_complete = time_stop - time_start;

	//print_vec_info(&test_out, test_kernel_name);

	println!("	{} Iterations complete in: {} sec.", READBACK_TEST_ITERATIONS, time_complete);

	//print_vec_info(test_source, test_kernel_name);
	print!("	");
	print_vec_info(&test_out, test_kernel_name);
	
	ocl::release_kernel(test_out_kernel);
}
 

pub fn print_vec_info<T: Clone + Default + Integer>(my_vec: &Vec<T>, info: &str) {
	let mut total = 0us;
	for x in range(0, my_vec.len()) {
		total += my_vec[x].to_uint().expect("test_4.print_vec_info()");
	}
	println!("[{}: total:{}; len:{}; avg:{}]", info, total, my_vec.len(), total/my_vec.len());

}
*/
