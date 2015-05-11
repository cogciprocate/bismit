
use super::synapse_drill_down;

use cortex::{ Cortex };
use cortex;
use ocl;
use cmn;
use chord::{ Chord };
use ocl::{ Envoy };
use microcosm::entity::{ EntityBody, EntityKind, EntityBrain, Mobile };
use microcosm::worm::{ WormBrain };
use microcosm::common::{ Location, Peek, Scent, WORM_SPEED, TAU };
use microcosm::world::{ World };
use motor_state;

use std::default::Default;
use std::iter;
use std::fmt::{ Display };
use std::ops;
use std::io::{ self, Write, Stdout };
use std::borrow::{ Borrow };
use rand::{ self, ThreadRng, Rng };
use num::{ self, Integer, NumCast, FromPrimitive, ToPrimitive };
use time;

pub const INITIAL_TEST_ITERATIONS: i32 	= 1; 
pub const STATUS_EVERY: i32 			= 1000;
pub const PRINT_DETAILS_EVERY: i32		= 10000;
pub const SHUFFLE_ONCE: bool 			= true;
pub const SHUFFLE_EVERY: bool 			= false;
pub const WORLD_TURN_FACTOR: f32 		= 3f32;	


/* RUN(): Run the interactive testing command line
	- TODO:
		- [incomplete][priority:very low] Proper command line using enums to represent user input and a seperate struct to manage its state
			- Or just be lazy and leave it the beautiful disaster that it is...	
*/
pub fn run() -> bool {
	let sc_width = cmn::SENSORY_CHORD_WIDTH;
	let mut cortex = cortex::Cortex::new();
	let mut world: World = World::new(sc_width);

	let mut vec1: Vec<u8> = iter::repeat(0).take(sc_width as usize).collect();
	//let mut vec1: Vec<ocl::cl_uchar> = test_vec_init(&mut cortex);

	let mut vec_out_prev: Vec<u8> = iter::repeat(0).take(sc_width as usize).collect();
	let mut vec_ff_prev: Vec<u8> = iter::repeat(0).take(sc_width as usize).collect();

	//let mut vec2: Vec<ocl::cl_uchar> = iter::repeat(0).take(sc_width as usize).collect();
	//cortex.write_vec(0, "pre_thal", &mut vec2);
	//cortex.write_vec(0, "post_thal", &mut vec2);
	//cortex.write_vec(0, "post_thal2", &mut vec2);
	//cortex.write_vec(0, "post_thal3", &mut vec2);
	//cortex.write_vec(0, "post_thal4", &mut vec2);
	//cortex.write_vec(0, "post_thal5", &mut vec2);

	
	let worm =  EntityBody::new("worm", EntityKind::Creature, Location::origin());

	world.entities().add(worm);
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, -220f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, 220f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, -220f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, 220f32)));

	//world.entities().print();

	let mut motor_state = motor_state::MotorState::new();

	let mut rng: rand::XorShiftRng = rand::weak_rng();
	let mut turn_bomb_i = 0usize;
	let mut turn_bomb_n = rng.gen::<u8>() as usize;
	
	let mut test_iters: i32 = INITIAL_TEST_ITERATIONS;
	let mut first_run: bool = true;
	let mut bypass_act = false;
	let mut bypass_sense = false;

	let mut view_all_axons: bool = false;
	let mut view_sdr_only: bool = true;
	let mut cur_ttl_iters: i32 = 0;

	loop {
		/*######### COMMAND LINE #########*/
		let vso = if view_sdr_only { "sdr" } else { "all" };

		bypass_act = false;
		bypass_sense = false;

		if test_iters == 0 {
			test_iters = 1;
			bypass_act = true;
			bypass_sense = true;
		}

		let mut in_string: String = if first_run {
			first_run = false;
			"\n".to_string()
		} else {
			rin(format!("<{}>bismit: [q]uit [i]ters [v]iew [a]xons [t]ests [m]otor [i={} v={} m={}]", 
				cur_ttl_iters, test_iters, vso, motor_state.cur_str()))
		};

		if "q\n" == in_string {
			println!("Quitting...");
			break;
		} else if "i\n" == in_string {
			let in_s = rin(format!("iters: [i={}]", test_iters));
			if "\n" == in_s {
				continue;
				//test_iters = TEST_ITERATIONS;
			} else {
				let in_int: Option<i32> = in_s.trim().parse().ok();
				match in_int {
					Some(x)	=> {
						 test_iters = x;
						 //continue;
					},
					None    => {
						print!("\nError parsing number.");
						continue;
					},
				}
			}
		} else if "v\n" == in_string {
			view_sdr_only = !view_sdr_only;
			bypass_act = true;
			bypass_sense = true;
		} else if "\n" == in_string {
			// Go
		} else if "a\n" == in_string {
			view_all_axons = !view_all_axons;
			bypass_act = true;
			bypass_sense = true;
		} else if "t\n" == in_string {
			bypass_act = true;
			bypass_sense = true;
			let in_s = rin(format!("tests: [p]yrs [c]ols [f]ract"));
			if "p\n" == in_s {
				synapse_drill_down::print_pyrs(&mut cortex);
				//println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
				continue;
				//test_iters = TEST_ITERATIONS;
			} else if "c\n" == in_s {
				synapse_drill_down::print_cols(&mut cortex);
				//println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
				continue;
				//test_iters = TEST_ITERATIONS;
			} else if "f\n" == in_s {
				let in_s = rin(format!("fractal seed"));
				let in_int: Option<u8> = in_s.trim().parse().ok();

				let seed = match in_int {
					Some(x)	=> x,
					None => {
						print!("\nError parsing number.");
						continue;
					},
				};

				let in_s = rin(format!("cardinality factor: 256 * "));
				let in_int: Option<usize> = in_s.trim().parse().ok();

				let c_factor = match in_int {
					Some(x)	=> x,
					None => {
						print!("\nError parsing number.");
						continue;
					},
				};

				let tvec = cmn::gen_fract_sdr(seed, 256 * c_factor);
				cmn::print_vec_simple(&tvec);
				//println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
				continue;
				//test_iters = TEST_ITERATIONS;
			} else {
				continue;
			}
		} else if "m\n" == in_string {
			bypass_act = true;
			bypass_sense = true;
			let in_s = rin(format!("motor: [s]witch"));
			if "s\n" == in_s {
				motor_state.switch();
				//println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
				continue;
				//test_iters = TEST_ITERATIONS;

			/*
			} else if "c\n" == in_s {
				synapse_drill_down::print_cols(&mut cortex);
				//println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
				continue;
				//test_iters = TEST_ITERATIONS;

			
			} else if "f\n" == in_s {
				let in_s = rin(format!("fractal seed"));
				let in_int: Option<u8> = in_s.trim().parse().ok();
				let seed = match in_int {
					Some(x)	=> x,
					None => {
						print!("\nError parsing number.");
						continue;
					},
				};
				let tvec = cmn::gen_fract_sdr(seed, 256 * 1);
				cmn::print_vec_simple(&tvec);
				//println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
				continue;
				//test_iters = TEST_ITERATIONS;
			*/


			} else {
				continue;
			}
		} else {
			continue;
		}

		let time_start = time::get_time();



		/*######### SENSE ONLY LOOP #########*/
		if !view_sdr_only { print!("\nRunning {} sense only loop(s) ... \n", test_iters - 1); }

		let mut i = 0i32;

		loop {
			if i >= (test_iters - 1) { break; }

			if i % STATUS_EVERY == 0 || i < 0 {
				let t = time::get_time() - time_start;
				if i >= 1 {
					print!("[{}: {:02.4}ms]", i, t.num_milliseconds());
				}
				io::stdout().flush().ok();
			}

			if i % PRINT_DETAILS_EVERY == 0 || i < 0 {
				if !view_sdr_only { 
					print_sense_only(&mut cortex); 
				} else { 
					//print!("\n");
				}
			}
						
			if !bypass_act {
				act(&mut world, worm.uid, &mut vec1, motor_state.cur_turn());
			}

			if !bypass_sense {
				cortex.write_vec(0, "thal", &mut vec1);
				cortex.sense_vec(0, "motor", &mut motor_state.cur_sdr());
			}

			i += 1;
			turn_bomb_i += 1;

			if turn_bomb_i >= turn_bomb_n {
				//print!(" >- pow!:{} -< ", turn_bomb_i);
				//motor_state.switch();
				turn_bomb_i = 0;
				turn_bomb_n = (rng.gen::<u8>() as usize) << 1;
			}
		}



		/*######### SENSE AND PRINT LOOP #########*/
		if !view_sdr_only { print!("\n\nRunning {} sense and print loop(s)...", 1usize); }

		loop {
			if i >= (test_iters) { break; }

			let (out_start, out_end) = cortex.region_cells.cols.axn_output_range();
			let axn_space_len = cortex.region_cells.axns.states.vec.len();

			{
				let out_slice_prev = &cortex.region_cells.axns.states.vec[out_start..(out_end + 1)];
				let ff_slice_prev = &cortex.region_cells.cols.states.vec[..];

				vec_out_prev.clone_from_slice(out_slice_prev);
				vec_ff_prev.clone_from_slice(ff_slice_prev);
			}

			if !bypass_act {
				act(&mut world, worm.uid, &mut vec1, motor_state.cur_turn());
			}
			if !bypass_sense {
				cortex.write_vec(0, "thal", &mut vec1);
				cortex.sense_vec(0, "motor", &mut motor_state.cur_sdr());
			}
			//let sr_start = (512 << cmn::SYNAPSES_PER_CELL_PROXIMAL_LOG2) as usize;

			if !view_sdr_only {
				print!("\n\n=== Iteration {}/{} ===", i + 1, test_iters);

				if true {
					print!("\nSENSORY INPUT VECTOR:");
					cmn::print_vec(&vec1, 1 , None, None, false);
				}

				print_sense_and_print(&mut cortex);
			}

			// REQUIRES cortex.region_cells.axns.states TO BE FILLED BY .print() unless:

			if view_sdr_only { cortex.region_cells.cols.states.read(); }

			cortex.region_cells.axns.states.read();

			let out_slice = &cortex.region_cells.axns.states.vec[out_start..(out_end + 1)];
			let ff_slice = &cortex.region_cells.cols.states.vec[..];

			//print!("\n****** out_slice.len(): {} ***********", out_slice.len());
			//print!("\n****** vec_out_prev.len(): {} ***********", vec_out_prev.len());
			//println!("\n****** vec_out_prev.clone_from_slice(out_slice): {} ***********", vec_out_prev.clone_from_slice(out_slice));

			//cmn::render_sdr(&vec_out_prev[..], Some(&vec_ff_prev[..]), None, None, &cortex.region_cells.row_map);

			cmn::render_sdr(out_slice, Some(ff_slice), Some(&vec_out_prev[..]), Some(&vec_ff_prev[..]), &cortex.region_cells.row_map, true);

			if view_all_axons {
				print!("\n\nAXON SPACE:\n");
				cmn::render_sdr(&cortex.region_cells.axns.states.vec[128..axn_space_len - 128], None, None, None, &cortex.region_cells.row_map, true);
			}

			i += 1;
			turn_bomb_i += 1;
		}

		if !bypass_act {
			cur_ttl_iters += i;
		}
	}

	println!("");

	cortex.release_components();
	true
}




