use num;
use num:: { Integer, Signed };

pub fn run() {
	let x = -50i;
	let y: uint = num::cast(x.abs()).unwrap();
	println!("({}) cast to uint = {}", x, y);
}
