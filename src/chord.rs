use std::collections::BTreeMap;
//use std::option::Option; 
use std::fmt::{ Show, Formatter, Error };
use std::iter;
use common;

pub struct Chord {
	pub chord: BTreeMap<u16, u8>,
}
impl Chord {
	pub fn new() -> Chord {
		Chord { chord: BTreeMap::new(), }
	}

	pub fn from_vec(vec: &Vec<u8>) -> Chord {
		let mut chord = BTreeMap::new();

		let mut i: u16 = 0;
		for x in vec.iter() {

			if *x > 0 {
				chord.insert(i, *x);
			}
			
			i += 1;
		}
		Chord { chord: chord, }
	}

	pub fn note_sum(&mut self, addr: u16, val: u8) {
		match self.chord.insert(addr, val) {
			Some(x) => {
				let sum_val = if (x / 2) + (val / 2) > 127 {
					255
				} else {
					x + val
				};
				self.chord.insert(addr, sum_val);
				()
			},
			None => (),
		};
	}

	pub fn note_gt(&mut self, addr: u16, val: u8) {
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

	pub fn unfold(&self) -> ChordUnfolded {
		let mut cuf = ChordUnfolded::new();
		for (k, v) in self.chord.iter() {
			cuf.notes[*k as usize] = *v;
		}
		cuf
	}

	pub fn unfold_into(&self, dest_vec: &mut Vec<u8>) {
		dest_vec.clear();
		dest_vec.push_all(self.unfold().notes.as_slice());		
	}

	pub fn print(&self) {
		let color = common::C_DEFAULT;
		for (k, v) in self.chord.iter() {
			print!("({}addr:{}, val:{}{})", color, k, v, common::C_DEFAULT);
		}
    }
}


pub struct ChordUnfolded {
	pub notes: [u8; common::SENSORY_CHORD_WIDTH],
}
impl ChordUnfolded {
	pub fn new() -> ChordUnfolded {
		ChordUnfolded { 
			notes: [0u8; common::SENSORY_CHORD_WIDTH],
		}
	}

	pub fn print(&self) {
		println!("");
		let mut color: &'static str;
		for i in range(0, self.notes.len()) {
			if self.notes[i] != 0u8 {
				color = common::C_ORA;
			} else {
				color = common::C_DEFAULT;
			}
			print!("({}[{}]:{}{})", color, i, self.notes[i], common::C_DEFAULT);
		}
		println!("");
    }
}
