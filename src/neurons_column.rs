use common;
use ocl;
use cortical_component::{ CorticalComponent };

use std::num;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, Num, Primitive, UnsignedInt, Int };
use std::fmt::Show;
use std::default::{ Default };


pub struct Axons {
	pub target_columns: CorticalComponent<ocl::cl_ushort>,
	pub target_column_synapses: CorticalComponent<ocl::cl_uchar>,
}
impl Axons {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Axons {

		let mut target_columns = CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl);
		let mut target_column_synapses = CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl);

		init_axon(&mut target_columns, &mut target_column_synapses);

		Axons {
			target_columns: CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl),
			target_column_synapses: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
		}
	}
}

pub fn init_axon(
			target_cols: &mut CorticalComponent<ocl::cl_ushort>, 
			target_col_synapses: &mut CorticalComponent<ocl::cl_uchar>,

) {
	let mut rng = rand::task_rng();

	//let rng_normal = Normal::new(0f64, (common::COLUMNS_PER_SEGMENT / 8u) as f64);

	let rng_range = Range::new(0u16, (common::COLUMNS_PER_SEGMENT) as u16);

	//println!("neurons_column::init_axon(): target_cols: [len: {}]", target_cols.vec.len());
	//println!("columns per segment: {}", common::COLUMNS_PER_SEGMENT);
	 
	//let mut raw_sum = 0u64;
	//let mut norm_sum = 0u64;
	for i in range(0u, target_cols.vec.len()) {
		let rnum: u16 = rng_range.ind_sample(&mut rng);

		//let mut rnum: u16 = num::cast(rng_normal.ind_sample(&mut rng)).unwrap();

		//let val_bak: u16 = val;

		//rnum += i as u16;

		//raw_sum += val as u64;

		target_cols.vec[i] = rnum;

		//norm_sum += target_cols.vec[i] as u64;

		/*
		if i % 1000 == 0 {
			println!("[i:{}]: neurons_column::Axons.target_columns.vec[{}], [rnum: {}]", i, target_cols.vec[i], rnum);
		}
		*/
		
		
	}

	//println!("raw_sum = {}", raw_sum as uint);

	//println!("Average neurons_column::target_cols: (RAW) = {}.", raw_sum as uint / target_cols.vec.len());
	//println!("Average neurons_column::target_cols: (Norm) = {}.", norm_sum as uint / target_cols.vec.len());

	let rng_range = Range::new(0u8, 255u8);

	for i in range(0u, target_col_synapses.vec.len()) {
		target_col_synapses.vec[i] = num::cast(rng_range.ind_sample(&mut rng)).unwrap();
	}
	
	target_cols.write();
	target_col_synapses.write();

}


pub struct Dendrites {
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub thresholds: CorticalComponent<ocl::cl_uchar>,
}
impl Dendrites {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			values: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
			thresholds: CorticalComponent::<ocl::cl_uchar>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
		}
	}
}


pub struct Synapses {
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub strengths: CorticalComponent<ocl::cl_uchar>,
}
impl Synapses {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Synapses {
		Synapses {
			values: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
			strengths: CorticalComponent::<ocl::cl_uchar>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
		}
	}
}




// <T: Int + Num + Primitive + NumCast + Show + UnsignedInt + Default, U: num::NumCast>
