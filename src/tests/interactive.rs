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
use super::input_czar::{ self, InputCzar, InputKind, InputSource };
use super::hybrid;
use super::renderer::{ Renderer };
//use chord::{ Chord };
//use ocl::{ Envoy };
use proto::{ Protoregion, Protoregions, Protoareas, Protoarea, Cellular, Axonal, Spatial, Horizontal, Sensory, Thalamic, layer, Protocell, Protofilter, Protoinput };


pub const INITIAL_TEST_ITERATIONS: i32 		= 1; 
pub const STATUS_EVERY: i32 				= 5000;
pub const PRINT_DETAILS_EVERY: i32			= 10000;

pub const TOGGLE_DIRS: bool 				= false;
pub const INTRODUCE_NOISE: bool 			= false;
pub const COUNTER_RANGE: Range<usize>		= Range { start: 0, end: 5000 };
pub const COUNTER_RANDOM: bool				= false;
const REPEATS_PER_IMAGE: usize 				= 4;


/* Eventually move defines to a config file or some such */
pub fn define_protoregions() -> Protoregions {
	let mut cort_regs: Protoregions = Protoregions::new();

	cort_regs.add(Protoregion::new("visual", Sensory)
		//.layer("test_noise", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("motor_in", 1, layer::DEFAULT, Axonal(Horizontal))
		//.layer("olfac", 1, layer::DEFAULT, Axonal(Horizontal))
		.layer("eff_in", 0, layer::EFFERENT_INPUT, Axonal(Spatial))
		//.layer("nothing", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("aff_in", 0, layer::AFFERENT_INPUT, Axonal(Spatial))
		.layer("out", 1, layer::AFFERENT_OUTPUT | layer::EFFERENT_OUTPUT, Axonal(Spatial))
		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["aff_in"], 600)) 
		.layer("iv_inhib", 0, layer::DEFAULT, 
			Protocell::new_inhibitory(4, "iv"))
		.layer("iii", 1, layer::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(0, 5, vec!["iii"], 1200).apical(vec!["eff_in"]))
	);

	cort_regs.add(Protoregion::new("external", Thalamic)
		.layer("ganglion", 1, layer::AFFERENT_OUTPUT | layer::AFFERENT_INPUT, Axonal(Spatial))
	);

	cort_regs
}

