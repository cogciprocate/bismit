use ocl;
use common;

use std::ptr;
use std::iter;
use std::num::{ Int, NumCast, FromPrimitive };
use std::fmt::{ Display };
use std::default::{ Default };

pub struct CorticalComponent<T> {
	pub vec: Vec<T>,
	pub buf: ocl::cl_mem,
	//pub context: ocl::cl_context,
	//pub command_queue: ocl::cl_command_queue,
	pub ocl: Box<ocl::Ocl>,
}
impl<T: Clone + NumCast + Int + Default + Display + FromPrimitive> CorticalComponent<T> {
	pub fn new(size: usize, init_val: T, ocl: &ocl::Ocl) -> CorticalComponent<T> {
		let vec: Vec<T> = iter::repeat(init_val).take(size).collect();
		CorticalComponent::_new(vec, ocl)
	}

	pub fn shuffled(size: usize, init_val: T, ocl: &ocl::Ocl) -> CorticalComponent<T> {
		let vec: Vec<T> = common::shuffled_vec(size, init_val);
		CorticalComponent::_new(vec, ocl)
	}

	fn _new(mut vec:Vec<T>, ocl: &ocl::Ocl) -> CorticalComponent<T> {
		let buf: ocl::cl_mem = ocl::new_buffer(&mut vec, ocl.context);

		ocl::enqueue_write_buffer(&mut vec, buf, ocl.command_queue);

		CorticalComponent {
			vec: vec,
			buf: buf,
			//context: ocl.context,
			//command_queue: ocl.command_queue,
			ocl: Box::new(ocl.clone()),
		}
	}

	pub fn write(&mut self) {
		//println!("CorticalComponent.vec.len(): {}", self.vec.len());
		ocl::enqueue_write_buffer(&self.vec, self.buf, self.ocl.command_queue);
		
		//println!("CorticalComponent::write(): complete.");
	}

	pub fn len(&self) -> usize {
		self.vec.len()
	}

	pub fn print(&mut self, every: usize) {

		//let read_buf = ocl::new_read_buffer(&mut self.vec, self.ocl.context);
		//let kern = ocl::new_kernel(self.ocl.program, "read_char_array");

		//ocl::set_kernel_arg(0, self.buf, kern);
		//ocl::set_kernel_arg(1, read_buf, kern);

		//ocl::enqueue_kernel(kern, self.ocl.command_queue, self.vec.len());

		ocl::enqueue_read_buffer(&mut self.vec, self.buf, self.ocl.command_queue);

		//ocl::release_mem_object(read_buf);

		//println!("Printing Synapse Values...");

		common::print_vec(&self.vec, every, false);

		/*
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
		*/
    }

    pub fn release(&mut self) {
		ocl::release_mem_object(self.buf);
	}

}


