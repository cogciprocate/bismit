use std;
use num::{ self, Integer, Signed, NumCast, ToPrimitive, FromPrimitive };
//use std::num::{ NumCast, ToPrimitive, FromPrimitive };
use std::ops::{ self, BitOr };
use std::default::{ Default }; 
use std::fmt::{ Display, Debug, LowerHex, UpperHex };
use std::iter::{ self };
use std::cmp::{ Ord };
use std::io::{ self, Write, Stdout };
use std::collections::{ BTreeMap };
use rand;
use rand::distributions::{ self, Normal, IndependentSample, Range };

use ocl;




/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
========================= YES, IT'S A MESS IN HERE ============================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/




pub static C_DEFAULT: &'static str = "\x1b[0m";
pub static C_DRD: &'static str = "\x1b[31m";
pub static C_LRD: &'static str = "\x1b[91m";
pub static C_CYA: &'static str = "\x1b[36m";
pub static C_GRN: &'static str = "\x1b[32m";
pub static C_DBL: &'static str = "\x1b[34m";
pub static C_BLU: &'static str = "\x1b[94m";
pub static C_MAG: &'static str = "\x1b[95m";
pub static C_PUR: &'static str = "\x1b[35m";
pub static C_ORA: &'static str = "\x1b[33m";
pub static C_YEL: &'static str = "\x1b[93m";
pub static C_LBL: &'static str = "\x1b[94m";
pub static C_LGR: &'static str = "\x1b[37m";
pub static C_DGR: &'static str = "\x1b[90m";
pub static BGC_DEFAULT: &'static str = "\x1b[49m";
pub static BGC_GRN: &'static str = "\x1b[42m";
pub static BGC_MAG: &'static str = "\x1b[45m";
pub static BGC_DGR: &'static str = "\x1b[100m";

pub static KERNELS_FILE_NAME: &'static str = "bismit.cl";

pub const CORTICAL_SEGMENTS_TOTAL: usize = 1;
pub const SENSORY_SEGMENTS_TOTAL: usize = 1;
pub const MOTOR_SEGMENTS_TOTAL: usize = 1;

pub const HYPERCOLUMNS_PER_SEGMENT: usize = 16;		// appears to cause lots of delay... 256 is slow

pub const SYNAPSE_STRENGTH_INITIAL_DEVIATION: i8 = 5;


//pub const DST_SYNAPSE_STRENGTH_DEFAULT: i8 = 10;
//pub const PRX_SYNAPSE_STRENGTH_DEFAULT: i8 = 10;
pub const DST_SYNAPSE_STRENGTH_DEFAULT: i8 = 0;
pub const PRX_SYNAPSE_STRENGTH_DEFAULT: i8 = 0;

pub const COLUMNS_PER_HYPERCOLUMN: u32 = 64;



pub const DENDRITES_PER_CELL_DISTAL_LOG2: u32 = 3;
pub const DENDRITES_PER_CELL_DISTAL: u32 = 1 << DENDRITES_PER_CELL_DISTAL_LOG2;

pub const SYNAPSES_PER_DENDRITE_DISTAL_LOG2: u32 = 4;
pub const SYNAPSES_PER_DENDRITE_DISTAL: u32 = 1 << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;



pub const DENDRITES_PER_CELL_PROXIMAL_LOG2: u32 = 0;
pub const DENDRITES_PER_CELL_PROXIMAL: u32 = 1 << DENDRITES_PER_CELL_PROXIMAL_LOG2;

pub const SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2: u32 = 6;
pub const SYNAPSES_PER_DENDRITE_PROXIMAL: u32 = 1 << SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;


/* GET RID OF THIS UNLESS CL NEEDS IT */
//pub const SYNAPSES_PER_CELL_PROXIMAL_LOG2: u32 = DENDRITES_PER_CELL_PROXIMAL_LOG2 + SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
//pub const SYNAPSES_PER_CELL_PROXIMAL: u32 = 1 << SYNAPSES_PER_CELL_PROXIMAL_LOG2;



