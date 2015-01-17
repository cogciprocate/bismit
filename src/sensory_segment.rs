use ocl;
use common;
use cortical_component::{ CorticalComponent };
use column_neurons:: { Synapses, Axons };
use chord::{ Chord };
//use column_neurons;
//use neurons_cell;
//use std;
//use std::io;
use std::rand;
use std::rand::distributions::{IndependentSample, Range};
//use std::ptr;
use std::num;

//use time;

pub struct SensorySegment {
	//pub targets: CorticalComponent<ocl::cl_ushort>,
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub axons: Axons,
	pub context: ocl::cl_context,
	pub command_queue: ocl::cl_command_queue,
	pub sense_kernel: ocl::cl_kernel,

	/*
		// DEBUG
		pub temp_tar_syns_vec: Vec<ocl::cl_uint>,
		pub temp_tar_syns_buf: ocl::cl_mem,
		// END DEBUG
	*/

	//pub target_synapses_buf: ocl::cl_mem,
}
impl SensorySegment {
	pub fn new(width: usize, tar_syns: &Synapses, ocl: &ocl::Ocl) -> SensorySegment {
		
		//let mut targets = CorticalComponent::<ocl::cl_ushort>::new(width, 0u16, ocl);
		//init_ss_targets(&mut targets, tar_syn);
		let axons = Axons::new(common::COLUMN_SYNAPSES_PER_SEGMENT, ocl);

		let sense_kern_name = "sense";
		let values = CorticalComponent::<ocl::cl_uchar>::new(width, 0u8, ocl);
		let kern = ocl::new_kernel(ocl.program, sense_kern_name);

		ocl::set_kernel_arg(0, values.buf, kern);
		ocl::set_kernel_arg(1, tar_syns.values.buf, kern);
		ocl::set_kernel_arg(2, axons.target_columns.buf, kern);
		ocl::set_kernel_arg(3, axons.target_column_synapses.buf, kern);
		//ocl::set_kernel_arg(4, common::SYNAPSES_PER_NEURON, kern);

		/*
		// DEBUG
			println!("ocl::set_kernel_arg(0, values.buf, kern) -- buffer len: {}", values.vec.len());
			println!("ocl::set_kernel_arg(1, tar_syns.values.buf, kern) -- buffer len: {}", tar_syns.values.vec.len());
			println!("ocl::set_kernel_arg(2, axons.target_columns.buf, kern) -- buffer len: {}", axons.target_columns.vec.len());
			println!("ocl::set_kernel_arg(3, axons.target_column_synapses.buf, kern) -- buffer len: {}", axons.target_column_synapses.vec.len());

			let mut temp_tar_syns_vec = Vec::from_elem(common::COLUMN_SYNAPSES_PER_SEGMENT, 0);
			let temp_tar_syns_buf = ocl::new_read_buffer(&mut temp_tar_syns_vec, ocl.context);

			ocl::set_kernel_arg(4, temp_tar_syns_buf, kern);
			println!("ocl::set_kernel_arg(4, temp_tar_syns_buf, kern) -- buffer len: {}", temp_tar_syns_vec.len());

			common::print_vec(&axons.target_columns.vec);
		// END DEBUG
		*/

		SensorySegment { 
			//targets : targets,
			axons: axons,
			values: values,
			context: ocl.context,
			command_queue: ocl.command_queue,
			sense_kernel: kern,

			/*
			// DEBUG
				temp_tar_syns_vec: temp_tar_syns_vec,
				temp_tar_syns_buf: temp_tar_syns_buf,
			// END DEBUG
			*/
		}
	}

	pub fn sense(&mut self, chord: &Chord) {
		chord.unfold_into(&mut self.values.vec);
		self.values.write();

		let wi = common::COLUMN_SYNAPSES_PER_SEGMENT;

		/*
		// DEBUG
			println!("[SensorySegment::sense(): enqueuing kernel with {} work items...]", wi);

			ocl::enqueue_kernel(self.sense_kernel, self.command_queue, wi);

			ocl::enqueue_read_buffer(&mut self.temp_tar_syns_vec, self.temp_tar_syns_buf, self.command_queue);

			common::print_vec(&self.temp_tar_syns_vec);
		// END DEBUG
		*/

	}
}

/*
pub fn init_ss_targets(targets: &mut CorticalComponent<u16>, tar_syn: &Synapses) {
	let mut rng = rand::task_rng();
	let rng_range = Range::new(0u, );

	for tar in targets.vec.iter_mut() {
		*tar = rng_range.ind_sample(&mut rng) as u16;
	}
	targets.write();
}
*/
