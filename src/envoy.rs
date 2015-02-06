use ocl;
use common;

use std::ptr;
use std::iter;
use std::num::{ Int, NumCast, FromPrimitive };
use std::fmt::{ Display };
use std::default::{ Default };
use std::ops::{ Index, IndexMut };

pub struct Envoy<T> {
	pub vec: Vec<T>,
	pub buf: ocl::cl_mem,
	pub padding: usize,
	pub width: u32,
	pub height: u8,
	pub ocl: Box<ocl::Ocl>,
}
impl<T: Clone + NumCast + Int + Default + Display + FromPrimitive> Envoy<T> {
	pub fn new(width: u32, height: u8, init_val: T, ocl: &ocl::Ocl) -> Envoy<T> {
		let len = len(width, height);
		let vec: Vec<T> = iter::repeat(init_val).take(len).collect();

		Envoy::_new(0, width, height, vec, ocl)
	}

	pub fn with_padding(padding: usize, width: u32, height: u8, init_val: T, ocl: &ocl::Ocl) -> Envoy<T> {
		let len = len(width, height) + padding;
		let vec: Vec<T> = iter::repeat(init_val).take(len).collect();

		Envoy::_new(padding, width, height, vec, ocl)
	}

	pub fn shuffled(width: u32, height: u8, init_val: T, ocl: &ocl::Ocl) -> Envoy<T> {
		let len = len(width, height);
		let vec: Vec<T> = common::shuffled_vec(len, init_val);

		Envoy::_new(0, width, height, vec, ocl)
	}

	pub fn _new(padding: usize, width: u32, height: u8, mut vec: Vec<T>, ocl: &ocl::Ocl) -> Envoy<T> {
		let buf: ocl::cl_mem = ocl::new_buffer(&mut vec, ocl.context);

		let envoy = Envoy {
			vec: vec,
			buf: buf,
			padding: padding,
			width: width,
			height: height,
			ocl: Box::new(ocl.clone()),
		};

		envoy.len();

		ocl.enqueue_write_buffer(&envoy);

		envoy
	}

	pub fn write(&mut self) {
		self.ocl.enqueue_write_buffer(self);
	}

	pub fn read(&mut self) {
		ocl::enqueue_read_buffer(&mut self.vec, self.buf, self.ocl.command_queue);
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn height(&self) -> u8 {
		self.height
	}

	pub fn len(&self) -> usize {
		assert!(((self.width as usize * self.height as usize) + self.padding ) == self.vec.len(), "Envoy len mismatch" );
		len(self.width, self.height)
	}

	pub fn print(&mut self, every: usize) {
		self.read();
		common::print_vec(&self.vec, every, true);
    }

    pub fn release(&mut self) {
		ocl::release_mem_object(self.buf);
	}

}

impl <T> Index<usize> for Envoy<T>
{
    type Output = T;

    fn index<'a>(&'a self, index: &usize) -> &'a T {
        &self.vec.as_slice()[*index]
    }
}

impl <T> IndexMut<usize> for Envoy<T>
{
    type Output = T;

    fn index_mut<'a>(&'a mut self, index: &usize) -> &'a mut T {
        &mut self.vec.as_mut_slice()[*index]
    }
}


fn len(width: u32, height: u8) -> usize {
	width as usize * height as usize
}