/* PRINT_SENSE_ONLY() & PRINT_SENSE_AND_PRINT():
	- TODO:
		- [incomplete][priority: low] Roll up into integrated command line system and make each item togglable
*/
fn print_sense_only(cortex: &mut Cortex) {
	if false {
		print!("\nAXON STATES: ");
		cortex.region_cells.axns.states.print_val_range(1 << 8, Some((1, 255)));
	}

	if false {
		print!("\nAXON REGION OUTPUT:");
		cortex.region_cells.axns.states.print((1 << 0) as usize, Some((1, 255)), Some(cortex.region_cells.cols.axn_output_range()), true);
	}
	if false {
		print!("\nCOLUMN SYNAPSE STRENGTHS:");
		cortex.region_cells.cols.syns.strengths.print(1 << 0, None, Some((256, 288)), true);
	}
	if false{	
		print!("\nCOLUMN SYNAPSE SOURCE COLUMN OFFSETS:");
		cortex.region_cells.cols.syns.src_col_x_offs.print(1 << 0, None, Some((256, 288)), true);
	}

	if false {
		print!("\nPYRAMIDAL DENDRITE SYNAPSE STRENGTHS:");
		cortex.region_cells.pyrs.dens.syns.strengths.print(1 << 0, None, Some((256, 319)), true);
	}
}


