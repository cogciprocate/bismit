
use super::synapse_drill_down;
use super::input_czar::{ self, InputCzar };
//use super::motor_state;
use cortex::{ self, Cortex };
use ocl;
use cmn;
use chord::{ Chord };
use ocl::{ Envoy };
//use microcosm::entity::{ EntityBody, EntityKind, EntityBrain, Mobile };
//use microcosm::worm::{ WormBrain };
//use microcosm::common::{ Location, Peek, Scent, WORM_SPEED, TAU };
//use microcosm::world::{ World };


use std::default::{ Default };
use std::iter;
use std::fmt::{ Display };
use std::ops::{ Range };
use std::io::{ self, Write, Stdout };
use std::borrow::{ Borrow };
use rand::{ self, ThreadRng, Rng };
use num::{ self, Integer, NumCast, FromPrimitive, ToPrimitive };
use time;

pub const INITIAL_TEST_ITERATIONS: i32 	= 1; 
pub const STATUS_EVERY: i32 			= 5000;
pub const PRINT_DETAILS_EVERY: i32		= 10000;
pub const SHUFFLE_ONCE: bool 			= true;
pub const SHUFFLE_EVERY: bool 			= false;


/* RUN(): Run the interactive testing command line
	- TODO:
		- [incomplete][priority:very low] Proper command line using enums to represent user input and a seperate struct to manage its state
			- Or just be lazy and leave it the beautiful disaster that it is...	
*/
pub fn run() -> bool {
	let sc_width = cmn::SENSORY_CHORD_WIDTH;
	let mut cortex = cortex::Cortex::new();
	//let mut world: World = World::new(sc_width);

	let mut input_czar = InputCzar::new(0..5, false);

	//let mut vec1: Vec<u8> = iter::repeat(0).take(sc_width as usize).collect();
	//let mut vec1: Vec<ocl::cl_uchar> = input_czar::test_vec_init(&mut cortex);

	let mut vec_out_prev: Vec<u8> = iter::repeat(0).take(sc_width as usize).collect();
	let mut vec_ff_prev: Vec<u8> = iter::repeat(0).take(sc_width as usize).collect();

	//let mut vec2: Vec<ocl::cl_uchar> = iter::repeat(0).take(sc_width as usize).collect();
	//cortex.write_vec(0, "pre_thal", &mut vec2);
	//cortex.write_vec(0, "post_thal", &mut vec2);
	//cortex.write_vec(0, "post_thal2", &mut vec2);
	//cortex.write_vec(0, "post_thal3", &mut vec2);
	//cortex.write_vec(0, "post_thal4", &mut vec2);
	//cortex.write_vec(0, "post_thal5", &mut vec2);


	//world.entities().print();

	//let mut motor_state = motor_state::MotorState::new();

	//let mut rng: rand::XorShiftRng = rand::weak_rng();
	//let mut turn_bomb_i = 0usize;
	//let mut turn_bomb_n = 6;
	//let mut turn_bomb_n = rng.gen::<u8>() as usize;
	
	let mut test_iters: i32 = INITIAL_TEST_ITERATIONS;
	let mut first_run: bool = true;
	let mut bypass_act = false;

	let mut view_all_axons: bool = false;
	let mut view_sdr_only: bool = true;
	let mut cur_ttl_iters: i32 = 0;

	loop {
		/*######### COMMAND LINE #########*/
		let vso = if view_sdr_only { "sdr" } else { "all" };

		bypass_act = false;

		if test_iters == 0 {
			test_iters = 1;
			bypass_act = true; 
		}

		let mut in_string: String = if first_run {
			first_run = false;
			"\n".to_string()
		} else {
			rin(format!("<{}>bismit: [q]uit [i]ters [v]iew [a]xons [t]ests [m]otor [i={} v={} m={} z={}]", 
				cur_ttl_iters, test_iters, vso, input_czar.motor_state.cur_str(), input_czar.counter()))
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
				let in_int: Option<i32> = parse_num(in_s);
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


		} else if "\n" == in_string {
			// Go


		} else if "a\n" == in_string {
			view_all_axons = !view_all_axons;
			bypass_act = true;


		} else if "t\n" == in_string {
			bypass_act = true;
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
			let in_s = rin(format!("motor: [s]witch"));
			if "s\n" == in_s {
				input_czar.motor_state.switch();
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

		if test_iters > 1 {
			print!("Running {} iterations... \n", test_iters);
		}


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
				input_czar.next(&mut cortex);
			}


			i += 1;
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
				input_czar.next(&mut cortex);
			}

			//let sr_start = (512 << cmn::SYNAPSES_PER_CELL_PROXIMAL_LOG2) as usize;

			if !view_sdr_only {
				print!("\n\n=== Iteration {}/{} ===", i + 1, test_iters);

				if true {
					print!("\nSENSORY INPUT VECTOR:");
					cmn::print_vec(&input_czar.vec_optical, 1 , None, None, false);
				}

				print_sense_and_print(&mut cortex);
			}

			// REQUIRES cortex.region_cells.axns.states TO BE FILLED BY .print() unless:

			if view_sdr_only { cortex.region_cells.cols.states.read(); }

			cortex.region_cells.axns.states.read();

			let out_slice = &cortex.region_cells.axns.states.vec[out_start..(out_end + 1)];
			let ff_slice = &cortex.region_cells.cols.states.vec[..];

			cmn::render_sdr(out_slice, Some(ff_slice), Some(&vec_out_prev[..]), Some(&vec_ff_prev[..]), &cortex.region_cells.row_map, true);

			if view_all_axons {
				print!("\n\nAXON SPACE:\n");
				cmn::render_sdr(&cortex.region_cells.axns.states.vec[128..axn_space_len - 128], None, None, None, &cortex.region_cells.row_map, true);
			}

			i += 1;
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
	if true {	
		print!("\nPYRAMIDAL SYNAPSE SOURCE FLAG SETS: ");
		cortex.region_cells.pyrs.dens.syns.flag_sets.print(1 << 14, None, None, true);
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



fn rin(prompt: String) -> String {
	let mut in_string: String = String::new();
	print!("\n{}:> ", prompt);
	io::stdout().flush().unwrap();
	io::stdin().read_line(&mut in_string).ok().expect("Failed to read line");
	in_string
}

fn parse_num(in_s: String) -> Option<i32> {
	in_s.trim().replace("k","000").parse().ok()
}


