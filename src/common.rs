
use std;
use std::num::{ Int, FromPrimitive, ToPrimitive, SignedInt };
use std::ops::{ self, BitOr };
use std::default::{ Default }; 
use std::fmt::{ Display, Debug };
use std::num;
use std::iter;
use std::rand;
use std::rand::distributions::{ self, Normal, IndependentSample, Range };

use ocl;

pub static C_DEFAULT: &'static str = "\x1b[0m";
pub static C_RED: &'static str = "\x1b[91m";
pub static C_CYA: &'static str = "\x1b[36m";
pub static C_GRN: &'static str = "\x1b[32m";
pub static C_BLU: &'static str = "\x1b[94m";
pub static C_MAG: &'static str = "\x1b[95m";
pub static C_PUR: &'static str = "\x1b[35m";
pub static C_ORA: &'static str = "\x1b[33m";
pub static C_YEL: &'static str = "\x1b[93m";
pub static C_LBL: &'static str = "\x1b[94m";
pub static C_LGR: &'static str = "\x1b[37m";
pub static C_DGR: &'static str = "\x1b[90m";

pub static KERNELS_FILE_NAME: &'static str = "bismit.cl";

pub const CORTICAL_SEGMENTS_TOTAL: usize = 1;
pub const SENSORY_SEGMENTS_TOTAL: usize = 1;
pub const MOTOR_SEGMENTS_TOTAL: usize = 1;

pub const HYPERCOLUMNS_PER_SEGMENT: usize = 16;		// appears to cause lots of delay... 256 is slow

pub const SYNAPSE_STRENGTH_INITIAL_DEVIATION: i8 = 3;
pub const DENDRITE_INITIAL_THRESHOLD: i8 = 1;

pub const DST_SYNAPSE_STRENGTH_DEFAULT: i8 = 16;
pub const PRX_SYNAPSE_STRENGTH_DEFAULT: i8 = 64;

pub const COLUMNS_PER_HYPERCOLUMN: u32 = 64;

pub const DENDRITES_PER_CELL_DISTAL_LOG2: u32 = 4;
pub const DENDRITES_PER_CELL_PROXIMAL_LOG2: u32 = 0;
//pub const DENDRITES_PER_CELL_APICAL_LOG2: u32 = 3;

pub const DENDRITES_PER_CELL_DISTAL: u32 = 1 << DENDRITES_PER_CELL_DISTAL_LOG2;
pub const DENDRITES_PER_CELL_PROXIMAL: u32 = 1 <<DENDRITES_PER_CELL_PROXIMAL_LOG2;
//pub const DENDRITES_PER_CELL_APICAL: u32 = 8;

pub const SYNAPSES_PER_DENDRITE: u32 = 16;
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

pub const SENSORY_CHORD_WIDTH: u32 = 1024; // COLUMNS_PER_SEGMENT;
pub const MOTOR_CHORD_WIDTH: usize = 2;

pub const SYNAPSE_REACH: u32 = 128;
pub const MAX_SYNAPSE_RANGE: u32 = SYNAPSE_REACH * 2;
pub const AXONS_MARGIN: usize = 128;

pub const DST_DEN_BOOST_LOG2: u8 = 0;
pub const PRX_DEN_BOOST_LOG2: u8 = 0;

pub const SYNAPSE_DECAY_INTERVAL: usize = 256 * 64;
 
pub const SYNAPSE_WORKGROUP_SIZE: usize = 256;


