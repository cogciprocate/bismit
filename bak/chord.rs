use std::collections::BTreeMap;
//use std::option::Option; 
use std::fmt::{ Show, Formatter, Error };

pub struct Chord {
	pub chord: BTreeMap<u16, u8>,
}
impl Chord {
	pub fn new() -> Chord {
		Chord { chord: BTreeMap::new(), }
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
			cuf.vals[*k as uint] = *v;
		}
		cuf
	}

	pub fn print(&self) {
		println!("");
		for (k, v) in self.chord.iter() {
			print!("(addr:{}, val:{})", k, v);
		}
		println!("");
    }
}


pub struct ChordUnfolded {
	vals: [u8, ..1024],
}
impl ChordUnfolded {
	pub fn new() -> ChordUnfolded {
		ChordUnfolded { 
			vals: [0u8, ..1024],
		}
	}

	pub fn print(&self) {
		println!("");
		for i in range(0, self.vals.len()) {
			print!("([{}]:{})", i, self.vals[i]);
		}
		println!("");
    }
}
