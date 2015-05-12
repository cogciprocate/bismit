
//use prediction;

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


pub const PYR_JUST_ACTIVE_FLAG		: u8 = 0b10000000;
pub const PYR_BEST_COL_DEN_FLAG		: u8 = 0b01000000;


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


//pub const DENDRITES_PER_CELL_DISTAL_LOG2: u32 = 2; 
pub const DENDRITES_PER_CELL_DISTAL_LOG2: u32 = 3;
pub const DENDRITES_PER_CELL_DISTAL: u32 = 1 << DENDRITES_PER_CELL_DISTAL_LOG2;

//pub const SYNAPSES_PER_DENDRITE_DISTAL_LOG2: u32 = 2; 
pub const SYNAPSES_PER_DENDRITE_DISTAL_LOG2: u32 = 4;
pub const SYNAPSES_PER_DENDRITE_DISTAL: u32 = 1 << SYNAPSES_PER_DENDRITE_DISTAL_LOG2;



pub const DENDRITES_PER_CELL_PROXIMAL_LOG2: u32 = 0;
pub const DENDRITES_PER_CELL_PROXIMAL: u32 = 1 << DENDRITES_PER_CELL_PROXIMAL_LOG2;


//pub const SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2: u32 = 3; 
pub const SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2: u32 = 6;
pub const SYNAPSES_PER_DENDRITE_PROXIMAL: u32 = 1 << SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;


pub const SYNAPSE_REACH_LOG2: u32 = 7;
pub const SYNAPSE_REACH: u32 = 1 << SYNAPSE_REACH_LOG2;
pub const SYNAPSE_SPAN: u32 = SYNAPSE_REACH << 1;
pub const AXONS_MARGIN: usize = SYNAPSE_REACH as usize;

pub const SYNAPSE_ROW_POOL_SIZE: u32 = 256;

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

pub const DENDRITE_INITIAL_THRESHOLD_PROXIMAL: u32 = 300;
pub const DENDRITE_INITIAL_THRESHOLD_DISTAL: u32 = 400;
// ***** pub const DENDRITE_INITIAL_THRESHOLD_PROXIMAL: u32 = 550;
// ***** pub const DENDRITE_INITIAL_THRESHOLD_DISTAL: u32 = 1080;

pub const SYNAPSE_STRENGTH_FLOOR: i8 = -15;
pub const SYNAPSE_REGROWTH_INTERVAL: usize = 1000;

pub const PREFERRED_WORKGROUP_SIZE: u32 = 256;
pub const MINIMUM_WORKGROUP_SIZE: u32 = 64;

pub const LEARNING_ACTIVE: bool = true;	// *****

//pub const HORIZONTAL_AXON_ROW_DEMARCATION: u8 = 200;




pub static KERNELS_FILE_NAME: &'static str = "bismit.cl";
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