//pub const AXONS_PER_CELL: usize = DENDRITES_PER_CELL * SYNAPSES_PER_DENDRITE;
//pub const SYNAPSES_PER_CELL: usize = SYNAPSES_PER_DENDRITE * DENDRITES_PER_CELL;

pub const COLUMNS_PER_SEGMENT: usize = COLUMNS_PER_HYPERCOLUMN as usize * HYPERCOLUMNS_PER_SEGMENT;
//pub const COLUMN_AXONS_PER_SEGMENT: usize = AXONS_PER_CELL * COLUMNS_PER_SEGMENT;
//pub const COLUMN_DENDRITES_PER_SEGMENT: usize = DENDRITES_PER_CELL * COLUMNS_PER_SEGMENT;
//pub const COLUMN_SYNAPSES_PER_SEGMENT: usize = SYNAPSES_PER_DENDRITE * COLUMN_DENDRITES_PER_SEGMENT;

pub const CELLS_PER_SEGMENT: usize = LAYERS_PER_SEGMENT * COLUMNS_PER_SEGMENT;
//pub const CELL_AXONS_PER_SEGMENT: usize = AXONS_PER_CELL * CELLS_PER_SEGMENT;
//pub const CELL_DENDRITES_PER_SEGMENT: usize = DENDRITES_PER_CELL * CELLS_PER_SEGMENT;
//pub const CELL_SYNAPSES_PER_SEGMENT: usize = SYNAPSES_PER_DENDRITE * CELL_DENDRITES_PER_SEGMENT;

pub const LAYERS_PER_SEGMENT: usize = 16;
pub const CELLS_PER_LAYER: usize = COLUMNS_PER_SEGMENT;
//pub const DENDRITES_PER_LAYER: usize = CELLS_PER_LAYER * DENDRITES_PER_CELL;
//pub const SYNAPSES_PER_LAYER: usize = CELLS_PER_LAYER * SYNAPSES_PER_CELL;


pub const SENSORY_CHORD_WIDTH_LOG2: usize = 10;
pub const SENSORY_CHORD_WIDTH: u32 = 1 << SENSORY_CHORD_WIDTH_LOG2; // COLUMNS_PER_SEGMENT;
pub const MOTOR_CHORD_WIDTH: usize = 2;


pub const SYNAPSE_REACH_LOG2: u32 = 7;
pub const SYNAPSE_REACH: u32 = 1 << SYNAPSE_REACH_LOG2;
pub const SYNAPSE_SPAN: u32 = SYNAPSE_REACH << 1;
pub const AXONS_MARGIN: usize = SYNAPSE_REACH as usize;

pub const DST_DEN_BOOST_LOG2: u8 = 0;
pub const PRX_DEN_BOOST_LOG2: u8 = 0;


pub const SYNAPSES_WORKGROUP_SIZE: u32 = 256;
pub const AXONS_WORKGROUP_SIZE: u32 = 256;


pub const ASPINY_REACH_LOG2: usize 			= 2;
pub const ASPINY_REACH:	u32					= 1 << ASPINY_REACH_LOG2;
pub const ASPINY_SPAN_LOG2: usize 			= ASPINY_REACH_LOG2 + 1;
pub const ASPINY_SPAN: u32	 				= 1 << ASPINY_SPAN_LOG2;

pub const ASPINY_HEIGHT: u8 = 1;

pub const STATE_ZERO: u8 = 0;

pub const COLUMN_DOMINANCE_FLOOR: usize = 7;

pub const DENDRITE_INITIAL_THRESHOLD_PROXIMAL: u32 = 640;
pub const DENDRITE_INITIAL_THRESHOLD_DISTAL: u32 = 512;
// ***** pub const DENDRITE_INITIAL_THRESHOLD_PROXIMAL: u32 = 550;
// ***** pub const DENDRITE_INITIAL_THRESHOLD_DISTAL: u32 = 1080;

pub const SYNAPSE_STRENGTH_FLOOR: i8 = -40;
pub const SYNAPSE_DECAY_INTERVAL: usize = 256 * 2;

pub const PREFERRED_WORKGROUP_SIZE: u32 = 256;
pub const MINIMUM_WORKGROUP_SIZE: u32 = 64;

