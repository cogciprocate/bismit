use ocl;
use common;

use std::ptr;

pub struct CorticalComponent<T> {
	pub vec: Vec<T>,
	pub buff: ocl::cl_mem,
	pub context: ocl::cl_context,
	pub command_queue: ocl::cl_command_queue,
}
impl <T> CorticalComponent<T> {
	pub fn new<T: Clone>(size: uint, init_val: T, ocl: &ocl::Ocl) -> CorticalComponent<T> {
		let vec: Vec<T> = Vec::from_elem(size, init_val);
		let buff: ocl::cl_mem = ocl::new_write_buffer(&vec, ocl.context);
		ocl::enqueue_write_buffer(&vec, buff, ocl.command_queue);

		CorticalComponent {
			vec: vec,
			buff: buff,
			context: ocl.context,
			command_queue: ocl.command_queue,
		}
	}

	pub fn write(&self) {
		ocl::enqueue_write_buffer(&self.vec, self.buff, self.command_queue);
	}

	pub fn release(&mut self) {
		ocl::release_mem_object(self.buff);
	}
}


