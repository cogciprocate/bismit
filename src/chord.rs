use std::collections::BTreeMap;
//use std::option::Option; 
use std::fmt::{ Formatter, Error };
use std::iter;
use common;
use ocl;

pub struct Chord {
	pub chord: BTreeMap<u16, ocl::cl_uchar>,
	pub width: u32,
}
impl Chord {
	pub fn new() -> Chord {
		Chord { 
			chord: BTreeMap::new(), 
			width: common::SENSORY_CHORD_WIDTH, 
		}
	}

	pub fn from_vec(vec: &Vec<ocl::cl_uchar>) -> Chord {
		let mut chord = BTreeMap::new();

		let mut i: u16 = 0;
		for x in vec.iter() {

			if *x != 0 {
				chord.insert(i, *x);
			}
			
			i += 1;
		}
		Chord { 
			chord: chord,
			width: common::SENSORY_CHORD_WIDTH,
		}
	}

	pub fn note_sum(&mut self, addr: u16, val: ocl::cl_uchar) {
		match self.chord.insert(addr, val) {
			Some(x) => {
				let sum_val = if (x / 2) + (val / 2) > 63 {
					127
				} else {
					x + val
				};
				self.chord.insert(addr, sum_val);
				()
			},
			None => (),
		};
	}

	pub fn note_gt(&mut self, addr: u16, val: ocl::cl_uchar) {
		match self.chord.insert(addr, val) {
			Some(x) => {
				let sum_val = if x > val {
					x
				} else {
					val
				};
				self.chord.insert(addr, sum_val);
				()
			},
			None => (),
		};
	}

	pub fn unfold(&self) -> Vec<ocl::cl_uchar> {
		//let mut vec = Vec::with_capacity(self.width);
		//let vec: Vec<ocl::cl_uchar> = Vec::with_capacity(self.width as usize);
		let mut vec: Vec<ocl::cl_uchar> = iter::repeat(common::STATE_ZERO).take(self.width as usize).collect();
		self.unfold_into(&mut vec, 0);
		vec
	}

	/*pub fn unfold(&self) -> ChordUnfolded {
		let mut cuf = ChordUnfolded::new();
		for (k, v) in self.chord.iter() {
			cuf.notes[*k as usize] = *v;
		}
		cuf
	}*/

	pub fn unfold_into(&self, vec: &mut Vec<ocl::cl_uchar>, offset: usize) {
		//vec.clear();
		assert!(vec.len() >= (self.width as usize + offset));

		for (k, v) in self.chord.iter() {
			vec[*k as usize + offset] = *v;
		}
	}

	/*pub fn unfold_into(&self, dest_vec: &mut Vec<ocl::cl_uchar>, offset: usize) {
		dest_vec.clear();
		dest_vec.push_all(&self.unfold().notes);		
	}*/

	pub fn print(&self) {
		let color = common::C_DEFAULT;
		for (k, v) in self.chord.iter() {
			print!("({}addr:{}, val:{}{})", color, k, v, common::C_DEFAULT);
		}
    }
}

/*
pub struct ChordUnfolded {
	pub notes: [ocl::cl_uchar; common::SENSORY_CHORD_WIDTH as usize],
}
impl ChordUnfolded {
	pub fn new() -> ChordUnfolded {
		ChordUnfolded { 
			notes: [0; common::SENSORY_CHORD_WIDTH as usize],
		}
	}

	pub fn print(&self) {
		println!("");
		let mut color: &'static str;
		for i in 0..self.notes.len() {
			if self.notes[i] != 0 {
				color = common::C_ORA;
			} else {
				color = common::C_DEFAULT;
			}
			print!("({}[{}]:{}{})", color, i, self.notes[i], common::C_DEFAULT);
		}
		println!("");
    }
}*/
