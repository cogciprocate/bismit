use std::io::{ BufReader };
use std::fs::{ File };
use std;

pub const INPUT_READ_OFFSET: u16= 0u16;
pub const INPUT_CHARS_TO_READ: u16 = 100u16;

pub const INPUT_FILE_NAME: &'static str = "/home/nick/dev/src/github.com/cogciprocate/bismit/data/tale_pg98.txt";

pub fn ascii_sense() -> Box<Vec<Box<Vec<u16>>>> {
	let mut song: Vec<Box<Vec<u16>>> = Vec::new();
	println!("** Starting song.len(): {}",song.len());

	let input_file = match File::open(&std::path::Path::new(INPUT_FILE_NAME)) {
		Ok(file) => file,
		Err(e) => panic!("error opening file: {}; {}", INPUT_FILE_NAME, e),
	};

	let mut reader = BufReader::new(input_file);

	/*for x in range(INPUT_READ_OFFSET, INPUT_READ_OFFSET + INPUT_CHARS_TO_READ) {
		match reader.read_u8() {
			Ok(b) => {
				song.push(chord_encode_byte(b));
			},
			Err(e) => println!("Err: {}", e),
		}

	}*/
	//println!("** Characters Read: {}", song);
	println!("** Final song.len(): {}",song.len());
	Box::new(song)
}

pub fn chord_encode_byte(byte: u8) -> Box<Vec<u16>> {
	let mut chord: Vec<u16> = Vec::with_capacity(256);

	for x in 0..1023 {
		chord.push((252u16 * byte as u16) + x as u16);
	}
	
	Box::new(chord)
}