pub const LEARNING_ACTIVE: bool = true;	// *****

//pub const HORIZONTAL_AXON_ROW_FLOOR: u8 = 200;




pub const CL_BUILD_OPTIONS: &'static str = "-cl-denorms-are-zero -cl-fast-relaxed-math";



//pub const SYNAPSE_FLAG_ADD_PENDING_ACTIVATION: u8 = 0b00000001;







/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/








pub fn build_options() -> String {

	assert!(SENSORY_CHORD_WIDTH % SYNAPSE_SPAN == 0);

	assert!(SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 >= 2);
	assert!(SYNAPSES_PER_DENDRITE_DISTAL_LOG2 >= 2);

	assert!(DENDRITES_PER_CELL_DISTAL_LOG2 <= 8);
	assert!(DENDRITES_PER_CELL_DISTAL <= 256);
	assert!(DENDRITES_PER_CELL_PROXIMAL_LOG2 == 0);

	BuildOptions::new(CL_BUILD_OPTIONS)
		.opt("SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2", SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 as i32)
		.opt("COLUMN_DOMINANCE_FLOOR", COLUMN_DOMINANCE_FLOOR as i32)
		.opt("ASPINY_REACH_LOG2", ASPINY_REACH_LOG2 as i32)
		.opt("DENDRITES_PER_CELL_DISTAL_LOG2", DENDRITES_PER_CELL_DISTAL_LOG2 as i32)
		.opt("DENDRITES_PER_CELL_DISTAL", DENDRITES_PER_CELL_DISTAL as i32)
		.opt("DENDRITES_PER_CELL_PROXIMAL_LOG2", DENDRITES_PER_CELL_PROXIMAL_LOG2 as i32)
		//.opt("SYNAPSES_PER_CELL_PROXIMAL_LOG2", SYNAPSES_PER_CELL_PROXIMAL_LOG2 as i32)
		.opt("SYNAPSE_REACH", SYNAPSE_REACH as i32)
		.opt("ASPINY_REACH", ASPINY_REACH as i32)
		.opt("ASPINY_SPAN_LOG2", ASPINY_SPAN_LOG2 as i32)
		.opt("ASPINY_SPAN", ASPINY_SPAN as i32)
		.opt("DENDRITE_INITIAL_THRESHOLD_PROXIMAL", DENDRITE_INITIAL_THRESHOLD_PROXIMAL as i32)
		.opt("SYNAPSE_STRENGTH_FLOOR", SYNAPSE_STRENGTH_FLOOR as i32)
		//.opt("HORIZONTAL_AXON_ROW_FLOOR", HORIZONTAL_AXON_ROW_FLOOR as i32)
		.to_string()
}


pub struct BuildOptions {
	options: Vec<BuildOption>,
	string: String,
}

impl BuildOptions {
	pub fn new(cl_options: &'static str) -> BuildOptions {
		let mut bo = BuildOptions {
			options: Vec::with_capacity(1 << 5),
			string: String::with_capacity(1 << 11),
		};

		bo.str(cl_options)
	}

	pub fn str(mut self, st: &'static str) -> BuildOptions {
		self.string.push_str(st);
		self
	}

	pub fn opt(mut self, name: &'static str, val: i32) -> BuildOptions {
		self.options.push(BuildOption::new(name, val));
		self
	}

	pub fn as_slice(&mut self) -> &str {
		&self.string
	}

	pub fn to_string(mut self) -> String {
		for option in self.options.iter_mut() {
			self.string.push_str(option.as_slice());
		}
		//println!("\n\tBuildOptions::as_slice(): length: {}, \n \tstring: {}", self.string.len(), self.string);
		self.string
	}

}



pub struct BuildOption {
	name: &'static str,
	val: i32,
	string: String,
}

impl BuildOption {
	pub fn new(name: &'static str, val: i32) -> BuildOption {
		BuildOption {
			name: name,
			val: val,
			string: String::with_capacity(name.len()),
		}
	}

	pub fn as_slice(&mut self) -> &str {
		self.string = format!(" -D{}={}", self.name, self.val);

		&self.string
	}
}





/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/






