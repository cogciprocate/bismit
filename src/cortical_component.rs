use ocl;
use common;

use std::ptr;
use std::iter;
use std::num::{ Int, NumCast };

pub struct CorticalComponent<T> {
	pub vec: Vec<T>,
	pub buf: ocl::cl_mem,
	pub context: ocl::cl_context,
	pub command_queue: ocl::cl_command_queue,
}
impl<T: Clone + NumCast> CorticalComponent<T> {
	pub fn new(size: usize, init_val: T, ocl: &ocl::Ocl) -> CorticalComponent<T> {
		//let mut vec: Vec<T> = Vec::from_elem(size, init_val);

		let mut vec: Vec<T> = iter::repeat(init_val).take(size).collect();
		let buf: ocl::cl_mem = ocl::new_write_buffer(&mut vec, ocl.context);

		ocl::enqueue_write_buffer(&mut vec, buf, ocl.command_queue);

		CorticalComponent {
			vec: vec,
			buf: buf,
			context: ocl.context,
			command_queue: ocl.command_queue,
		}
	}

	pub fn write(&mut self) {
		//println!("CorticalComponent.vec.len(): {}", self.vec.len());
		ocl::enqueue_write_buffer(&mut self.vec, self.buf, self.command_queue);
		//println!("CorticalComponent::write(): complete.");
	}

	pub fn release(&mut self) {
		ocl::release_mem_object(self.buf);
	}

}


