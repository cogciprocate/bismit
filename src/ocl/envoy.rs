// use std::ptr;
use std::iter::{ self };
//use std::num::{ NumCast, FromPrimitive, ToPrimitive };
use num::{ Integer, NumCast, FromPrimitive, ToPrimitive };
//use std::fmt::{ Display };
use std::fmt::{ Display, Debug, /*LowerHex,*/ UpperHex };
use std::default::{ Default };
use std::ops::{ /*self,*/ Index, IndexMut };

use ocl::{ self, OclProgQueue };
use cmn;

//pub trait NumCl: Integer + Copy + NumCast + Default + Display {}
//impl <T: NumCl> NumCl for T {}

pub trait EnvoyDimensions {
	fn len(&self) -> u32;
}

impl<'a, T> EnvoyDimensions for &'a T where T: EnvoyDimensions {
    fn len(&self) -> u32 { (*self).len() }
}

pub trait OclNum: Integer + Copy + Clone + NumCast + Default + Display + Debug
	+ FromPrimitive + ToPrimitive + UpperHex {}

impl<T> OclNum for T where T: Integer + Copy + Clone + NumCast + Default + Display + Debug
	+ FromPrimitive + ToPrimitive + UpperHex {}


pub type AxonState = Envoy<u8>;
pub type DendriteState = Envoy<u8>;
pub type SynapseState = Envoy<u8>;


pub struct Envoy<T> {
	vec: Vec<T>,
	buf: ocl::cl_mem,
	padding: u32,
	//dims: EnvoyDimensions,
	//pub width: u32,
	//pub depth: u8,
	ocl: OclProgQueue,
}

impl<T: OclNum> Envoy<T> {
	pub fn new<E: EnvoyDimensions>(dims: E, init_val: T, ocl: &OclProgQueue) -> Envoy<T> {
		let len = dims.len() as usize;
		let vec: Vec<T> = iter::repeat(init_val).take(len).collect();

		Envoy::_new(0, dims, vec, ocl)
	}

	pub fn with_padding<E: EnvoyDimensions>(dims: E, init_val: T, ocl: &OclProgQueue, padding: u32) -> Envoy<T> {
		let len = (dims.len() + padding) as usize;
		let vec: Vec<T> = iter::repeat(init_val).take(len).collect();

		Envoy::_new(padding, dims, vec, ocl)
	}

	// SHUFFLED(): max_val is inclusive!
	pub fn shuffled<E: EnvoyDimensions>(dims: E, min_val: T, max_val: T, ocl: &OclProgQueue) -> Envoy<T> {
		let len = dims.len() as usize;
		//println!("shuffled(): len: {}", len);
		let vec: Vec<T> = cmn::shuffled_vec(len, min_val, max_val);
		//println!("shuffled(): vec.len(): {}", vec.len());

		Envoy::_new(0, dims, vec, ocl)
	}

	fn _new<E: EnvoyDimensions>(padding: u32, dims: E, mut vec: Vec<T>, ocl: &OclProgQueue) -> Envoy<T> {
		//println!("New Envoy with depth: {}, width: {}, padding: {}", depth, width, padding);

		let buf: ocl::cl_mem = ocl::new_buffer(&mut vec, ocl.context());

		let mut envoy = Envoy {
			vec: vec,
			buf: buf,
			padding: padding,
			//width: width,
			//depth: depth,
			//dims: dims.clone(),
			ocl: ocl.clone(),
		};


		envoy.len();

		envoy.write();

		envoy
	}

	pub fn write(&mut self) {
		self.ocl.enqueue_write_buffer(self);
	}

	pub fn write_direct(&self, sdr: &[T], offset: usize) {
		ocl::enqueue_write_buffer(sdr, self.buf, self.ocl.queue(), offset);
	}

	pub fn read(&mut self) {
		ocl::enqueue_read_buffer(&mut self.vec, self.buf, self.ocl.queue(), 0);
	}

	pub fn read_direct(&self, sdr: &mut [T], offset: usize) {
		ocl::enqueue_read_buffer(sdr, self.buf, self.ocl.queue(), offset);
	}

	pub fn set_all_to(&mut self, val: T) {
		for ele in self.vec.iter_mut() {
			*ele = val;
		}
		self.write();
	}

	pub fn len(&self) -> usize {
		self.vec.len()
	}

	pub fn print_simple(&mut self) {
		self.read();
		cmn::print_vec_simple(&self.vec[..]);
    }

    pub fn print_val_range(&mut self, every: usize, val_range: Option<(T, T)>,) {
    	self.read();
		cmn::print_vec(&self.vec[..], every, val_range, None, true);
    }

    pub fn print(&mut self, every: usize, val_range: Option<(T, T)>, idx_range: Option<(usize, usize)>, zeros: bool) {
    	self.read();
		cmn::print_vec(&self.vec[..], every, val_range, idx_range, zeros);
	}

    pub fn release(&mut self) {
		ocl::release_mem_object(self.buf);
	}

	pub fn vec(&self) -> &Vec<T> {
		&self.vec
	}

	pub fn vec_mut(&mut self) -> &mut Vec<T> {
		&mut self.vec
	}

	pub fn buf(&self) -> ocl::cl_mem {
		self.buf
	}
}

impl<'b, T> Index<&'b usize> for Envoy<T> {
    type Output = T;

    fn index<'a>(&'a self, index: &'b usize) -> &'a T {
        &self.vec[..][*index]
    }
}

impl<'b, T> IndexMut<&'b usize> for Envoy<T> {
    fn index_mut<'a>(&'a mut self, index: &'b usize) -> &'a mut T {
        &mut self.vec[..][*index]
    }
}

impl<T> Index<usize> for Envoy<T> {
    type Output = T;

    fn index<'a>(&'a self, index: usize) -> &'a T {
        &self.vec[..][index]
    }
}

impl<T> IndexMut<usize> for Envoy<T> {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut T {
        &mut self.vec[..][index]
    }
}

/*
fn len(dims: E, padding: u32) -> usize {
	(padding + dims.len()) as usize
}*/