pub fn define_protoareas() -> Protoareas {
	let area_side = 48 as u32;

	let mut protoareas = Protoareas::new()
		
		//let mut ir_labels = IdxReader::new(CorticalDimensions::new(1, 1, 1, 0, None), "data/train-labels-idx1-ubyte", 1);
		.area_ext("u0", "external", area_side, area_side, 
			Protoinput::IdxReader { 
				file_name: "data/train-labels-idx1-ubyte", 
				repeats: REPEATS_PER_IMAGE,
			},

			None, 
			Some(vec!["u1"]),
		)

		.area("u1", "visual", area_side, area_side, None,
			//None,
			Some(vec!["b1"]),
		)
		

		.area_ext("v0", "external", area_side, area_side, 
			Protoinput::IdxReader { 
				file_name: "data/train-images-idx3-ubyte", 
				repeats: REPEATS_PER_IMAGE,
			},

			None, 
			Some(vec!["v1"]),
		)

		.area("v1", "visual", area_side, area_side, 
			Some(vec![Protofilter::new("retina", Some("filters.cl"))]),
			//None,
			Some(vec!["b1"]),
		)

		.area("b1", "visual", area_side, area_side,
		 	None,
		 	Some(vec!["a1"]),
		)

		.area("a1", "visual", area_side, area_side, None, None)
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
	// KEEP UNTIL WE FIGURE OUT HOW TO GENERATE THIS NUMBER ELSEWHERE
	let mut ir_labels = IdxReader::new(CorticalDimensions::new(1, 1, 1, 0, None), "data/train-labels-idx1-ubyte", 1);
	
	let mut cortex = cortex::Cortex::new(define_protoregions(), define_protoareas());
	let mut area_name = "v1".to_string();
	let inhib_layer_name = "iv_inhib";
	let area_dims = cortex.area(&area_name).dims().clone();

	/* ***** DISABLE STUFF ***** */	
	for (area_name, area) in &mut cortex.areas {
		//area.psal_mut().dens_mut().syns.set_offs_to_zero();
		//area.bypass_inhib = true;
		//area.disable_pyrs = true;
		//area.disable_regrowth = true;
		//area.disable_learning = true;
	}
	/* ************************* */

	//let input_kind = InputKind::Stripes { stripe_size: 512, zeros_first: true };
	//let input_kind = InputKind::Hexballs { edge_size: 9, invert: false, fill: false };
	//let input_kind = InputKind::World;
	//let input_kind = InputKind::Exp1;		
	

	// let mut ir = IdxReader::new(area_dims.clone(), "data/train-images-idx3-ubyte", REPEATS_PER_IMAGE);
	// let input_sources: Vec<InputSource> = vec![
	// 	InputSource::new(InputKind::IdxReader(Box::new(ir)), "v1"),
	// ];
	// let mut input_czar = InputCzar::new(area_dims.clone(), input_sources, COUNTER_RANGE, 
	// 	COUNTER_RANDOM, TOGGLE_DIRS, INTRODUCE_NOISE);

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
	let mut input_status: String = String::with_capacity(100);

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
					[m]otor:[X] a[r]ea:[{}] [t]ests [q]uit [i]ters:[{iters}]", 
					view_state, axn_state, /*input_czar.motor_state.cur_str(),*/ area_name, 
					iters = test_iters,
					loop_i = 0, //input_czar.counter(), 
					ttl_i = cur_ttl_iters,
				))
			};


			if "q\n" == in_string {
				print!("\nExiting interactive test mode... ");
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
				//continue;
				bypass_act = true;

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
				let in_s = rin(format!("motor: [s]witch(disconnected)"));
				if "s\n" == in_s {
					//input_czar.motor_state.switch();
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
				//input_czar.next(&mut cortex);
				cortex.cycle();
			}


			i += 1;
		}



		/*######### SENSE AND PRINT LOOP #########*/
		if !view_sdr_only { print!("\n\nRunning {} sense and print loop(s)...", 1usize); }

		loop {
			if i >= (test_iters) { break; }

			if !bypass_act {
				//input_czar.next(&mut cortex); // Just increments counter
				cortex.cycle();
				input_status.clear();
				let cur_frame = cur_ttl_iters as usize % 5000;
				input_status.push_str(&format!("[{}] -> '{}'", cur_frame, ir_labels.get_first_byte(cur_frame)));
			}

			//let sr_start = (512 << cmn::SYNAPSES_PER_CELL_PROXIMAL_LOG2) as usize;

			if !view_sdr_only {
				print!("\n\n=== Iteration {}/{} ===", i + 1, test_iters);

				if false {
					print!("\nSENSORY INPUT VECTOR:");
					//cmn::print_vec(&input_czar.vec_optical[..], 1 , None, None, false);
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

			rndr.render(out_slc, ff_slc, &input_status, true);

			if view_all_axons {
				print!("\n\nAXON SPACE:\n");
				
				let axn_space_len = cortex.area(&area_name).axns.states.vec.len();

				rndr.render_axon_space(&cortex.area(&area_name).axns.states.vec[..], 
					&cortex.area(&area_name).protoregion().slc_map(),
					cortex.area(&area_name).dims.columns(),
					cortex.area(&area_name).protoregion().hrz_demarc(),
				);				

				// cmn::render_sdr(&cortex.area(&area_name).axns.states.vec[128..axn_space_len - 128], None, None, None, &cortex.area(&area_name).protoregion().slc_map(), true, cortex.area(&area_name).dims.columns());
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