pub fn print_vec_simple<T: Integer + Display + Default + NumCast + Copy + FromPrimitive + ToPrimitive + UpperHex >(vec: &Vec<T>) {
	print_vec(vec, 1, None, None, true);
}


pub fn print_vec<T: Integer + Display + Default + NumCast + Copy + FromPrimitive + ToPrimitive + UpperHex >(
			vec: &Vec<T>, 
			every: usize, 
			val_range: Option<(T, T)>, 
			idx_range: Option<(usize, usize)>,
			show_zeros: bool, 
) {


	/*let val_range = match val_range {
		Some(x) => x,
		_ => 0,
	}*/
	let (ir_start, ir_end) = match idx_range {
		Some(ir)	=> ir,
		None		=> (0usize, 0usize),
	};

	let (vr_start, vr_end) = match val_range {
		Some(vr)	=> vr,
		None		=> (Default::default(), Default::default()),
	};

	let mut ttl_nz = 0usize;
	let mut ttl_ir = 0usize;
	let mut within_idx_range = false;
	let mut hi: T = vr_start;
	let mut lo: T = vr_end;
	let mut sum: isize = 0;
	let mut ttl_prntd: usize = 0;
	let len = vec.len();


	let mut color: &'static str = C_DEFAULT;
	let mut prnt: bool = false;

	print!("{cdgr}[{cg}{}{cdgr}/{}", vec.len(), every, cg = C_GRN, cdgr = C_DGR);

	if val_range.is_some() {
		print!(";[{},{}]", vr_start, vr_end);
	}

	if idx_range.is_some() {
		 		// DUPLICATE
		print!(";[{},{}]", ir_start, ir_end);
	}
	print!("]:{cd} ", cd = C_DEFAULT,);


		/* Yes, this clusterfuck needs rewriting someday */
	for i in 0..vec.len() {

		prnt = false;

		if every != 0 {
			if i % every == 0 {
				prnt = true;
			} else {
				prnt = false;
			}
		}

		if idx_range.is_some() {
			let ir = idx_range.as_ref().unwrap();

			if i < ir_start || i > ir_end {
				prnt = false;
				within_idx_range = false;
			} else {
				within_idx_range = true;
			}
		} else {
			within_idx_range = true;
		}

		if val_range.is_some() {
			if vec[i] < vr_start || vec[i] > vr_end {
				prnt = false;
			} else if within_idx_range {
				if show_zeros && vec[i] == Default::default() {
					ttl_ir += 1;
				} else if vec[i] != Default::default() {
					ttl_ir += 1;
				}
			}
		} else {
			//ttl_ir += 1;
		}

		sum += vec[i].to_isize().expect("common::print_vec(): vec[i]");
		//sum += std::num::cast::<T, isize>(vec[i]).expect("common::print_vec, sum");


		if vec[i] > hi { hi = vec[i] };

		if (vec[i] < lo) && (vec[i] != Default::default()) { lo = vec[i] };

		if vec[i] != Default::default() {
			ttl_nz += 1usize;
			color = C_ORA;
		} else {
			if show_zeros {
				color = C_DEFAULT;
			} else {
				prnt = false;
			}
		}

		if prnt {
			print!("{cg}[{cd}{}{cg}:{cc}{}{cg}]{cd}", i, vec[i], cc = color, cd = C_DEFAULT, cg = C_DGR);
			ttl_prntd += 1;
		}
	}

	let mut anz: f32 = 0f32;
	let mut nz_pct: f32 = 0f32;

	let mut ir_pct: f32 = 0f32;
	let mut avg_ir: f32 = 0f32;

	if ttl_nz > 0 {
		anz = sum as f32 / ttl_nz as f32;
		nz_pct = (ttl_nz as f32 / len as f32) * 100f32;
		//print!("[ttl_nz: {}, nz_pct: {:.0}%, len: {}]", ttl_nz, nz_pct, len);
	}

	if ttl_ir > 0 {
		avg_ir = sum as f32 / ttl_ir as f32;
		ir_pct = (ttl_ir as f32 / len as f32) * 100f32;
		//print!("[ttl_nz: {}, nz_pct: {:.0}%, len: {}]", ttl_nz, nz_pct, len);
	}


	println!("{cdgr}:(nz:{clbl}{}{cdgr}({clbl}{:.2}%{cdgr}),ir:{clbl}{}{cdgr}({clbl}{:.2}%{cdgr}),hi:{},lo:{},anz:{:.2},prntd:{}){cd} ", ttl_nz, nz_pct, ttl_ir, ir_pct, hi, lo, anz, ttl_prntd, cd = C_DEFAULT, clbl = C_LBL, cdgr = C_DGR);
}

