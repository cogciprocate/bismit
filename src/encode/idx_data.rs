use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::iter;

pub struct IdxData {
	file_path: String,
	// file_reader: BufReader<File>,
	data: Vec<u8>,
	dims: Vec<usize>,
}

impl IdxData {
	pub fn new(file_name: &str) -> IdxData {
		// let assets = Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
		// let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");		
		let path_string = format!("{}/{}/{}", env!("P"), "bismit", file_name);
		let path = Path::new(&path_string);
		let display = path.display();

		let file = match File::open(&path) {
			Err(why) => panic!("\ncouldn't open '{}': {}", display, Error::description(&why)),
			Ok(file) => file,
		};

		let mut reader = BufReader::new(file);
		let mut header_magic: Vec<u8> = iter::repeat(0).take(4).collect();

		match reader.read(&mut header_magic[..]) {
		    Err(why) => panic!("\ncouldn't read '{}': {}", display, Error::description(&why)),
		    Ok(bytes) => (), //println!("{} contains:\n{:?}\n{} bytes read.", display, header_magic, bytes),
		}

		let magic_data_type = header_magic[2];
		let magic_dims = header_magic[3] as usize;
		assert!(magic_data_type == 8, format!("IDX file: '{}' does not contain unsigned bytes.", display));

		let mut header_dim_sizes_bytes: Vec<u8> = iter::repeat(0).take(magic_dims * 4).collect();

		match reader.read(&mut header_dim_sizes_bytes[..]) {
		    Err(why) => panic!("\ncouldn't read '{}': {}", display, Error::description(&why)),
		    Ok(bytes) => (), //println!("{} contains:\n{:?}\n{} bytes read.", display, header_dim_sizes_bytes, bytes),
		}
		
		let ttl_header_len = 4 + (magic_dims * 4);
		let mut dim_sizes: Vec<usize> = iter::repeat(0).take(magic_dims).collect();

		for i in 0..magic_dims {
			let header_ofs = 4 * i;
			dim_sizes[i] = 
				(header_dim_sizes_bytes[header_ofs] as usize) << 24 
				| (header_dim_sizes_bytes[header_ofs + 1] as usize) << 16 
				| (header_dim_sizes_bytes[header_ofs + 2] as usize) << 8 
				| (header_dim_sizes_bytes[header_ofs + 3] as usize)
			;
		}

    	let mut buffer_cap: usize = 1;

    	for &size in &dim_sizes {
    		buffer_cap *= size as usize;
		}

    	let mut idx_buffer: Vec<u8> = Vec::with_capacity(buffer_cap);
    	
    	// TODO: CONVERT TO STREAM
    	match reader.read_to_end(&mut idx_buffer) {
    		Err(why) => panic!("\ncouldn't read '{}': {}", &path_string, Error::description(&why)),
		    Ok(bytes) => println!("{}: {} bytes read.", display, bytes),
		}

		println!("IDXREADER: initialized with dimensions: {:?}", dim_sizes);

	    IdxData {	    	  	    	
	    	file_path: format!("{}", path.display()),
	    	// file_reader: reader,
	    	data: idx_buffer,
	    	dims: dim_sizes, 
    	}
	}

	pub fn data(&self) -> &[u8] {
		&self.data[..]
	}

	pub fn dims(&self) -> &[usize] {
		&self.dims[..]
	}
}



// THE IDX FILE FORMAT

// the IDX file format is a simple format for vectors and multidimensional matrices of various numerical types.
// The basic format is

// magic number 
// size in dimension 0 
// size in dimension 1 
// size in dimension 2 
// ..... 
// size in dimension N 
// data

// The magic number is an integer (MSB first). The first 2 bytes are always 0.

// The third byte codes the type of the data: 
// 0x08: unsigned byte 
// 0x09: signed byte 
// 0x0B: short (2 bytes) 
// 0x0C: int (4 bytes) 
// 0x0D: float (4 bytes) 
// 0x0E: double (8 bytes)

// The 4-th byte codes the number of dimensions of the vector/matrix: 1 for vectors, 2 for matrices....

// The sizes in each dimension are 4-byte integers (MSB first, high endian, like in most non-Intel processors).

// The data is stored like in a C array, i.e. the index in the last dimension changes the fastest. 
  
// Happy hacking.

