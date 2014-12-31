use std::num;
use std::num:: { SignedInt };

pub fn run() {
	let x = -50i;
	let y: uint = num::cast(x.abs()).unwrap();
	println!("(-50) cast to uint = {}", y);
}
