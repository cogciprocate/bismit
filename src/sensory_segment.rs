extern crate libc;

use ocl;
use common;
use cortical_component::{ CorticalComponent };
use column;
use chord::{ Chord };
use cortical_segment::{ CorticalSegment };
	//	use column_neurons;
	//	use neurons_cell;
	//	use std;
	//	use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};
	//	use std::ptr;
use std::num;
use std::mem;
use std::num::{ Int };

	//	use time;

pub struct SensorySegment {
		//	pub target_addresses: CorticalComponent<ocl::cl_ushort>,
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub target_addresses: column::Axons,
	pub target_segment_idx: usize,
	pub context: ocl::cl_context,
	pub command_queue: ocl::cl_command_queue,
	pub sense_kernel: ocl::cl_kernel,

	pub tmp_out: CorticalComponent<ocl::cl_uint>,

	
		//	pub temp_target_segment.columns.synapses_vec: Vec<ocl::cl_uint>,
		//	pub temp_target_segment.columns.synapses_buf: ocl::cl_mem,
	

		//	pub target_segment.columns.synapses_buf: ocl::cl_mem,
}
impl SensorySegment {
	pub fn new(width: usize, target_segment: &CorticalSegment, target_segment_idx: usize, ocl: &ocl::Ocl) -> SensorySegment {
		
		//	let mut target_addresses = CorticalComponent::<ocl::cl_ushort>::new(width, 0u16, ocl);
		//	init_ss_target_addresses(&mut target_addresses, tar_syn);
		let source_values = CorticalComponent::<ocl::cl_uchar>::new(width, 0u8, ocl);



		//	println!("****** Creating SensorySegment Axon ******");
		let target_addresses = column::Axons::new(common::COLUMN_SYNAPSES_PER_SEGMENT, ocl);

		/*
			println!("Printing target_addresses.target_column_synapses.vec (len: {}):", target_addresses.target_column_synapses.len());
			common::print_vec(&target_addresses.target_column_synapses.vec, 10000);
		*/



		let kern = ocl::new_kernel(ocl.program, "sense");
		let dup_factor: ocl::cl_uint = num::FromPrimitive::from_uint(target_segment.columns.synapses.values.len() / source_values.len()).unwrap();


		//	println!("dup_factor: {:b}", dup_factor);
		//	println!("dup_factor.leading_zeros(): {}", dup_factor.leading_zeros());
		//	println!("dup_factor.dup_factor.trailing_zeros(): {}", dup_factor.trailing_zeros());


		let dup_factor_shift: ocl::cl_uchar = common::int_log2(dup_factor);
		
		//	println!("*** dup_factor_shift: {}", dup_factor_shift);

		//	println!("dfs: {:b}", dup_factor_shift);

		let tmp_out = CorticalComponent::new(common::COLUMN_SYNAPSES_PER_SEGMENT, 0u32, ocl);

	
		ocl::set_kernel_arg(0, source_values.buf, kern);
		ocl::set_kernel_arg(1, target_segment.columns.synapses.values.buf, kern);
		ocl::set_kernel_arg(2, target_addresses.target_column_bodies.buf, kern);
		ocl::set_kernel_arg(3, target_addresses.target_column_synapses.buf, kern);
		ocl::set_kernel_arg(4, dup_factor_shift, kern);
		ocl::set_kernel_arg(5, tmp_out.buf, kern);

		//target_addresses.target_column_bodies.print(1000);
		//target_addresses.target_column_synapses.print(1000);		

		//tmp_out.print(1);
		
		/*
			println!("ocl::set_kernel_arg(0, values.buf, kern) -- buffer len: {}", source_values.len());
			println!("ocl::set_kernel_arg(1, target_segment.columns.synapses.values.buf, kern) -- buffer len: {}", target_segment.columns.synapses.values.len());
			println!("ocl::set_kernel_arg(2, target_addresses.target_column_bodies.buf, kern) -- buffer len: {}", target_addresses.target_column_bodies.len());
			println!("ocl::set_kernel_arg(3, target_addresses.target_column_synapses.buf, kern) -- buffer len: {}", target_addresses.target_column_synapses.len());
		*/

		//	ocl::set_kernel_arg(4, temp_target_segment.columns.synapses_buf, kern);
		//	println!("ocl::set_kernel_arg(4, temp_target_segment.columns.synapses_buf, kern) -- buffer len: {}", temp_target_segment.columns.synapses_len());
		

		//	common::print_vec(&target_addresses.target_column_bodies.vec, 1);
		
		

		SensorySegment { 
			//	target_addresses : target_addresses,
			target_addresses: target_addresses,
			target_segment_idx: target_segment_idx,
			values: source_values,
			context: ocl.context,
			command_queue: ocl.command_queue,
			sense_kernel: kern,

			tmp_out: tmp_out,

			
			//	temp_target_segment.columns.synapses_vec: temp_target_segment.columns.synapses_vec,
			//	temp_target_segment.columns.synapses_buf: temp_target_synapses_buf,
			
		}
	}

	pub fn sense(&mut self, chord: &Chord, target_segment: &CorticalSegment) {
		chord.unfold_into(&mut self.values.vec);
		self.values.write();


		//	println!("[[[printing sensory_segment.values; local:");
		//	common::print_vec(&self.values.vec, 10000);
		//	println!("*** printing sensory_segment.values; remote: [[[");
		//	self.values.print();
		//	println!("]]]");

		let csps = common::COLUMN_SYNAPSES_PER_SEGMENT;

		
		//	println!("[SensorySegment::sense(): enqueuing kernel with {} work items...]", csps);

		ocl::enqueue_kernel(self.sense_kernel, self.command_queue, csps);

		//target_segment.cycle();

		/*
			let mut temp_target_segment.columns.synapses_vec = iter::repeat(0).take(common::COLUMN_SYNAPSES_PER_SEGMENT).collect();
			let temp_target_segment.columns.synapses_buf = ocl::new_read_buffer(&mut temp_target_segment.columns.synapses_vec, ocl.context);
			ocl::enqueue_read_buffer(&mut self.temp_target_segment.columns.synapses_vec, self.temp_target_segment.columns.synapses_buf, self.command_queue);
			common::print_vec(&self.temp_target_segment.columns.synapses_vec, 1);
		*/

	}

}

/*
	pub fn init_ss_target_addresses(target_addresses: &mut CorticalComponent<u16>, tar_syn: &Synapses) {
		let mut rng = rand::task_rng();
		let rng_range = Range::new(0u, );

		for tar in target_addresses.vec.iter_mut() {
			*tar = rng_range.ind_sample(&mut rng) as u16;
		}
		target_addresses.write();
	}
*/