pub fn shuffled_vec<T: Integer + Default + Display + NumCast + Copy + Clone + ToPrimitive + FromPrimitive >(size: usize, min_val: T, max_val: T) -> Vec<T> {

	//println!("min_val: {}, max_val: {}", min_val, max_val);

	//let min: isize = num::cast(min_val).expect("common::shuffled_vec(), min");
	//let max: isize = num::cast::<T, isize>(max_val).expect("common::shuffled_vec(), max") + 1is;
	//let size: usize = num::cast(max_val - min_val).expect("common::shuffled_vec(), size");
	//let size: usize = num::from_int(max - min).expect("common::shuffled_vec(), size");

	//assert!(max - min > 0, "Vector size must be greater than zero.");
	let mut vec: Vec<T> = Vec::with_capacity(size);

	assert!(size > 0, "\ncommon::shuffled_vec(): Vector size must be greater than zero.");
	assert!(min_val < max_val, "\ncommon::shuffled_vec(): Minimum value must be less than maximum.");

	let min = min_val.to_isize().expect("\ncommon::shuffled_vec(), min");
	let max = max_val.to_isize().expect("\ncommon::shuffled_vec(), max") + 1;

	let mut range = (min..max).cycle();

	for i in (0..size) {
		vec.push(FromPrimitive::from_isize(range.next().expect("\ncommon::shuffled_vec(), range")).expect("\ncommon::shuffled_vec(), from_usize"));
	}

	//let mut vec: Vec<T> = (min..max).cycle().take(size).collect();


	/*let mut vec: Vec<T> = iter::range_inclusive::<T>(min_val, max_val).cycle().take(size).collect();*/

	
	shuffle_vec(&mut vec);

	vec

}

// Fisher-Yates
pub fn shuffle_vec<T: Integer + Copy >(vec: &mut Vec<T>) {
	let len = vec.len();
	let mut rng = rand::weak_rng();

	let mut ridx: usize;
	let mut tmp: T;

	for i in 0..len {
		ridx = distributions::Range::new(i, len).ind_sample(&mut rng);
		tmp = vec[i];
		vec[i] = vec[ridx];
		vec[ridx] = tmp;
	}

}

/* SPARSE_VEC():

	sp_fctr_log2: sparsity factor (log2)
*/
pub fn sparse_vec<T: Integer + Signed + Default + Copy + Clone + NumCast + FromPrimitive + ToPrimitive >(size: usize, min_val: T, max_val: T, sp_fctr_log2: usize) -> Vec<T> {
	let mut vec: Vec<T> = iter::repeat(min_val).cycle().take(size).collect();

	let len = vec.len();

	let notes = len >> sp_fctr_log2;

	let range_max: isize = max_val.to_isize().expect("common::sparse_vec(): max_val.to_isize()") as isize + 1;
	let range_min: isize = min_val.to_isize().expect("common::sparse_vec(): min_val.to_isize()") as isize;

	let mut rng = rand::weak_rng();
	let val_range = Range::new(range_min, range_max);
	let idx_range = Range::new(0, 1 << sp_fctr_log2);

	for i in 0..notes {
		vec[(i << sp_fctr_log2) + idx_range.ind_sample(&mut rng)] = FromPrimitive::from_isize(val_range.ind_sample(&mut rng)).expect("common::sparse_vec()");
		//vec[(i << sp_fctr_log2) + idx_range.ind_sample(&mut rng)] = std::num::cast(val_range.ind_sample(&mut rng)).unwrap();
	}

	vec
}