fn print_sense_and_print(cortex: &mut Cortex) {

	/* COLUMN, COLUMN SYNAPSE, COLUMN RAW STATES */
	if true {	
		print!("\nCOLUMN STATES: ");
		cortex.region_cells.cols.states.print_val_range(1 << 0, Some((1, 255)));
	}
	if false {	
		print!("\nCOLUMN STATES RAW: ");
		cortex.region_cells.cols.states_raw.print_val_range(1 << 0, Some((1, 255)));
	}
	if true {	
		print!("\nCOLUMN SYNAPSE STATES: ");
		cortex.region_cells.cols.syns.states.print(1 << 10, Some((1, 255)), None, true);
	}

		/*if true {	
			print!("\nCOLUMN SYNAPSE STATES: ");
			cortex.region_cells.cols.syns.states.print(1 << 3, Some((1, 255)), None, true);
		}*/

	if true {
		print!("\nCOLUMN SYNAPSE SOURCE ROW IDS:");
		cortex.region_cells.cols.syns.src_row_ids.print(1 << 11, None, None, true);
	}
		if false {
			print!("\nCOLUMN SYNAPSE SOURCE ROW IDS(0 - 1300):");
			cortex.region_cells.cols.syns.src_row_ids.print(1 << 0, None, Some((0, 1300)), true);
		}
	if true {	
		print!("\nCOLUMN SYNAPSE SOURCE COLUMN OFFSETS: ");
		cortex.region_cells.cols.syns.src_col_x_offs.print(1 << 11, None, None, true);
	}
	if true {
		print!("\nCOLUMN SYNAPSE STRENGTHS:");
		cortex.region_cells.cols.syns.strengths.print(1 << 11, None, None, true);
	}
	if false {
		print!("\nCOLUMN PEAK COL IDS: ");
		cortex.region_cells.cols.peak_spis.spi_ids.print_val_range(1 << 0, Some((0, 255)));
	}
	if false {
		print!("\nCOLUMN PEAK COL STATES: ");
		cortex.region_cells.cols.peak_spis.states.print_val_range(1 << 0, Some((1, 255)));
	}



	/* PYRAMIDAL */
	if true {
		print!("\nPYRAMIDAL DEPOLARIZATIONS:");
		cortex.region_cells.pyrs.depols.print_val_range(1 << 8, Some((1, 255)));
	}
	if false {
		print!("\nPYRAMIDAL AXON OUTPUT:");
		cortex.region_cells.axns.states.print((1 << 0) as usize, Some((1, 255)), Some(cortex.region_cells.pyrs.axn_output_range()), false);
	}
	if true {	
		print!("\nPYRAMIDAL DENDRITE STATES: ");
		cortex.region_cells.pyrs.dens.states.print_val_range(1 << 10, Some((1, 255)));
	}
	if false {	
		print!("\nPYRAMIDAL DENDRITE STATES RAW: ");
		cortex.region_cells.pyrs.dens.states_raw.print_val_range(1 << 12, Some((1, 255)));
	}
	if true {	
		print!("\nPYRAMIDAL SYNAPSE STATES: ");
		cortex.region_cells.pyrs.dens.syns.states.print(1 << 16, Some((1, 255)), None, true);
	}	

		if false {	
			print!("\nPYRAMIDAL SYNAPSE STATES (all): ");
			cortex.region_cells.pyrs.dens.syns.states.print(1 << 0, Some((0, 255)), None, true);
			//print!("\nPYRAMIDAL SYNAPSE STATES (524288 - 524588): ");
			//cortex.region_cells.pyrs.dens.syns.states.print(1 << 1, Some((0, 255)), Some((524288, 524588)), true);
		}

	if true {	
		print!("\nPYRAMIDAL SYNAPSE SOURCE ROW IDS: ");
		cortex.region_cells.pyrs.dens.syns.src_row_ids.print(1 << 14, None, None, true);
	}

		if false {
			print!("\nPYRAMIDAL SYNAPSE SOURCE ROW IDS(0 - 1300):");
			cortex.region_cells.pyrs.dens.syns.src_row_ids.print(1 << 1, None, Some((0, 1300)), true);
		}

	if true {	
		print!("\nPYRAMIDAL SYNAPSE SOURCE COLUMN OFFSETS: ");
		cortex.region_cells.pyrs.dens.syns.src_col_x_offs.print(1 << 14, None, None, true);
	}
	if true {
		print!("\nPYRAMIDAL SYNAPSE STRENGTHS:");
		cortex.region_cells.pyrs.dens.syns.strengths.print(1 << 14, None, None, true);
	}



	/* AUX (DEBUG) */
	if true {
		print!("\naux.ints_0: ");
		//cortex.region_cells.aux.ints_0.print((1 << 12) as usize, Some((0, 17000)), None, false);
		cortex.region_cells.aux.ints_0.print((1 << 0) as usize, Some((0, 1023)), Some((1, 19783029)), false);
		print!("\naux.ints_1: ");
		cortex.region_cells.aux.ints_1.print((1 << 0) as usize, None, None, false);
	}
	if false {
		print!("\naux.chars_0: ");
		cortex.region_cells.aux.chars_0.print((1 << 0) as usize, Some((-128, 127)), None, true);
		print!("\naux.chars_1: ");
		cortex.region_cells.aux.chars_1.print((1 << 0) as usize, Some((-128, 127)), None, true);
	}



	/* AXON STATES (ALL) */
	if false {
		print!("\nAXON STATES: ");
		cortex.region_cells.axns.states.print((1 << 4) as usize, Some((1, 255)), None, true);

	}



	/* AXON REGION OUTPUT (L3) */
	if false {
		print!("\nAXON REGION OUTPUT (L3):");
		//cortex.region_cells.axns.states.print((1 << 0) as usize, Some((1, 255)), Some((3000, 4423)));
		cortex.region_cells.axns.states.print(
			(1 << 0) as usize, Some((0, 255)), 
			Some(cortex.region_cells.cols.axn_output_range()), 
			false
		);
	}

	print!("\n");

}