pub fn build_options() -> ocl::BuildOptions {

	assert!(SENSORY_CHORD_WIDTH % SYNAPSE_SPAN == 0);

	assert!(SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 >= 2);
	assert!(SYNAPSES_PER_DENDRITE_DISTAL_LOG2 >= 2);

	assert!(DENDRITES_PER_CELL_DISTAL_LOG2 <= 8);
	assert!(DENDRITES_PER_CELL_DISTAL <= 256);
	assert!(DENDRITES_PER_CELL_PROXIMAL_LOG2 == 0);

	ocl::BuildOptions::new(CL_BUILD_OPTIONS)
		.opt("SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2", SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 as i32)
		.opt("COLUMN_DOMINANCE_FLOOR", COLUMN_DOMINANCE_FLOOR as i32)
		.opt("ASPINY_REACH_LOG2", ASPINY_REACH_LOG2 as i32)
		.opt("DENDRITES_PER_CELL_DISTAL_LOG2", DENDRITES_PER_CELL_DISTAL_LOG2 as i32)
		.opt("DENDRITES_PER_CELL_DISTAL", DENDRITES_PER_CELL_DISTAL as i32)
		.opt("DENDRITES_PER_CELL_PROXIMAL_LOG2", DENDRITES_PER_CELL_PROXIMAL_LOG2 as i32)
		//.opt("SYNAPSES_PER_CELL_PROXIMAL_LOG2", SYNAPSES_PER_CELL_PROXIMAL_LOG2 as i32)
		.opt("SYNAPSE_REACH", SYNAPSE_REACH as i32)
		.opt("SYNAPSE_SPAN", SYNAPSE_SPAN as i32)
		.opt("ASPINY_REACH", ASPINY_REACH as i32)
		.opt("ASPINY_SPAN_LOG2", ASPINY_SPAN_LOG2 as i32)
		.opt("ASPINY_SPAN", ASPINY_SPAN as i32)
		.opt("DENDRITE_INITIAL_THRESHOLD_PROXIMAL", DENDRITE_INITIAL_THRESHOLD_PROXIMAL as i32)
		.opt("SYNAPSE_STRENGTH_FLOOR", SYNAPSE_STRENGTH_FLOOR as i32)
		.opt("PYR_JUST_ACTIVE_FLAG", PYR_JUST_ACTIVE_FLAG as i32)
		.opt("PYR_BEST_COL_DEN_FLAG", PYR_BEST_COL_DEN_FLAG as i32)
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

		sum += vec[i].to_isize().expect("cmn::print_vec(): vec[i]");
		//sum += std::num::cast::<T, isize>(vec[i]).expect("cmn::print_vec, sum");


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

	//let min: isize = num::cast(min_val).expect("cmn::shuffled_vec(), min");
	//let max: isize = num::cast::<T, isize>(max_val).expect("cmn::shuffled_vec(), max") + 1is;
	//let size: usize = num::cast(max_val - min_val).expect("cmn::shuffled_vec(), size");
	//let size: usize = num::from_int(max - min).expect("cmn::shuffled_vec(), size");

	//assert!(max - min > 0, "Vector size must be greater than zero.");
	let mut vec: Vec<T> = Vec::with_capacity(size);

	assert!(size > 0, "\ncmn::shuffled_vec(): Vector size must be greater than zero.");
	assert!(min_val < max_val, "\ncmn::shuffled_vec(): Minimum value must be less than maximum.");

	let min = min_val.to_isize().expect("\ncmn::shuffled_vec(), min");
	let max = max_val.to_isize().expect("\ncmn::shuffled_vec(), max") + 1;

	let mut range = (min..max).cycle();

	for i in (0..size) {
		vec.push(FromPrimitive::from_isize(range.next().expect("\ncmn::shuffled_vec(), range")).expect("\ncmn::shuffled_vec(), from_usize"));
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

	let range_max: isize = max_val.to_isize().expect("cmn::sparse_vec(): max_val.to_isize()") as isize + 1;
	let range_min: isize = min_val.to_isize().expect("cmn::sparse_vec(): min_val.to_isize()") as isize;

	let mut rng = rand::weak_rng();
	let val_range = Range::new(range_min, range_max);
	let idx_range = Range::new(0, 1 << sp_fctr_log2);

	for i in 0..notes {
		vec[(i << sp_fctr_log2) + idx_range.ind_sample(&mut rng)] = FromPrimitive::from_isize(val_range.ind_sample(&mut rng)).expect("cmn::sparse_vec()");
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
	if n > 0 {
		31 - n.leading_zeros()
	} else {
		0
	}
}



/*pub fn render_sdr<T: Integer + Display + Default + NumCast + Copy + FromPrimitive + ToPrimitive + UpperHex >(
			vec_out: &[T], 
			vec_ff_opt: Option<&[T]>, 
			vec_out_prev_opt: Option<&[T]>, 
			vec_ff_prev_opt: Option<&[T]>,
			row_map: &BTreeMap<u8, &'static str>,
) {
*/
pub fn render_sdr(
			vec_out: &[u8], 
			vec_ff_opt: Option<&[u8]>, 
			vec_out_prev_opt: Option<&[u8]>, 
			vec_ff_prev_opt: Option<&[u8]>,
			row_map: &BTreeMap<u8, &'static str>,
			print: bool,
) -> f32 {
	let vec_ff = match vec_ff_opt {
		Some(v) => v,
		None => vec_out.clone(),
	};

	let vec_out_prev = match vec_out_prev_opt {
		Some(v) => v,
		None => vec_out.clone(),
	};

	let vec_ff_prev = match vec_ff_prev_opt {
		Some(v) => v,
		None => vec_out.clone(),
	};

	assert!(vec_ff.len() == vec_out.len() && vec_out.len() == vec_out_prev.len() && vec_out.len() == vec_ff_prev.len(), 
		"cmn::render_sdr(): Input vectors must be of equal length.");

	let mut failed_preds = 0usize;
	let mut corr_preds = 0usize;
	let mut missed_preds = 0usize;

	let mut new_preds = 0usize;

	let region_cells_per_line = 64;
	let line_character_width = (region_cells_per_line * (4 + 4 + 2 + 4 + 4 + 1)) + 8;	// 8 extra for funsies

	//println!("\n[{}{}{}]:", C_GRN, vec_ff.len(), C_DEFAULT);

	let mut out_line: String = String::with_capacity(line_character_width);
	let mut i_line = 0usize;
	let mut i_global = 0usize;

	print!("\n");
	io::stdout().flush().ok();

	loop {
		if i_line >= vec_ff.len() { break }

		out_line.clear();

		for i in i_line..(i_line + region_cells_per_line) {
			let cur_active = vec_out[i] != Default::default();
			let col_active = vec_ff[i] != Default::default();
			let prediction = vec_out[i] != vec_ff[i];
			let new_prediction = prediction && (!col_active);

			//let prev_active = vec_ff_prev[i] != Default::default();
			let prev_prediction = new_pred(vec_out_prev[i], vec_ff_prev[i]);

			if new_prediction { new_preds += 1 };

			if (prev_prediction && !new_prediction) && !col_active {
				failed_preds += 1;
			} else if prev_prediction && col_active {
				corr_preds += 1;
			}

			if cur_active && !prev_prediction {
				missed_preds += 1;
			}

			if print {
				if cur_active {
					if new_prediction {
						assert!(new_pred(vec_out[i], vec_ff[i]));
						out_line.push_str(C_MAG);
					} else {

						out_line.push_str(C_BLU);
					}
					/*if corr_pred(vec_out[i], vec_ff[i], vec_out_prev[i], vec_ff_prev[i]) {
						corr_preds += 1;
					}*/
				} else {
					out_line.push_str(C_DEFAULT);
				}

				if cur_active {
					out_line.push_str(&format!("{:02X}", vec_out[i]));
				} else {
					if (i & 0x07) == 0 || (i_global & 0x07) == 0 {				// || ((i_global & 0x0F) == 7) || ((i_global & 0x0F) == 8)
						out_line.push_str("  ");
					} else {
						out_line.push_str("--");
					}
				} 

				out_line.push_str(C_DEFAULT);
				out_line.push_str(BGC_DEFAULT);
				out_line.push_str(" ");
			}
		}


		if print {
			if ((i_global & 0xF) == 00) && (vec_ff.len() > SENSORY_CHORD_WIDTH as usize) {
				let row_id = (i_global >> 4) as u8;
				let row_name = match row_map.get(&row_id) {
					Some(&name) => name,
					None => "<render_sdr(): row name not found in map>",
				};
				println!("\n[{}: {}]", row_id, row_name);
			}
			
			println!("{}", out_line);
		}

		i_line += region_cells_per_line;
		i_global += 1;
	}


	let preds_total = (corr_preds + failed_preds + missed_preds) as f32;

	let pred_accy = if preds_total > 0f32 {
		(corr_preds as f32 / preds_total) * 100f32
	} else {
		0f32
	};

	if print {
		if vec_out_prev_opt.is_some() {
			println!("\n[correct: {}, failed: {}, missed: {}, accuracy: {:.1}%, new_preds: {}]", 
				corr_preds, failed_preds, missed_preds, pred_accy, new_preds);
		}
	}

	pred_accy
}


pub fn corr_pred(
			out: u8, 
			ff: u8, 
			prev_out: u8, 
			prev_ff: u8, 
) -> Option<bool> {
	let prev_new_pred = new_pred(prev_out, prev_ff);
	let curr_new_pred = new_pred(out, ff);

	if prev_new_pred && (ff != 0) {
		Some(true)
	} else if prev_new_pred && curr_new_pred {
		None
	} else {
		Some(false)
	}
}


pub fn new_pred(
			out: u8, 
			ff: u8, 
) -> bool {
	let out_active = out != 0;
	let ff_active = ff != 0;
	let pred = out != ff;
	let new_pred = pred && (!ff_active);

	new_pred
}


/*fn pred_accy<T: Integer + Display + Default + NumCast + Copy + FromPrimitive + ToPrimitive + UpperHex>(
			vec_out: &[T], 
			vec_ff: &[T], 
			prev_vec: &[T], 
) -> f32 {
	assert!(vec_out.len() == vec_ff.len() && vec_out.len() == prev_vec.len());

	let len = vec_out.len();
	let mut corr_pred = 0usize;
	let mut icor_pred = 0usize;

	for i in 0..len {


	}

}*/





/* AXN_IDX_2D(): Host side address resolution - concerned with start idx of a row
	- OpenCL device side version below (for reference) - concerned with invidiual indexes: 
		static inline uint axn_idx_2d(uchar row_id, uint row_width, uint col_id, char col_ofs) {
			uint axn_idx_spt = mad24((uint)row_id, row_width, (uint)(col_id + col_ofs + SYNAPSE_REACH));
			int hrow_id = row_id - HORIZONTAL_AXON_ROW_DEMARCATION;
			int hcol_id = mad24(hrow_id, SYNAPSE_SPAN, col_ofs + SYNAPSE_REACH);
			uint axn_idx_hrz = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, row_width, (uint)(hcol_id + SYNAPSE_REACH));
			return mul24((uint)(hrow_id < 0), axn_idx_spt) + mul24((uint)(hrow_id >= 0), axn_idx_hrz);
		}
}*/
pub fn axn_idx_2d(axn_row: u8, width: u32, hrz_demarc: u8) -> u32 {
	let mut axn_idx: u32 = if axn_row < hrz_demarc {
		(axn_row as u32 * width)
	} else {
		(hrz_demarc as u32 * width) + SYNAPSE_SPAN * (axn_row as u32 - hrz_demarc as u32)
	};

	axn_idx + AXONS_MARGIN as u32
}


/* GEN_FRACT_SDR(): Generate simple SDR from integer seed
	- FUTURE IMPROVEMENTS: 
		- Once the API for wrapping integers is sorted out, use one of those instead of wrap_idx.
		- Create and store sdr as a "chord" or whatever else becomes the preferred SDR storage container

*/
pub fn gen_fract_sdr(seed: u8, len: usize) -> Vec<u8> {
	let mut vec: Vec<u8> = iter::repeat(0u8).take(len).collect();

	let mut idx = wrap_idx(seed as usize, len);
	let n = 1 + ((len >> 5) as f64).sqrt() as usize;

	for i in 0..n {
		for j in 0..(n - i) {
			vec[idx] = seed;
			idx = wrap_idx(idx + 1, len)
		}
		idx = wrap_idx(idx + (i << 1) + 1, len);
	}

	vec
}

pub fn wrap_idx(idx: usize, len: usize) -> usize {
	let mut w_idx = idx;
	loop {
		if w_idx < len {
			break;
		} else {
			w_idx -= len;
		}
	}
	w_idx
}




/*=============================================================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
================================ UNIT TESTS ===================================
===============================================================================
===============================================================================
===============================================================================
===============================================================================
=============================================================================*/






#[test]
fn test_axn_idx_2d() {
	assert!(axn_idx_2d(1, 1024, 4) == 1024u32 + AXONS_MARGIN as u32);
	assert!(axn_idx_2d(5, 1024, 4) == 4096u32 + SYNAPSE_SPAN + AXONS_MARGIN as u32);
	assert!(axn_idx_2d(15, 1024, 4) == 4096u32 + (11 * SYNAPSE_SPAN) + AXONS_MARGIN as u32);

}

#[test]
fn test_wrap_idx() {
	assert!(wrap_idx(50, 40) == 10);
	assert!(wrap_idx(30, 40) == 30);
}

#[test]
fn test_log2() {
	assert!(log2(126) == 6);
	assert!(log2(128) == 7);
	assert!(log2(129) == 7);
	assert!(log2(7) == 2);
	assert!(log2(8) == 3);
	assert!(log2(9) == 3);
}