pub fn dup_check<T: Integer + Copy + Clone + Ord >(in_vec: &mut Vec<T>) -> (usize, usize) {
	

	let mut vec = in_vec.clone();

	vec.sort();


	let mut dups = 0usize;
	let mut unis = 0usize;
	let mut prev_val = vec[vec.len() - 1];

	for x in vec.iter() {
		if prev_val == *x {
			dups += 1;
			//print!{"[{}]", *x};
		} else {
			unis += 1;
		}
		prev_val = *x;
	}

	println!("len: {}, dups: {}, unis: {}", vec.len(), dups, unis);
	(dups, unis)
}


/*pub fn log2(n: u32) -> u32 {
	let mut t = n;
	t = t | t >> 1;
	t = t | t >> 2;
	t = t | t >> 4;
	t = t | t >> 8;
	t = t | t >> 16;
	assert!((t - (t >> 1)).trailing_zeros() == t.trailing_zeros());

	(t - (t >> 1)).trailing_zeros()
}*/


pub fn log2(n: u32) -> u32 {
	n.trailing_zeros()
}


pub fn render_sdr<T: Integer + Display + Default + NumCast + Copy + FromPrimitive + ToPrimitive + UpperHex >(
			//title: &'static str,
			ff_vec: &[T],
			col_output_vec: &[T],
			row_map: &BTreeMap<u8, &'static str>,
			//condense_factor: usize, 
			//val_range: Option<(T, T)>, 
			//idx_range: Option<(usize, usize)>,
) {

	assert!(ff_vec.len() == col_output_vec.len(), "common::render_sdr(): Input vectors must be of equal length.");
	let cells_per_line = 64;
	let line_character_width = (cells_per_line * (4 + 4 + 2 + 4 + 4 + 1)) + 8;	// 8 extra for funsies

	//println!("\n[{}{}{}]:", C_GRN, ff_vec.len(), C_DEFAULT);

	let mut line_out: String = String::with_capacity(line_character_width);
	let mut line_i = 0usize;	// RENAME ME FOR FUCKS SAKE
	let mut global_i = 0usize;		// AND ME TOO IF YOU GIVE EVEN ONE SINGLE SHIT FOR HUMANITY!!!!

	print!("\n");
	io::stdout().flush().ok();

	loop {
		if line_i >= ff_vec.len() { break }

		line_out.clear();

		for i in line_i..(line_i + cells_per_line) {
			let output_active = col_output_vec[i] != Default::default();
			let col_active = ff_vec[i] != Default::default();
			let prediction = col_output_vec[i] != ff_vec[i];
			let new_prediction = prediction && (!col_active);

			if prediction && (!new_prediction) {
				line_out.push_str(BGC_DGR);
			} else {
				line_out.push_str(BGC_DEFAULT);
			}

			if output_active {
				if new_prediction {
					line_out.push_str(C_MAG);
				} else {
					line_out.push_str(C_BLU);
				}
			} else {
				line_out.push_str(C_DEFAULT);
			}

			//if  && !output_active 

			if output_active {
				line_out.push_str(&format!("{:02X}", col_output_vec[i]));
			} else {
				if (i & 0x07) == 0 || (global_i & 0x07) == 0 {				// || ((global_i & 0x0F) == 7) || ((global_i & 0x0F) == 8)
					line_out.push_str("  ");
				} else {
					line_out.push_str("--");
				}
			} 

			line_out.push_str(C_DEFAULT);
			line_out.push_str(BGC_DEFAULT);
			line_out.push_str(" ");
		}

		if ((global_i & 0xF) == 00) && (ff_vec.len() > SENSORY_CHORD_WIDTH as usize) {
			let row_id = (global_i >> 4) as u8;
			let row_name = match row_map.get(&row_id) {
				Some(&name) => name,
				None => "<render_sdr(): row name not found in map>",
			};
			println!("\n[{}: {}]", row_id, row_name);
		}
		
		println!("{}",line_out);

		line_i += cells_per_line;
		global_i += 1;
	}

}




/*let mut test_test: String = String::new();
	test_test.push_str(C_DEFAULT);
	println!("### color code length: {} ###", test_test.len());*/
