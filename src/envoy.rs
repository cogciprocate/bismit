use ocl::{ self, Ocl };
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
	pub depth: u8,
	pub ocl: Ocl,
}
impl<T: Int + Default + Display + FromPrimitive> Envoy<T> {
	pub fn new(width: u32, depth: u8, init_val: T, ocl: &Ocl) -> Envoy<T> {
		let len = len(width, depth, 0);
		let vec: Vec<T> = iter::repeat(init_val).take(len).collect();

		Envoy::_new(0, width, depth, vec, ocl)
	}

	pub fn with_padding(padding: u32, width: u32, depth: u8, init_val: T, ocl: &Ocl) -> Envoy<T> {
		let len = len(width, depth, padding);
		let vec: Vec<T> = iter::repeat(init_val).take(len).collect();

		Envoy::_new(padding, width, depth, vec, ocl)
	}

	pub fn shuffled(width: u32, depth: u8, min_val: T, max_val: T, ocl: &Ocl) -> Envoy<T> {
		let len = len(width, depth, 0);
		//println!("shuffled(): len: {}", len);
		let vec: Vec<T> = common::shuffled_vec(len, min_val, max_val);
		//println!("shuffled(): vec.len(): {}", vec.len());

		Envoy::_new(0, width, depth, vec, ocl)
	}

	pub fn _new(padding: u32, width: u32, depth: u8, mut vec: Vec<T>, ocl: &Ocl) -> Envoy<T> {
		//println!("New Envoy with depth: {}, width: {}, padding: {}", depth, width, padding);

		let buf: ocl::cl_mem = ocl::new_buffer(&mut vec, ocl.context);

		let mut envoy = Envoy {
			vec: vec,
			buf: buf,
			padding: padding,
			width: width,
			depth: depth,
			ocl: ocl.clone(),
		};


		envoy.len();

		envoy.write();

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

	pub fn depth(&self) -> u8 {
		self.depth
	}

	pub fn len(&self) -> usize {
		assert!(((self.width as usize * self.depth as usize) + self.padding as usize ) == self.vec.len(), "envoy::Envoy::len(): Envoy len mismatch" );
		len(self.width, self.depth, 0)
	}

	pub fn print_simple(&mut self) {
		self.read();
		common::print_vec_simple(&self.vec);
    }

    pub fn print_val_range(&mut self, every: usize, val_range: Option<(T, T)>,) {
    	self.read();
		common::print_vec(&self.vec, every, true, val_range, None);
    }

    pub fn print(&mut self, every: usize, val_range: Option<(T, T)>, idx_range: Option<(usize, usize)>) {
    	self.read();
		common::print_vec(&self.vec, every, true, val_range, idx_range);
	}

    pub fn release(&mut self) {
		ocl::release_mem_object(self.buf);
	}

}

impl<'b, T> Index<&'b usize> for Envoy<T>
{
    type Output = T;

    fn index<'a>(&'a self, index: &'b usize) -> &'a T {
        &self.vec.as_slice()[*index]
    }
}

impl<'b, T> IndexMut<&'b usize> for Envoy<T>
{

    fn index_mut<'a>(&'a mut self, index: &'b usize) -> &'a mut T {
        &mut self.vec.as_mut_slice()[*index]
    }
}


fn len(width: u32, depth: u8, padding: u32) -> usize {
	(width as usize * depth as usize) + padding as usize
}