fn act(world: &mut World, ent_uid: usize, vec: &mut Vec<u8>, turn_left: bool) {
	world.entities().get_mut(ent_uid).turn((WORLD_TURN_FACTOR/cmn::SENSORY_CHORD_WIDTH as f32), turn_left);
	world.peek_from(ent_uid).unfold_into(vec, 0);
}


fn rin(prompt: String) -> String {
	let mut in_string: String = String::new();
	print!("\n{}:> ", prompt);
	io::stdout().flush().unwrap();
	io::stdin().read_line(&mut in_string).ok().expect("Failed to read line");
	in_string
}


fn test_vec_init(cortex: &mut Cortex) -> Vec<ocl::cl_uchar> {

	//let vv1 = cmn::sparse_vec(2048, -128i8, 127i8, 6);
	//cmn::print_vec(&vv1, 1, false, Some(ops::Range{ start: -127, end: 127 }));

	//let mut vec1: Vec<i8> = cmn::shuffled_vec(1024, 0, 127);
	//let mut vec1: Vec<i8> = cmn::sparse_vec(2048, -128i8, 127i8, 8);

	//cmn::print_vec(&vec1, 1, false, Some(ops::Range{ start: -128, end: 127 }));
	let scw = cmn::SENSORY_CHORD_WIDTH;

	//print!("\n*********** scl_fct: {}", scl_fct);
	//print!("\n*********** cmn::log2(sct_fct): {}", cmn::log2(scl_fct));

	let mut vec1: Vec<ocl::cl_uchar> = Vec::with_capacity(scw as usize);

	//let mut vec1: Vec<ocl::cl_uchar> = iter::repeat(0).take(sc_width as usize).collect();
	/*for i in range(0, scw) {
		if i < scw >> 1 {
			vec1.push(64i8);
		} else {
			vec1.push(0i8);
		}
	}*/

	/* MAKE THIS A STRUCT OR SOMETHING */
	let scw_1_2 = scw >> 1;

	let scw_1_4 = scw >> 2;
	let scw_3_4 = scw - scw_1_4;

	let scw_1_8 = scw >> 3;
	let scw_3_8 = scw_1_2 - scw_1_8;
	let scw_5_8 = scw_1_2 + scw_1_8;

	let scw_1_16 = scw >> 4;

	//println!("***** scw_1_4: {}, scw_3_4: {}", scw_1_4, scw_3_4);
	/*for i in 0..scw {
		if i >= scw_3_8 + scw_1_16 && i < scw_5_8 - scw_1_16 {
		//if i >= scw_3_8 && i < scw_5_8 {
			vec1.push(0);
		} else {
			vec1.push(0);
		}
	}*/


	vec1.clear();
	for i in 0..scw {
		if i >= scw_1_2 - (scw_1_16 / 2) && i < scw_1_2 + (scw_1_16 / 2) {
		//if ((i >= scw_1_4 - scw_1_16) && (i < scw_1_4 + scw_1_16)) || ((i >= scw_3_4 - scw_1_16) && (i < scw_3_4 + scw_1_16)) {
		//if i >= scw_3_8 && i < scw_5_8 {
		//if (i >= scw_1_2 - scw_1_16 && i < scw_1_2 + scw_1_16) || (i < scw_1_16) || (i >= (scw - scw_1_16)) {
		//if i >= scw_3_8 && i < scw_5_8 {
		//if i < scw_1_16 {
		//if i < scw_1_16 || i >= (scw - scw_1_16) {
			vec1.push(1);
		} else {
			vec1.push(0);
		}
	}


	vec1

	/*if SHUFFLE_ONCE {
		cmn::shuffle_vec(&mut vec1);
		//chord1 = Chord::from_vec(&vec1);
	}*/

}

