use std::default::{ Default };
use std::iter;
use std::fmt::{ Display };
use std::ops::{ Range };
use std::io::{ self, Write, Stdout };
//use std::borrow::{ Borrow };
use rand::{ self, ThreadRng, Rng };
use num::{ self, Integer, NumCast, FromPrimitive, ToPrimitive };
use time;

use ocl::{ self, CorticalDimensions };
use cmn;
use cortex::{ self, Cortex };
use encode:: { IdxReader };
//use proto::layer;
use super::output_czar;
//use super::synapse_drill_down;
use super::input_czar::{ self, InputCzar, InputVecKind };
use super::hybrid;
use super::renderer::{ Renderer };
//use chord::{ Chord };
//use ocl::{ Envoy };
use proto::{ Protoregion, Protoregions, Protoareas, ProtoareasTrait, Protoarea, Cellular, Axonal, Spatial, Horizontal, Sensory, layer, Protocell };


pub const INITIAL_TEST_ITERATIONS: i32 		= 1; 
pub const STATUS_EVERY: i32 				= 5000;
pub const PRINT_DETAILS_EVERY: i32			= 10000;

pub const TOGGLE_DIRS: bool 				= false;
pub const INTRODUCE_NOISE: bool 			= false;
pub const COUNTER_RANGE: Range<usize>		= Range { start: 0, end: 8 };
pub const COUNTER_RANDOM: bool				= false;


/* Eventually move defines to a config file or some such */
pub fn define_protoregions() -> Protoregions {
	let mut cort_regs: Protoregions = Protoregions::new();

	let mut sen = Protoregion::new(Sensory)
		//.layer("test_noise", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("motor_in", 1, layer::DEFAULT, Axonal(Horizontal))
		.layer("eff_in", 1, layer::EFFERENT_INPUT, Axonal(Spatial))
		//.layer("nothing", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("aff_in", 1, layer::AFFERENT_INPUT, Axonal(Spatial))
		.layer("aff_out", 1, layer::AFFERENT_OUTPUT | layer::EFFERENT_OUTPUT, Axonal(Spatial))

		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["aff_in"], 384)) 

		.layer("iv_inhib", 0, layer::DEFAULT, 
			Protocell::new_inhibitory(4, "iv"))

		.layer("iii", 1, layer::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(2, 5, vec!["iii"], 900).apical(vec!["eff_in"]))

		.freeze()
	;

	cort_regs.add(sen);
	cort_regs
}

pub fn define_protoareas() -> Protoareas {
	let area_side = 48 as u32;

	let mut protoareas = Protoareas::new()
		.area("v1", area_side, area_side, Sensory, 
			None
			//Some(vec!["b1"])
		)

		// .area("b1", area_side, area_side, Sensory, 
		//  	//None
		//  	Some(vec!["a1"])
		// )

		// .area("a1", area_side, area_side, Sensory, None)
	;

	protoareas
}


