
use cortex::{ Cortex };
use cortex;
use ocl;
use common;
use chord::{ Chord };
use envoy::{ Envoy };
//use axn_space::{ AxonSpace };

use microcosm::entity::{ EntityBody, EntityKind, EntityBrain, Mobile };
use microcosm::worm::{ WormBrain };
use microcosm::common::{ Location, Peek, Scent, WORM_SPEED, TAU };
use microcosm::world::{ World };


use std::default::Default;
//use std::num::{ NumCast, FromPrimitive, ToPrimitive };
use num::{ self, Integer, NumCast, FromPrimitive, ToPrimitive };
use std::fmt::{ Display };
use std::ops;
use std::io::{ self, Write };
use std::borrow::{ Borrow };
use time;


pub const TEST_ITERATIONS: i32 			= 10; 
pub const PRINT_EVERY: i32 				= 400;

pub const SHUFFLE_ONCE: bool 			= true;
pub const SHUFFLE_EVERY: bool 			= false;


pub fn test_cycle() -> bool {
	let mut cortex = cortex::Cortex::new();

	//let vv1 = common::sparse_vec(2048, -128i8, 127i8, 6);
	//common::print_vec(&vv1, 1, false, Some(ops::Range{ start: -127, end: 127 }));

	//let mut vec1: Vec<i8> = common::shuffled_vec(1024, 0, 127);
	//let mut vec1: Vec<i8> = common::sparse_vec(2048, -128i8, 127i8, 8);

	//common::print_vec(&vec1, 1, false, Some(ops::Range{ start: -128, end: 127 }));
	let time_start = time::get_time();
	let scw = common::SENSORY_CHORD_WIDTH;
	let scl_fct_log2 = common::log2(scw / 1024);

	//print!("\n*********** scl_fct: {}", scl_fct);
	//print!("\n*********** common::log2(sct_fct): {}", common::log2(scl_fct));

	let mut vec1: Vec<ocl::cl_uchar> = Vec::with_capacity(scw as usize);
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
	for i in 0..scw {
		if i >= scw_3_8 + scw_1_16 && i < scw_5_8 - scw_1_16 {
		//if i >= scw_3_8 && i < scw_5_8 {
			vec1.push(0);
		} else {
			vec1.push(0);
		}
	}

	cortex.write_vec(0, "pre_thal", &mut vec1);
	cortex.write_vec(0, "post_thal", &mut vec1);


	vec1.clear();
	for i in 0..scw {
		if i >= scw_1_2 - (scw_1_16 / 2) && i < scw_1_2 + (scw_1_16 / 2) {
		//if ((i >= scw_1_4 - scw_1_16) && (i < scw_1_4 + scw_1_16)) || ((i >= scw_3_4 - scw_1_16) && (i < scw_3_4 + scw_1_16)) {
		//if i >= scw_3_8 && i < scw_5_8 {
		//if (i >= scw_1_2 - scw_1_16 && i < scw_1_2 + scw_1_16) || (i < scw_1_16) || (i >= (scw - scw_1_16)) {
		//if i >= scw_3_8 && i < scw_5_8 {
		//if i < scw_1_16 {
			vec1.push(254);
		} else {
			vec1.push(0);
		}
	}



	/*if SHUFFLE_ONCE {
		common::shuffle_vec(&mut vec1);
		//chord1 = Chord::from_vec(&vec1);
	}*/



	let mut world: World = World::new(common::SENSORY_CHORD_WIDTH);

	let worm =  EntityBody::new("worm", EntityKind::Creature, Location::origin());

	world.entities().add(worm);
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, -220f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, 220f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, -220f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, 220f32)));
	//world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-0.0001f32, 0.0001f32)));

	world.entities().print();

	/*let peek_vec = world.peek_from(worm.uid).unfold();

	common::print_vec_simple(&peek_vec);

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
	
	
	/*for x in chord1.chord.iter() {
		print!("{:?}",x);
	}*/

	let mut test_iters: i32 = TEST_ITERATIONS;
	let mut first_run: bool = true;

	loop {



		let mut in_string: String = if first_run {
			first_run = false;
			"\n".to_string()
		} else {
			rin(format!("test_4:([q]uit [i]ters)[i={}]", test_iters))
			//print!("You Entered: {}", &in_string);
		};


		if "q\n" == in_string {
			println!("Quitting...");
			break;
		} else if "i\n" == in_string {
			let in_s = rin(format!("Enter number of iterations[Enter=default]"));
			if "\n" == in_s {
				test_iters = TEST_ITERATIONS;
			} else {
				let in_int: Option<i32> = in_s.trim().parse().ok();
				match in_int {
					Some(x)	=> {
						//print!("\nLooping {} Times... ", x);
						 //test_iters = num::cast(x).expect("test_4::test_cycle(): Error casting.");
						 test_iters = x;
					},
					None    => {
						print!("\nError parsing number.");
						continue;
					},
				}
			}
		} else if "\n" == in_string {
			// Go
		} else {
			continue;
		}


		let time_start = time::get_time();

			/* SENSE ONLY LOOP */
		print!("\nRunning {} sense only loop(s) ... ", test_iters - 1);

		let mut i = 0i32;

		//println!("");

		loop {
			if i >= (test_iters - 1) { break; }

			if i % PRINT_EVERY == 0 || i < 0 {
				let t = time::get_time() - time_start;
				print!("\n[i:{}; {}.{}s] ", i, t.num_seconds(), t.num_milliseconds());
	 

					/* AXON STATES */
				if false {
					print!("\naxns.states: ");
					cortex.cells.axns.states.print_val_range(1 << 8, Some((1, 255)));
				}

				if false {
					print!("\nREGION OUTPUT: cells.axns.states: ");
					//cortex.cells.axns.states.print_val_range(1 << (0 + scl_fct_log2) as usize , 1, 63);
					cortex.cells.axns.states.print((1 << 0) as usize, Some((1, 255)), Some(cortex.cells.cols.axn_output_range()), true);
				}
				if false {
					print!("\ncols.syns.strengths:");
					cortex.cells.cols.syns.strengths.print(1 << 0, None, Some((256, 288)), true);
				}
				if false{	
					print!("\ncols..syns.src_col_offs: ");
					cortex.cells.cols.syns.src_col_offs.print(1 << 0, None, Some((256, 288)), true);
				}

				if false {
					print!("\npyrs.dens.syns.strengths:");
					cortex.cells.pyrs.dens.syns.strengths.print(1 << 0, None, Some((256, 319)), true);
				}
			}
						
			act(&mut world, worm.uid, &mut vec1);
			cortex.sense_vec(0, "thal", &mut vec1);
			
			/*if SHUFFLE_EVERY {
				common::shuffle_vec(&mut vec1);
			}
			cortex.sense_vec(0, "thal", &mut vec1);*/

			//common::print_vec_simple(&peek_vec);
			//print!("[i:{}]",i);
			i += 1;
		}



			/* SENSE AND PRINT LOOP */
		print!("\n\nRunning {} sense and print loop(s)...", 1usize);

		loop {
			if i >= (test_iters) { break; }

			print!("\n\n=== Iteration {}/{} ===", i + 1, test_iters);

			/* INITIAL AXON STATES */
			if false {
				println!("\naxns.states: ");
				cortex.cells.axns.states.print_val_range(1 << (0 + scl_fct_log2), None);
			}

			
			act(&mut world, worm.uid, &mut vec1);
			cortex.sense_vec(0, "thal", &mut vec1);

			/*if SHUFFLE_EVERY {
				common::shuffle_vec(&mut vec1);
			}
			cortex.sense_vec(0, "thal", &vec1); 
			*/
			
			let sr_start = (512 << common::SYNAPSES_PER_CELL_PROXIMAL_LOG2) as usize;

			if true {
				print!("\nInput Vec:");
				common::print_vec(&vec1, 1 , None, None, false);
			}


			/* SYNAPSE IDS & STRENGTHS */
			if false {
				print!("\ncols.syns.src_row_ids:");
				cortex.cells.cols.syns.src_row_ids.print(1 << 14, None, None, true);
			}
			if false{	
				print!("\ncols..syns.src_col_offs: ");
				cortex.cells.cols.syns.src_col_offs.print(1 << 14, None, None, true);
				//cortex.cells.cols.syns.states.print((1 << 8) as usize, None, None);
			}
			if true {
				print!("\ncols.syns.strengths:");
				cortex.cells.cols.syns.strengths.print(1 << 14, None, None, true);
			}

			if false {	
				print!("\npyrs.dens.syns.src_row_ids: ");
				cortex.cells.pyrs.dens.syns.src_row_ids.print(1 << 14, None, None, true);
				//cortex.cells.cols.syns.states.print((1 << 8) as usize, None, None);
			}
			if false {	
				print!("\npyrs.dens.syns.src_col_offs: ");
				cortex.cells.pyrs.dens.syns.src_col_offs.print(1 << 14, None, None, true);
				//cortex.cells.cols.syns.states.print((1 << 8) as usize, None, None);
			}
			if true {
				print!("\npyrs.dens.syns.strengths:");
				cortex.cells.pyrs.dens.syns.strengths.print(1 << 19, None, None, true);
			}



			/* PROXIMAL (COLUMN) SYNAPSE STATES */
			if true {	
				print!("\ncols.syns.states: ");
				cortex.cells.cols.syns.states.print(1 << 12, Some((1, 255)), None, true);
				//cortex.cells.cols.syns.states.print((1 << 8) as usize, None, None);
			}

			/* COLUMN STATES */
			if true {	
				print!("\ncols.states: ");
				cortex.cells.cols.states.print_val_range(1 << 0, Some((1, 255)));
			}

			if false {	
				print!("\ncols.states_raw: ");
				cortex.cells.cols.states_raw.print_val_range(1 << 0, Some((1, 255)));
			}


			/* PYRAMIDAL SYNAPSE STATES */
			if false {	
				print!("\npyrs.dens.syns.states: ");
				cortex.cells.pyrs.dens.syns.states.print(1 << 16, Some((1, 255)), None, true);
				//cortex.cells.cols.syns.states.print((1 << 8) as usize, None, None);
			}

			/* PYRAMIDAL DENDRITE STATES */
			if true {	
				print!("\npyrs.dens.states: ");
				cortex.cells.pyrs.dens.states.print_val_range(1 << 12, Some((1, 255)));
			}
			if false {	
				print!("\npyrs.dens.states_raw: ");
				cortex.cells.pyrs.dens.states_raw.print_val_range(1 << 12, Some((1, 255)));
			}


			/* ASPINY IDS */
			if false {
				print!("\npeak_cols.col_ids: ");
				cortex.cells.cols.peak_cols.col_ids.print_val_range(1 << 0, Some((0, 255)));
			}


			/* ASPINY STATES */
			if false {
				print!("\npeak_cols.states: ");
				cortex.cells.cols.peak_cols.states.print_val_range(1 << 0, Some((1, 255)));
			}


			/* PYRAMIDAL CELL STATES */
			if true {
				print!("\npyrs.states: ");
				cortex.cells.pyrs.states.print_val_range(1 << 8, Some((1, 255)));
			}


			/* AXON STATES */
			if false {
				print!("\naxns.states: ");
				//cortex.cells.axns.states.print_val_range(1 << (0 + scl_fct_log2) as usize , 1, 63);
				cortex.cells.axns.states.print((1 << 4) as usize, Some((1, 255)), None, true);

			}
			if true {
				print!("\nREGION OUTPUT: cells.axns.states: ");
				//cortex.cells.axns.states.print((1 << 0) as usize, Some((1, 255)), Some((3000, 4423)));
				cortex.cells.axns.states.print((1 << 0) as usize, Some((0, 255)), Some(cortex.cells.cols.axn_output_range()), false);

			}


			/* AUX VALS */
			if true {
				print!("\naux.ints_0: ");
				cortex.cells.aux.ints_0.print((1 << 0) as usize, None, None, false);
				print!("\naux.ints_1: ");
				cortex.cells.aux.ints_1.print((1 << 16) as usize, None, None, false);
			}

			if false {
				print!("\naux.chars_0: ");
				cortex.cells.aux.chars_0.print((1 << 0) as usize, Some((-128, 127)), None, true);
				//pe("aux.chars_1", &cortex.cells.aux.chars_1, (1 << 0) as usize, Some((-128, 127)), None, true);
				print!("\naux.chars_1: ");
				cortex.cells.aux.chars_1.print((1 << 0) as usize, Some((-128, 127)), None, true);
			}

			i += 1;
			
		}
	}

	println!("");

	cortex.release_components();
	true

}

