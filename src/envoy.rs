use ocl;
use common;

use std::ptr;
use std::iter;
use std::num::{ Int, NumCast, FromPrimitive };
use std::fmt::{ Display };
use std::default::{ Default };

pub struct Envoy<T> {
	pub vec: Vec<T>,
	pub buf: ocl::cl_mem,
	pub ocl: Box<ocl::Ocl>,
}
impl<T: Clone + NumCast + Int + Default + Display + FromPrimitive> Envoy<T> {
	pub fn new(size: usize, init_val: T, ocl: &ocl::Ocl) -> Envoy<T> {
		let vec: Vec<T> = iter::repeat(init_val).take(size).collect();
		Envoy::_new(vec, ocl)
	}

	pub fn shuffled(size: usize, init_val: T, ocl: &ocl::Ocl) -> Envoy<T> {
		let vec: Vec<T> = common::shuffled_vec(size, init_val);
		Envoy::_new(vec, ocl)
	}

	fn _new(mut vec:Vec<T>, ocl: &ocl::Ocl) -> Envoy<T> {
		let buf: ocl::cl_mem = ocl::new_buffer(&mut vec, ocl.context);

		ocl::enqueue_write_buffer(&mut vec, buf, ocl.command_queue);

		Envoy {
			vec: vec,
			buf: buf,
			ocl: Box::new(ocl.clone()),
		}
	}

	pub fn write(&mut self) {
		ocl::enqueue_write_buffer(&self.vec, self.buf, self.ocl.command_queue);
	}

	pub fn read(&mut self) {
		ocl::enqueue_read_buffer(&mut self.vec, self.buf, self.ocl.command_queue);
	}

	pub fn len(&self) -> usize {
		self.vec.len()
	}

	pub fn print(&mut self, every: usize) {
		self.read();
		common::print_vec(&self.vec, every, false);
    }

    pub fn release(&mut self) {
		ocl::release_mem_object(self.buf);
	}

}


