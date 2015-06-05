use std::default::{ Default };
use std::iter;
use std::fmt::{ Display };
use std::ops::{ Range };
use std::io::{ self, Write, Stdout };
//use std::borrow::{ Borrow };
use rand::{ self, ThreadRng, Rng };
use num::{ self, Integer, NumCast, FromPrimitive, ToPrimitive };
use time;

use ocl;
use cmn;
use cortex::{ self, Cortex };
use super::output_czar;
use super::synapse_drill_down;
use super::input_czar::{ self, InputCzar, InputVecKind };
use super::hybrid;
//use chord::{ Chord };
//use ocl::{ Envoy };


pub const INITIAL_TEST_ITERATIONS: i32 	= 1; 
pub const STATUS_EVERY: i32 			= 5000;
pub const PRINT_DETAILS_EVERY: i32		= 10000;

pub const TOGGLE_DIRS: bool 				= true;
pub const INTRODUCE_NOISE: bool 			= false;
pub const COUNTER_RANGE: Range<usize>		= Range { start: 0, end: 10 };
pub const COUNTER_RANDOM: bool				= false;


/* RUN(): Run the interactive testing command line
	- TODO:
		- [incomplete][priority:very low] Proper command line using enums to represent user input and a seperate struct to manage its state
			- Or just be lazy and leave it the beautiful disaster that it is...	
*/
pub fn run(autorun_iters: i32) -> bool {
	let mut cortex = cortex::Cortex::new(cortex::define_protoregions(), cortex::define_protoareas());
	let sc_columns = cortex.cortical_area.dims.columns();
	let mut input_czar = InputCzar::new(sc_columns, InputVecKind::World, COUNTER_RANGE, COUNTER_RANDOM, TOGGLE_DIRS, INTRODUCE_NOISE);

	let mut vec_out_prev: Vec<u8> = iter::repeat(0).take(sc_columns as usize).collect();
	let mut vec_ff_prev: Vec<u8> = iter::repeat(0).take(sc_columns as usize).collect();

	let mut test_iters: i32 = if autorun_iters > 0 {
		autorun_iters
	} else {
		INITIAL_TEST_ITERATIONS
	};

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

		if autorun_iters == 0 {
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
				let in_s = rin(format!("iterations: [i={}]", test_iters));
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
				let in_s = rin(format!("tests: [p]yrs [m]cols [f]ract [c]ycles"));

				if "p\n" == in_s {
					synapse_drill_down::print_pyrs(&mut cortex);
					//println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
					continue;
					//test_iters = TEST_ITERATIONS;
				} else if "m\n" == in_s {
					synapse_drill_down::print_mcols(&mut cortex);
					//println!("\nREPLACE ME - synapse_sources::run() - line 100ish");
					continue;
					//test_iters = TEST_ITERATIONS;
				} else if "c\n" == in_s {
					hybrid::test_cycles(&mut cortex);
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

				} else {
					continue;
				}
			} else {
				continue;
			}
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

			if i % STATUS_EVERY == 0 || i < 0 || i == (test_iters - 2) {
				let t = time::get_time() - time_start;
				if i > 0 || (test_iters > 1 && i == 0) {
					print!("[{}: {:02.4}ms]", i, t.num_milliseconds());
				}
				io::stdout().flush().ok();
			}

			if i % PRINT_DETAILS_EVERY == 0 || i < 0 {
				if !view_sdr_only { 
					output_czar::print_sense_only(&mut cortex); 
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

			let (out_start, out_end) = cortex.cortical_area.mcols.axn_output_range();
			let axn_space_len = cortex.cortical_area.axns.states.vec.len();
			//println!("\n##### out_start: {}, out_end: {}, axn_space_len: {}", out_start, out_end, axn_space_len);

			{
				let out_slice_prev = &cortex.cortical_area.axns.states.vec[out_start..(out_end + 1)];
				let ff_slice_prev = &cortex.cortical_area.ssts.get_mut("iv").expect("interactive.rs").dens.states.vec[..];

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

				output_czar::print_sense_and_print(&mut cortex);
			}

			// REQUIRES cortex.cortical_area.axns.states TO BE FILLED BY .print() unless:

			if view_sdr_only { cortex.cortical_area.ssts.get_mut("iv").expect("interactive.rs").dens.states.read(); }

			cortex.cortical_area.axns.states.read();

			let out_slice = &cortex.cortical_area.axns.states.vec[out_start..(out_end + 1)];
			let ff_slice = &cortex.cortical_area.ssts.get_mut("iv").expect("interactive.rs").dens.states.vec[..];

			cmn::render_sdr(out_slice, Some(ff_slice), Some(&vec_out_prev[..]), Some(&vec_ff_prev[..]), &cortex.cortical_area.slice_map, true, cortex.cortical_area.dims.columns());

			if view_all_axons {
				print!("\n\nAXON SPACE:\n");
				cmn::render_sdr(&cortex.cortical_area.axns.states.vec[128..axn_space_len - 128], None, None, None, &cortex.cortical_area.slice_map, true, cortex.cortical_area.dims.columns());
			}

			i += 1;
		}

		if !bypass_act {
			cur_ttl_iters += i;
		}

		if autorun_iters > 0 {
			break;
		}
	}

	println!("");

	cortex.release_components();
	true
}


struct Prompt {
	prompt_pre: &'static str,
	prompt_post: &'static str,
}


struct Command {
	text: &'static str,
}

/*struct Parameter {

}

struct Section {

}*/





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