fn act(world: &mut World, ent_uid: usize, vec: &mut Vec<u8>) {
	world.entities().get_mut(ent_uid).turn(0.0005f32);
	world.peek_from(ent_uid).unfold_into(vec, 0);
}


/*pub fn pe<T: Integer + Copy + Clone + NumCast + Default + Display + FromPrimitive + ToPrimitive, V>(label: &'static str, env: &Envoy<T>, scale: usize, 
				val_range: Option<(V, V)>, 
				idx_range: Option<(usize, usize)>
) {
	print!("\n{}: ", label);
	env.len();
	//env.print(scale, val_range, idx_range);
}*/

fn rin(prompt: String) -> String {
	let mut in_string: String = String::new();
	print!("\n{}:> ", prompt);
	io::stdout().flush().unwrap();
	io::stdin().read_line(&mut in_string).ok().expect("Failed to read line");
	in_string
}


		//
		// 128:1 RATIO FOR PRINTING IS COOL (100% ACTIVITY)
		// 512:1 (25% ACTIVITY, 262144 len)
		//

		//	println!("\n tmp_out: ");
		//	cortex.sensory_segments[0].tmp_out.print(1000);


		/* SYNAPSE COL_OFS (SRC_OFS) */

		/*print!("\ncols.syns.src_ofs:");
		cortex.cells.cols.syns.src_ofs.print_val_range(1 << 12, -128, 127);*/

		/*if false {
			print!("\nsoma.bsl_dst_dens.syns.src_col_offs:");
			cortex.cells.soma.bsl_dst_dens.syns.src_col_offs.print(1 << 14);		// 16384

			print!("\ncols.bsl_prx_dens.syns.src_col_offs:");
			cortex.cells.cols.bsl_prx_dens.syns.src_col_offs.print(1 << 16);
		}*/

		/* SYNAPSE AXN_ROW_IDS */

		/*if false {
			print!("\nsoma.bsl_dst_dens.syns.src_row_ids:");
			cortex.cells.soma.bsl_dst_dens.syns.src_row_ids.print(1 << 14);		// 16384
		}

		if false {
			print!("\ncols.bsl_prx_dens.syns.src_row_ids:");
			cortex.cells.cols.bsl_prx_dens.syns.src_row_ids.print(1 << 10);
		}*/


		/* SYNAPSE STRENGTHS */

		/*if false {		
			println!("\nsoma.bsl_dst_dens.syns.strengths: ");
			cortex.cells.soma.bsl_dst_dens.syns.strengths.print_val_range(1 << 6, 17, 127);
		}

		if false {
			print!("\ncols.bsl_prx_dens.syns.strengths: ");
			cortex.cells.cols.bsl_prx_dens.syns.strengths.print_val_range(1 << 4, 17, 127);
		}*/

		/*if true {	
			print!("\nsoma.dst_dens.syns.states: ");
			cortex.cells.soma.dst_dens.syns.states.print(1 << 14);
		}*/

		/*if true {
			print!("\ncols.bsl_prx_dens.syns.states: ");
			cortex.cells.cols.bsl_prx_dens.syns.states.print(1 << 10);
		}*/

		/* DENDRITE STATES */

		/*if true {
			print!("\nsoma.bsl_dst_dens.states: ");
			cortex.cells.soma.bsl_dst_dens.states.print(1 << 10);
		}

		if true {
			print!("\ncols.bsl_prx_dens.states: ");
			cortex.cells.cols.bsl_prx_dens.states.print(1 << 6);
		}*/


		/* AUX VALS */

		/*if true {
			print!("\naux.chars_0: ");
			cortex.cells.aux.chars_0.print(1 << 0);
		}

		if true {
			print!("\naux.chars_1: ");
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