pub fn print_vec<T: Int + Display + Default>(vec: &Vec<T>, every: usize, show_zeros: bool, val_range: Option<std::ops::Range<T>>) {


	/*let val_range = match val_range {
		Some(x) => x,
		_ => 0,
	}*/

	let mut ttl_nz = 0us;
	let mut ttl_ir = 0us;
	let mut hi = Default::default();
	let mut lo: T = Default::default();
	let mut sum: i64 = 0;
	let mut ttl_prntd: usize = 0;
	let len = vec.len();


	let mut color: &'static str = C_DEFAULT;
	let mut prnt: bool = false;

	print!("{cdgr}[{cg}{}{cdgr}/{}", vec.len(), every, cg = C_GRN, cdgr = C_DGR);
	if val_range.is_some() {
		let vr = val_range.as_ref().unwrap(); 		// DUPLICATE
		print!("({}-{})", vr.start, vr.end);
	}
	print!("]:{cd} ", cd = C_DEFAULT,);

	for i in range(0, vec.len()) {

		prnt = false;

		if every != 0 {
			if i % every == 0 {
				prnt = true;
			} else {
				prnt = false;
			}
		}

		if val_range.is_some() {
			let vr = val_range.as_ref().unwrap();	// DUPLICATE
			if vec[i] < vr.start || vec[i] > vr.end {
				prnt = false;
			} else {
				ttl_ir += 1;
			}
		} else {
			ttl_ir += 1;
		}

		sum += num::cast(vec[i]).expect("common::print_vec, sum");

		if vec[i] != Default::default() {
			ttl_nz += 1us;
			if vec[i] > hi { hi = vec[i] };
			if lo == Default::default() && hi != Default::default() {
				lo = hi 
			} else {
				if vec[i] < lo { lo = vec[i] };
			}
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


	println!("{cdgr}:(nz:{clbl}{}{cdgr}({clbl}{:.2}%{cdgr}),ir:{clbl}{}{cdgr}({clbl}{:.2}%{cdgr}),hi:{},lo:{},anz:{:.2},prtd:{}){cd} ", ttl_nz, nz_pct, ttl_ir, ir_pct, hi, lo, anz, ttl_prntd, cd = C_DEFAULT, clbl = C_LBL, cdgr = C_DGR);
}

pub fn int_hb_log2<T: Int + BitOr + Eq >(mut n: T) -> u8 {
	let tmp = n;
	n = n | n >> 1;
	n = n | n >> 2;
	n = n | n >> 4;
	n = n | n >> 8;
	n = n | n >> 16;
	assert!((n - (n >> 1)).trailing_zeros() == tmp.trailing_zeros());
	FromPrimitive::from_uint((n - (n >> 1)).trailing_zeros()).expect("common::int_hb_log2")
}

pub fn shuffled_vec<T: Int + FromPrimitive + ToPrimitive + Default + Display>(size: usize, min_val: T, max_val: T) -> Vec<T> {

	//println!("min_val: {}, max_val: {}", min_val, max_val);

	//let min: isize = num::cast(min_val).expect("common::shuffled_vec(), min");
	//let max: isize = num::cast::<T, isize>(max_val).expect("common::shuffled_vec(), max") + 1is;
	//let size: usize = num::cast(max_val - min_val).expect("common::shuffled_vec(), size");
	//let size: usize = num::from_int(max - min).expect("common::shuffled_vec(), size");

	//assert!(max - min > 0, "Vector size must be greater than zero.");

	assert!(size > 0, "Vector size must be greater than zero.");


	let mut vec: Vec<T> = iter::range_inclusive(min_val, max_val).cycle().take(size).collect();

	//println!("shuffled_vec(): vec.len(): {}", vec.len());
	/*let mut i: usize = 0;
	for val in iter::range_inclusive(min_val, max_val) {
		vec[i] = val;
		//vec[i] = FromPrimitive::from_int(val).expect("common::shuffled_vec(), vec[i]");
		i += 1;
	}*/

	shuffle_vec(&mut vec);

	vec

}

pub fn shuffle_vec<T: Int + FromPrimitive + ToPrimitive + Default>(vec: &mut Vec<T>) {
	let size = vec.len();

	let mut rng = rand::weak_rng();
	let rng_range = distributions::Range::new(0, size);

	for i in range(0, 6) {
		for j in range(0, size) {
			let ridx = rng_range.ind_sample(&mut rng);
			let tmp = vec[j];
			vec[j] = vec[ridx];
			vec[ridx] = tmp;
		}
	}

}

/* SPARSE_VEC():

	sp_fctr_log2: sparsity factor (log2)
*/
pub fn sparse_vec<T: SignedInt + FromPrimitive + ToPrimitive + Default>(size: usize, min_val: T, max_val: T, sp_fctr_log2: usize) -> Vec<T> {
	let mut vec: Vec<T> = iter::repeat(min_val).cycle().take(size).collect();

	let len = vec.len();

	let notes = len >> sp_fctr_log2;

	let range_max = max_val.to_i64().expect("common::sparse_vec(): max_val.to_i64()") as isize + 1;
	let range_min = min_val.to_i64().expect("common::sparse_vec(): min_val.to_i64()") as isize;

	let mut rng = rand::weak_rng();
	let val_range = Range::new(range_min, range_max);
	let idx_range = Range::new(0, 1 << sp_fctr_log2);

	for i in range(0, notes) {
		vec[(i << sp_fctr_log2) + idx_range.ind_sample(&mut rng)] = num::cast(val_range.ind_sample(&mut rng)).unwrap();
	}

	vec
}

pub fn dup_check<T: Int>(in_vec: &Vec<T>) -> (usize, usize) {
	

	let mut vec = in_vec.clone();

	vec.sort();


	let mut dups = 0us;
	let mut unis = 0us;
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