/* RUN(): Run the interactive testing command line
	- TODO:
		- [incomplete][priority:very low] Proper command line using enums to 
		represent user input and a seperate struct to manage its state
			- Or just be lazy and leave it the beautiful disaster that it is...	
*/
pub fn run(autorun_iters: i32) -> bool {
	let mut cortex = cortex::Cortex::new(define_protoregions(), define_protoareas());
	let mut area_name = "v1".to_string();
	let inhib_layer_name = "iv_inhib";
	let area_dims = cortex.area(&area_name).dims().clone();

	//let input_kind = InputVecKind::Stripes { stripe_size: 512, zeros_first: true };
	//let input_kind = InputVecKind::Hexballs { edge_size: 9, invert: false, fill: false };
	//let input_kind = InputVecKind::World;
	//let input_kind = InputVecKind::Exp1;

	let mut ir = IdxReader::new(area_dims.clone(), "data/train-images-idx3-ubyte");
	let input_kind = InputVecKind::IdxReader(Box::new(ir));

	/* ***** DISABLE STUFF ***** */	
	cortex.area_mut(&area_name).psal_mut().dens_mut().syns.set_offs_to_zero();
	cortex.area_mut(&area_name).bypass_inhib = true;
	cortex.area_mut(&area_name).disable_pyrs = true;
	//cortex.area_mut(&area_name).disable_regrowth = true;
	/* ***** ############# ***** */

	let mut input_czar = InputCzar::new(area_dims.clone(), input_kind, COUNTER_RANGE, COUNTER_RANDOM, TOGGLE_DIRS, INTRODUCE_NOISE);

	let mut rndr = Renderer::new(cortex.area(&area_name).dims().clone());

	//let mut vec_out_prev: Vec<u8> = iter::repeat(0).take(area_dims.columns() as usize).collect();
	//let mut vec_ff_prev: Vec<u8> = iter::repeat(0).take(area_dims.columns() as usize).collect();

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
				let axn_state = if view_all_axons { "on" } else { "off" };
				let view_state = if view_sdr_only { "sdr" } else { "all" };

				rin(format!("bismit: [{ttl_i}/({loop_i})]: [v]iew:[{}] [a]xons:[{}] \
					[m]otor:[{}] a[r]ea:[{}] [t]ests [q]uit [i]ters:[{iters}]", 
					view_state, axn_state, input_czar.motor_state.cur_str(), area_name, 
					iters = test_iters,
					loop_i = input_czar.counter(), 
					ttl_i = cur_ttl_iters,
				))
			};


			if "q\n" == in_string {
				print!("Quitting... ");
				break;
			} else if "i\n" == in_string {
				let in_s = rin(format!("Iterations: [i={}]", test_iters));
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
							print!("\nInvalid number.\n");
							continue;
						},
					}
				}

			} else if "r\n" == in_string {
				let in_str = rin(format!("area name"));
				let in_s1 = in_str.trim();
				let new_area_name = in_s1.to_string();
				
				if cortex.valid_area(&new_area_name) {
					area_name = new_area_name;
				} else {
					print!("\nInvalid area.");
				}
				continue;

			} else if "v\n" == in_string {
				view_sdr_only = !view_sdr_only;
				bypass_act = true;

			} else if "\n" == in_string {
				// DO NOT REMOVE

			} else if "a\n" == in_string {
				view_all_axons = !view_all_axons;
				bypass_act = true;

			} else if "t\n" == in_string {
				bypass_act = true;
				let in_s = rin(format!("tests: [f]ract [c]ycles [l]earning [a]ctivate a[r]ea_output o[u]tput"));

				if "p\n" == in_s {
					//synapse_drill_down::print_pyrs(&mut cortex);
					continue;

				} else if "u\n" == in_s {
					let in_str = rin(format!("area name"));
					let in_s1 = in_str.trim();
					let out_len = cortex.area(&in_s).dims.columns();
					let mut t_vec: Vec<u8> = iter::repeat(0).take(out_len as usize).collect();
					cortex.area_mut(&in_s).read_output(&mut t_vec, layer::AFFERENT_OUTPUT);
					cmn::print_vec_simple(&t_vec);
					continue;

				} else if "c\n" == in_s {
					hybrid::test_cycles(&mut cortex, &area_name);
					continue;

				} else if "l\n" == in_s {
					hybrid::test_learning(&mut cortex, inhib_layer_name, &area_name);
					continue;

				} else if "a\n" == in_s {
					hybrid::test_activation_and_learning(&mut cortex, &area_name);
					continue;

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
					cmn::print_vec_simple(&tvec[..]);
					continue;

				} else if "r\n" == in_s {
					let in_str = rin(format!("area name"));
					let in_s = in_str.trim();
					//let in_int: Option<u8> = in_s.trim().parse().ok();

					cortex.print_area_output(&in_s);
					continue;

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



		// CURRENT ACTIVE ITERATION
		let mut i = 0i32;

		/*######### SENSE ONLY LOOP #########*/
		if !view_sdr_only { print!("\nRunning {} sense only loop(s) ... \n", test_iters - 1); }

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
					output_czar::print_sense_only(&mut cortex, &area_name); 
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

			if !bypass_act {
				input_czar.next(&mut cortex);
			}

			//let sr_start = (512 << cmn::SYNAPSES_PER_CELL_PROXIMAL_LOG2) as usize;

			if !view_sdr_only {
				print!("\n\n=== Iteration {}/{} ===", i + 1, test_iters);

				if true {
					print!("\nSENSORY INPUT VECTOR:");
					cmn::print_vec(&input_czar.vec_optical[..], 1 , None, None, false);
				}

				output_czar::print_sense_and_print(&mut cortex, &area_name);
			}

			// REQUIRES cortex.area(&area_name).axns.states TO BE FILLED BY .print() unless:

			if view_sdr_only { cortex.area_mut(&area_name).psal_mut().dens.states.read(); }

			cortex.area_mut(&area_name).axns.states.read();

			let (eff_out_idz, eff_out_idn) = cortex.area(&area_name).mcols.axn_output_range();
			let (ssts_axn_idz, ssts_axn_idn) = cortex.area_mut(&area_name).psal_mut().axn_range();

			let out_slc = &cortex.area(&area_name).axns.states.vec[eff_out_idz..eff_out_idn];
			let ff_slc = &cortex.area(&area_name).axns.states.vec[ssts_axn_idz..ssts_axn_idn];
			//let ff_slc = &cortex.area(&area_name).psal_mut().dens.states.vec[..];

			print!("\n'{}' output:", &area_name);

			rndr.render(out_slc, ff_slc);


			//cmn::render_sdr(out_slc, Some(ff_slc), Some(&vec_out_prev[..]), Some(&vec_ff_prev[..]), &cortex.area(&area_name).protoregion().slc_map(), true, cortex.area(&area_name).dims.columns());

			if view_all_axons {
				print!("\n\nAXON SPACE:\n");
				let axn_space_len = cortex.area(&area_name).axns.states.vec.len();

				//cmn::render_sdr(&cortex.area(&area_name).axns.states.vec[128..axn_space_len - 128], None, None, None, &cortex.area(&area_name).protoregion().slc_map(), true, cortex.area(&area_name).dims.columns());
			}

			i += 1;
		}

		if test_iters > 1000 {
			test_iters = 1;
		}

		if !bypass_act {
			cur_ttl_iters += i;
		}

		if autorun_iters > 0 {
			break;
		}
	}

	println!("");

	true
}


struct Prompt {
	prompt_pre: &'static str,
	prompt_post: &'static str,
}


struct Command {
	text: &'static str,
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
	//in_s.trim().replace("m","000000").parse().ok()
}


