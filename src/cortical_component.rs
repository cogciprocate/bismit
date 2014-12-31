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
		CorticalComponent {
			vec: Vec::from_elem(common::HYPERCOLUMNS_PER_SEGMENT, init_val),
			buff: ptr::null_mut(),
			context: ocl.context,
			command_queue: ocl.command_queue,
		}
	}

	pub fn init(&mut self) {
		self.buff = ocl::new_write_buffer(&self.vec, self.context);
		ocl::enqueue_write_buffer(&self.vec, self.buff, self.command_queue);
	}

	pub fn release(&mut self) {
		ocl::release_mem_object(self.buff);
	}
}


