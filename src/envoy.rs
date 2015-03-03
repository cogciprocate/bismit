use ocl;
use common;

use std::ptr;
use std::iter;
use std::num::{ Int, NumCast, FromPrimitive };
use std::fmt::{ Display };
use std::default::{ Default };
use std::ops::{ self, Index, IndexMut };

pub struct Envoy<T> {
	pub vec: Vec<T>,
	pub buf: ocl::cl_mem,
	pub padding: u32,
	pub width: u32,
	pub height: u8,
	pub ocl: Box<ocl::Ocl>,
}
impl<T: Clone + NumCast + Int + Default + Display + FromPrimitive> Envoy<T> {
	pub fn new(width: u32, height: u8, init_val: T, ocl: &ocl::Ocl) -> Envoy<T> {
		let len = len(width, height, 0);
		let vec: Vec<T> = iter::repeat(init_val).take(len).collect();

		Envoy::_new(0, width, height, vec, ocl)
	}

	pub fn with_padding(padding: u32, width: u32, height: u8, init_val: T, ocl: &ocl::Ocl) -> Envoy<T> {
		let len = len(width, height, padding);
		let vec: Vec<T> = iter::repeat(init_val).take(len).collect();

		Envoy::_new(padding, width, height, vec, ocl)
	}

	pub fn shuffled(width: u32, height: u8, min_val: T, max_val: T, ocl: &ocl::Ocl) -> Envoy<T> {
		let len = len(width, height, 0);
		//println!("shuffled(): len: {}", len);
		let vec: Vec<T> = common::shuffled_vec(len, min_val, max_val);
		//println!("shuffled(): vec.len(): {}", vec.len());

		Envoy::_new(0, width, height, vec, ocl)
	}

	pub fn _new(padding: u32, width: u32, height: u8, mut vec: Vec<T>, ocl: &ocl::Ocl) -> Envoy<T> {
		let buf: ocl::cl_mem = ocl::new_buffer(&mut vec, ocl.context);

		//println!("New Envoy with height: {}, width: {}, padding: {}", height, width, padding);


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
		assert!(((self.width as usize * self.height as usize) + self.padding as usize ) == self.vec.len(), "envoy::Envoy::len(): Envoy len mismatch" );
		len(self.width, self.height, 0)
	}

	pub fn print(&mut self, every: usize) {
		self.read();
		common::print_vec(&self.vec, every, true, None);
    }

    pub fn print_val_range(&mut self, every: usize, low: T, high: T) {
    	let range: ops::Range<T> = ops::Range { start: low, end: high };
    	self.read();
		common::print_vec(&self.vec, every, true, Some(range));
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


fn len(width: u32, height: u8, padding: u32) -> usize {
	(width as usize * height as usize) + padding as usize
}
