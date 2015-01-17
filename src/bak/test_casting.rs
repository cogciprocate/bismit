use std::num;
use std::num:: { SignedInt };

pub fn run() {
	let x = -50i;
	let y: uint = num::cast(x.abs()).unwrap();
	println!("({}) cast to uint = {}", x, y);
}
