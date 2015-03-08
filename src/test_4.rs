
use cortex::{ Cortex };
use cortex;
use ocl;
use common;
use chord::{ Chord };
use envoy::{ Envoy };
//use axn_space::{ AxonSpace };


use std::default::Default;
use std::num::{ Int };
use std::iter;
use std::ops;
use time;


pub const TEST_ITERATIONS: i32 = 1000; 
pub const SHUFFLE_CHORDS: bool = false;
pub const PRINT_EVERY: i32 = 5000;

pub fn test_cycle() {
	let mut cortex = cortex::Cortex::new();

	//let vv1 = common::sparse_vec(2048, -128i8, 127i8, 6);
	//common::print_vec(&vv1, 1, false, Some(ops::Range{ start: -127, end: 127 }));

	//let mut vec1: Vec<i8> = common::shuffled_vec(1024, 0, 127);
	//let mut vec1: Vec<i8> = common::sparse_vec(2048, -128i8, 127i8, 8);

	//common::print_vec(&vec1, 1, false, Some(ops::Range{ start: -128, end: 127 }));
	let time_start = time::get_time();
	let scw = common::SENSORY_CHORD_WIDTH;
	let scl_fct = scw / 1024;

	print!("\n*********** scl_fct: {}", scl_fct);
	print!("\n*********** common::log2(sct_fct): {}", common::log2(scl_fct));

	let mut vec1: Vec<i8> = Vec::with_capacity(scw as usize);
	/*for i in range(0, scw) {
		if i < scw >> 1 {
			vec1.push(64i8);
		} else {
			vec1.push(0i8);
		}
	}*/

	let scw_1_2 = scw >> 1;

	let scw_1_4 = scw >> 2;
	let scw_3_4 = scw - scw_1_4;

	let scw_1_8 = scw >> 3;
	let scw_3_8 = scw_1_2 - scw_1_8;
	let scw_5_8 = scw_1_2 + scw_1_8;

	let scw_1_16 = scw >> 4;

	//println!("***** scw_1_4: {}, scw_3_4: {}", scw_1_4, scw_3_4);
	for i in range(0, scw) {
		if i >= scw_3_8 + scw_1_16 && i < scw_5_8 - scw_1_16 {
			vec1.push(64i8);
		} else {
			vec1.push(0i8);
		}
	}


	let shuffle_chords = SHUFFLE_CHORDS;


	if shuffle_chords {
		common::shuffle_vec(&mut vec1);
		//chord1 = Chord::from_vec(&vec1);
	}

	cortex.sense_vec(0, "pre-thal", &mut vec1);
	cortex.sense_vec(0, "post-thal", &mut vec1);
	
	
	/*for x in chord1.chord.iter() {
		print!("{:?}",x);
	}*/
	

		/* SENSE ONLY LOOP */
	print!("\n\nRunning sense only loops ... ");

	let sense_only_loops: i32 = TEST_ITERATIONS;

	let mut i = 0i32;
	loop {
		if i >= sense_only_loops { break; }

		if i % PRINT_EVERY == 0 || i < 0 {
			let t = time::get_time() - time_start;
			print!("\n[i:{}; {}.{}s] ", i, t.num_seconds(), t.num_milliseconds());
			/*if true {
				print!("\ncells.soma.hcol_max_ids: ");
				cortex.cells.soma.hcol_max_ids.print(1 << 0);
			}

			if true {
				print!("\ncells.soma.hcol_max_vals: ");
				cortex.cells.soma.hcol_max_vals.print(1 << 0);
			}

			if false {		
				println!("\ncells.soma.bsl_dst_dens.syns.strengths: ");
				cortex.cells.soma.bsl_dst_dens.syns.strengths.print_val_range(1 << 6, 17, 127);
			}*/

			/* AXON STATES */
			if false {
				print!("\ncells.axons.states: ");
				cortex.cells.axons.states.print_val_range(1 << 8, 1, 127);
			}
		}

		if shuffle_chords {
			common::shuffle_vec(&mut vec1);
			//chord1 = Chord::from_vec(&vec1);
		}


		cortex.sense_vec(0, "thal", &mut vec1);
		//cortex.sense(0, 0, &chord2);

		i += 1;
	}

	/*print!("{} sense only iterations complete: ", i);
	if true {
		print!("\ncells.axons.states: ");
		cortex.cells.axons.states.print(1 << 4);
	}*/

	




		/* SENSE AND PRINT LOOP */
	print!("\n\nRunning sense and print loops...");
	//let mut i = 0u32;
	loop {
		if i >= 1 + sense_only_loops { break; }

		print!("\n\n=== Iteration {} ===", i + 1);

		if false {
			println!("\ncells.axons.states: ");
			cortex.cells.axons.states.print(1 << 5);
		}

		cortex.sense_vec(0, "thal", &vec1); 

		/* COLUMN STATES */
		if true {	
			print!("\ncells.cols.states: ");
			cortex.cells.cols.states.print_val_range(1 << 7, -128, 127);
		}


		/* SYNAPSE STATES */

		if true {	
			print!("\ncells.cols.syns.states: ");
			cortex.cells.cols.syns.states.print_val_range(1 << 15, -128, 127);
		}


		/* HCOL MAX IDXS */

		/*if false {
			print!("\ncells.soma.hcol_max_ids: ");
			cortex.cells.soma.hcol_max_ids.print(1 << 0);
		}

		if false {
			print!("\ncells.soma.hcol_max_vals: ");
			cortex.cells.soma.hcol_max_vals.print(1 << 0);
		}*/


		/* SOMA STATES */

		/*if false {
			print!("\ncells.soma.states: ");
			cortex.cells.soma.states.print_val_range(1 << 12, 1, 127);
		}*/


		/* AXON STATES */

		if true {
			print!("\ncells.axons.states: ");
			cortex.cells.axons.states.print_val_range(1 << 10 as usize , 1, 127);
		}

		i += 1;
		println!("");
	}


	cortex.release_components();

}


		//
		// 128:1 RATIO FOR PRINTING IS COOL (100% ACTIVITY)
		// 512:1 (25% ACTIVITY, 262144 len)
		//

		//	println!("\n tmp_out: ");
		//	cortex.sensory_segments[0].tmp_out.print(1000);


		/* SYNAPSE COL_OFS (SRC_OFS) */

		/*print!("\ncells.cols.syns.src_ofs:");
		cortex.cells.cols.syns.src_ofs.print_val_range(1 << 12, -128, 127);*/

		/*if false {
			print!("\ncells.soma.bsl_dst_dens.syns.axn_col_offs:");
			cortex.cells.soma.bsl_dst_dens.syns.axn_col_offs.print(1 << 14);		// 16384

			print!("\ncells.cols.bsl_prx_dens.syns.axn_col_offs:");
			cortex.cells.cols.bsl_prx_dens.syns.axn_col_offs.print(1 << 16);
		}*/

		/* SYNAPSE AXN_ROW_IDS */

		/*if false {
			print!("\ncells.soma.bsl_dst_dens.syns.axn_row_ids:");
			cortex.cells.soma.bsl_dst_dens.syns.axn_row_ids.print(1 << 14);		// 16384
		}

		if false {
			print!("\ncells.cols.bsl_prx_dens.syns.axn_row_ids:");
			cortex.cells.cols.bsl_prx_dens.syns.axn_row_ids.print(1 << 10);
		}*/


		/* SYNAPSE STRENGTHS */

		/*if false {		
			println!("\ncells.soma.bsl_dst_dens.syns.strengths: ");
			cortex.cells.soma.bsl_dst_dens.syns.strengths.print_val_range(1 << 6, 17, 127);
		}

		if false {
			print!("\ncells.cols.bsl_prx_dens.syns.strengths: ");
			cortex.cells.cols.bsl_prx_dens.syns.strengths.print_val_range(1 << 4, 17, 127);
		}*/

		/*if true {	
			print!("\ncells.soma.dst_dens.syns.states: ");
			cortex.cells.soma.dst_dens.syns.states.print(1 << 14);
		}*/

		/*if true {
			print!("\ncells.cols.bsl_prx_dens.syns.states: ");
			cortex.cells.cols.bsl_prx_dens.syns.states.print(1 << 10);
		}*/

		/* DENDRITE STATES */

		/*if true {
			print!("\ncells.soma.bsl_dst_dens.states: ");
			cortex.cells.soma.bsl_dst_dens.states.print(1 << 10);
		}

		if true {
			print!("\ncells.cols.bsl_prx_dens.states: ");
			cortex.cells.cols.bsl_prx_dens.states.print(1 << 6);
		}*/


		/* AUX VALS */

		/*if true {
			print!("\ncells.aux.chars_0: ");
			cortex.cells.aux.chars_0.print(1 << 0);
		}

		if true {
			print!("\ncells.aux.chars_1: ");
			cortex.cells.aux.chars_1.print(1 << 0);
		}*/



	/*if false {
		println!("\n cells.somata.states: ");
		cortex.cells.somata.states.print(1 << 8);
	}*/


	/*let mut t3: Envoy<i8> = Envoy::new(64, 0i8, &cortex.ocl);

	println!("t3.vec.len(): {}", t3.vec.len());


	let tkern = ocl::new_kernel(cortex.ocl.program, "test_int_shift");

	ocl::set_kernel_arg(0, t3.buf, tkern);
	ocl::set_kernel_arg(1, -96i8, tkern);

	ocl::enqueue_kernel(tkern, cortex.ocl.command_queue, t3.vec.len());

	t3.read();

	common::print_vec(&t3.vec, 1, true);*/


	/***** Testing Axon Stuff *****
		let mut vec2: Vec<u8> = iter::repeat(0).take(1024 * 256).collect();

		let mut tar_idxs: Vec<usize> = iter::repeat(0).take(1024 * 256).collect();


		for i in range(0, vec2.len()) {

			tar_idxs[i] = ((cortex.sensory_segments[0].target_idxesses.target_column_bodies.vec[i] as usize) << 8 ) + cortex.sensory_segments[0].target_idxesses.target_column_dst_dens.syns.vec[i] as usize;

			//	print!("[{}: {}]", i, tar_idx);

			vec2[tar_idxs[i]] = vec1[i as usize >> 8us];
			

			//	*x = (tcb_vec[i] << 8) as u32 + tcs_vec[i] as u32;

		}

		//common::dup_check(&tar_idxs);

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

	//common::print_vec(&vec2, 10);

	for i in range(0, 5) {
		for x in vec2.iter_mut() {
			*x += 1000;
		}
		//common::print_vec(&vec2, 10);

		cortex.ocl.enqueue_write_buffer(&mut vec2, buf2, cortex.ocl.command_queue);
	}

	

	_buffer_test(&cortex, &vec2, buf2, "buffer_test");

	for i in range(0, 5) {
		for x in vec2.iter_mut() {
			*x += 1000;
		}
		//common::print_vec(&vec2, 10);

		ocl::enqueue_write_buffer(&mut vec2, buf2, cortex.ocl.command_queue);
	}

	//common::print_vec(&vec2, 10);
	_buffer_test(&cortex, &vec2, buf2, "buffer_test");
	

	cortex.release_components();

	println!("--- test4::buffer_test() complete ---");

}

fn _buffer_test<T: Clone + Default + Int>(
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
 

pub fn print_vec_info<T: Clone + Default + Int>(my_vec: &Vec<T>, info: &str) {
	let mut total = 0us;
	for x in range(0, my_vec.len()) {
		total += my_vec[x].to_uint().expect("test_4.print_vec_info()");
	}
	println!("[{}: total:{}; len:{}; avg:{}]", info, total, my_vec.len(), total/my_vec.len());

}
*/
