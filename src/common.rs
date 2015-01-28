
use std::num::{ Int, FromPrimitive, ToPrimitive };
use std::ops::{ BitOr };
use std::default::{ Default }; 
use std::fmt::{ Display };
use std::num;
use std::iter;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };

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

pub const SYNAPSE_WEIGHT_ZERO: u8 = 16;
pub const SYNAPSE_WEIGHT_INITIAL_DEVIATION: u8 = 3;
pub const DENDRITE_INITIAL_THRESHOLD: u8 = 1;

pub const COLUMNS_PER_HYPERCOLUMN: usize = 64;
//pub const COLUMNS_PER_ADDRESS_BLOCK: usize = 16u;
pub const CELLS_PER_COLUMN: usize = 16;
pub const DENDRITES_PER_NEURON: usize = 16;
pub const SYNAPSES_PER_DENDRITE: usize = 16;
pub const AXONS_PER_NEURON: usize = DENDRITES_PER_NEURON * SYNAPSES_PER_DENDRITE;
pub const SYNAPSES_PER_NEURON: usize = SYNAPSES_PER_DENDRITE * DENDRITES_PER_NEURON;

pub const COLUMNS_PER_SEGMENT: usize = COLUMNS_PER_HYPERCOLUMN * HYPERCOLUMNS_PER_SEGMENT;
pub const COLUMN_AXONS_PER_SEGMENT: usize = AXONS_PER_NEURON * COLUMNS_PER_SEGMENT;
pub const COLUMN_DENDRITES_PER_SEGMENT: usize = DENDRITES_PER_NEURON * COLUMNS_PER_SEGMENT;
pub const COLUMN_SYNAPSES_PER_SEGMENT: usize = SYNAPSES_PER_DENDRITE * COLUMN_DENDRITES_PER_SEGMENT;

pub const CELLS_PER_SEGMENT: usize = CELLS_PER_COLUMN * COLUMNS_PER_SEGMENT;
pub const CELL_AXONS_PER_SEGMENT: usize = AXONS_PER_NEURON * CELLS_PER_SEGMENT;
pub const CELL_DENDRITES_PER_SEGMENT: usize = DENDRITES_PER_NEURON * CELLS_PER_SEGMENT;
pub const CELL_SYNAPSES_PER_SEGMENT: usize = SYNAPSES_PER_DENDRITE * CELL_DENDRITES_PER_SEGMENT;

pub const SENSORY_CHORD_WIDTH: usize = 1024; // COLUMNS_PER_SEGMENT;
pub const MOTOR_CHORD_WIDTH: usize = 2;

pub fn print_vec<T: Int + Display + Default>(vec: &Vec<T>, every: usize, show_zeros: bool) {

	let mut ttl_nz = 0us;
	let mut hi = Default::default();
	let mut lo: T = Default::default();
	let mut sum: usize = 0;
	let len = vec.len();

	let mut color: &'static str = C_DEFAULT;

	print!("{cdgr}[{cg}{}{cdgr}/{}]:{cd} ", vec.len(), every, cd = C_DEFAULT, cg = C_GRN, cdgr = C_DGR);

	for i in range(0, vec.len()) {

		let mut prnt: bool;

		if i % every == 0 {
			prnt = true;
		} else {
			prnt = false;
		}

		sum += num::cast(vec[i]).unwrap();

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
				prnt = false;	// bullshit so we don't print 0's
			}
		}

		if prnt {
			print!("{cg}[{cd}{}{cg}:{cc}{}{cg}]{cd}", i, vec[i], cc = color, cd = C_DEFAULT, cg = C_DGR);
		}
	}

	let mut anz: usize = 0;
	let mut nz_pct: f32 = 0f32;

	if ttl_nz > 0 {
		anz = sum / ttl_nz;
		nz_pct = (ttl_nz as f32 / len as f32) * 100f32;
		//print!("[ttl_nz: {}, nz_pct: {:.0}%, len: {}]", ttl_nz, nz_pct, len);
	}


	print!("{cdgr}:(nz:{clbl}{}{cdgr}({clbl}{:.1}%{cdgr}),hi:{},lo:{},anz:{}){cd} ", ttl_nz, nz_pct, hi, lo, anz, cd = C_DEFAULT, clbl = C_LBL, cdgr = C_DGR);
}

pub fn int_hb_log2<T: Int + BitOr + Eq >(mut n: T) -> u8 {
	let tmp = n;
	n = n | n >> 1;
	n = n | n >> 2;
	n = n | n >> 4;
	n = n | n >> 8;
	n = n | n >> 16;
	assert!((n - (n >> 1)).trailing_zeros() == tmp.trailing_zeros());
	FromPrimitive::from_uint((n - (n >> 1)).trailing_zeros()).unwrap()
}

pub fn shuffled_vec<T: Int + FromPrimitive + ToPrimitive + Default>(size: usize, init_val: T) -> Vec<T> {

	assert!(size > 0us, "Vector size must be greater than zero.");

	let mut vec: Vec<T> = iter::repeat(init_val).take(size).collect();

	for i in range(0us, vec.len()) {
		vec[i] = FromPrimitive::from_uint(i).unwrap();
	}

	let mut rng = rand::thread_rng();
	let rng_range = Range::new(0, size);

	for i in range(0, 3) {
		for j in range(0us, vec.len()) {
			let ridx = rng_range.ind_sample(&mut rng);
			let tmp = vec[j];
			vec[j] = vec[ridx];
			vec[ridx] = tmp;
		}
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
