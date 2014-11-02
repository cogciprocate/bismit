use std::io;
use std;

pub static INPUT_READ_OFFSET: u16= 0u16;
pub static INPUT_CHARS_TO_READ: u16 = 100u16;

pub fn ascii_sense() -> Box<Vec<Box<Vec<u16>>>> {
	let mut song: Vec<Box<Vec<u16>>> = Vec::new();
	println!("** Starting song.len(): {}",song.len());
	let input_file_name = "tale_pg98.txt";

	let input_file = match io::File::open(&Path::new(input_file_name)) {
		Ok(file) => file,
		Err(e) => fail!("error opening file: {}; {}", input_file_name, e),
	};

	let mut reader = std::io::BufferedReader::new(input_file);

	for x in range(INPUT_READ_OFFSET, INPUT_READ_OFFSET + INPUT_CHARS_TO_READ) {
		match reader.read_byte() {
			Ok(b) => {
				song.push(chord_encode_byte(b));
			},
			Err(e) => println!("Err: {}", e),
		}

		//song.push(x as u16);
		//println!("** Note {} = {}", x, chord[x as uint]);
	}
	//println!("** Characters Read: {}", song);
	println!("** Final song.len(): {}",song.len());
	box song
}

pub fn chord_encode_byte(byte: u8) -> Box<Vec<u16>> {
	let mut chord: Vec<u16> = Vec::with_capacity(256);

	for x in range(0u, 1023u) {
		chord.push((252u16 * byte as u16) + x as u16);
	}
	
	box chord
}


/*
pub fn test_sense() -> Box<Vec<u16>> {
	let mut vec: Vec<u16> = Vec::new();
	println!("** Starting vec.len(): {}",vec.len());

	for x in range(0u16, 8u16) {
		vec.push(x as u16);
		println!("** Note {} = {}", x, vec[x as uint]);
	}
	println!("** Final vec.len(): {}",vec.len());
	box vec
}
*/
