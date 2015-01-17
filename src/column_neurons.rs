use common;
use ocl;
use cortical_component::{ CorticalComponent };

use std::num;
use std::rand;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::num::{ NumCast, UnsignedInt, Int };
use std::fmt::Show;
use std::default::{ Default };


pub struct Axons {
	pub target_columns: CorticalComponent<ocl::cl_ushort>,
	pub target_column_synapses: CorticalComponent<ocl::cl_uchar>,
}
impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {

		let mut target_columns = CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl);
		let mut target_column_synapses = CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl);

		Axons::init(&mut target_columns, &mut target_column_synapses, ocl);

		//println!("[Initializing Axons with size: {}]", size);

		Axons {
			target_columns: CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl),
			target_column_synapses: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
		}
	}

	pub fn init(
				target_cols: &mut CorticalComponent<ocl::cl_ushort>, 
				target_col_synapses: &mut CorticalComponent<ocl::cl_uchar>,
				ocl: &ocl::Ocl,

	) {
		let mut rng = rand::thread_rng();

		let rng_range = Range::new(0u16, (common::COLUMNS_PER_SEGMENT) as u16);

		for i in range(0, target_cols.vec.len()) {
			let rnum: u16 = rng_range.ind_sample(&mut rng);
			target_cols.vec[i] = rnum;
		}

		let rng_range = Range::new(0u8, 255u8);

		for i in range(0, target_col_synapses.vec.len()) {
			target_col_synapses.vec[i] = num::cast(rng_range.ind_sample(&mut rng)).unwrap();
		}

		// common::print_vec(&target_cols.vec);
		// common::print_vec(&target_col_synapses.vec);
		
		// BRING ME BACK 		target_cols.write();
		// BRING ME BACK 		target_col_synapses.write();

		ocl::enqueue_write_buffer(&mut target_cols.vec, target_cols.buf, ocl.command_queue);

		//ocl::new_kernel()

		//ocl::enqueue_kernel()

		// common::print_vec(&target_cols.vec);

	}
}



pub struct Dendrites {
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub thresholds: CorticalComponent<ocl::cl_uchar>,
}
impl Dendrites {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Dendrites {
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
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Synapses {
		Synapses {
			values: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
			strengths: CorticalComponent::<ocl::cl_uchar>::new(size, common::DENDRITE_INITIAL_THRESHOLD, ocl),
		}
	}


	pub fn print_values(&mut self, ocl: &ocl::Ocl) {

		let read_buf = ocl::new_read_buffer(&mut self.values.vec, ocl.context);
		let kern = ocl::new_kernel(ocl.program, "get_synapse_values");

		ocl::set_kernel_arg(0, self.values.buf, kern);
		ocl::set_kernel_arg(1, read_buf, kern);

		ocl::enqueue_kernel(kern, ocl.command_queue, self.values.vec.len());

		ocl::enqueue_read_buffer(&mut self.values.vec, read_buf, ocl.command_queue);

		ocl::release_mem_object(read_buf);

		println!("Printing Synapse Values...");
		let mut color: &'static str;
		for i in range(0, self.values.vec.len()) {
			if self.values.vec[i] != 0u8 {
				color = common::C_ORA;
				print!("({}[{}]:{}{})", color, i, self.values.vec[i], common::C_DEFAULT);
			} else {
				//color = common::C_DEFAULT;
			}
		}
		println!("");
    }
}




// <T: Int + Num + Primitive + NumCast + Show + UnsignedInt + Default, U: num::NumCast>
