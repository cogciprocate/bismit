use std::error::{ Error };
use std::fs::{ File };
use std::io::prelude::*;
use std::io::{ BufReader };
use std::path::{ Path };
use std::iter;

use ocl::{ CorticalDimensions };

// IDXREADER: Reads IDX files containing a series of two dimensional matrices of unsigned 
//		bytes (u8) into a ganglion (SDR frame buffer: &[u8])
pub struct IdxReader {
	ganglion_dims: CorticalDimensions,
	images_count: usize,
	image_height: usize, 
	image_width: usize,
	margins: Margins,
	image_counter: usize,
	file_path: String,
	file_reader: BufReader<File>,
	// len_file: usize,
	// len_image: usize,
	//dim_sizes: Vec<usize>,
}

impl IdxReader {
	pub fn new(ganglion_dims: CorticalDimensions, file_name: &str) -> IdxReader {
		let path_string = format!("{}/{}/{}", env!("P"), "bismit", file_name);
		let path = Path::new(&path_string);
		let display = path.display();

		let mut file = match File::open(&path) {
			Err(why) => panic!("\ncouldn't open '{}': {}", display, Error::description(&why)),
			Ok(file) => file,
		};

		let mut reader = BufReader::new(file);
		let mut header_magic: Vec<u8> = iter::repeat(0).take(4).collect();

		match reader.read(&mut header_magic[..]) {
		    Err(why) => panic!("\ncouldn't read '{}': {}", display, Error::description(&why)),
		    Ok(bytes) => (), //print!("\n{} contains:\n{:?}\n{} bytes read.", display, header_magic, bytes),
		}

		let magic_data_type = header_magic[2];
		let magic_dims = header_magic[3] as usize;
		assert!(magic_data_type == 8, format!("IDX file: '{}' does not contain unsigned bytes.", display));

		let mut header_dim_sizes_bytes: Vec<u8> = iter::repeat(0).take(magic_dims * 4).collect();

		match reader.read(&mut header_dim_sizes_bytes[..]) {
		    Err(why) => panic!("\ncouldn't read '{}': {}", display, Error::description(&why)),
		    Ok(bytes) => (), //print!("\n{} contains:\n{:?}\n{} bytes read.", display, header_dim_sizes_bytes, bytes),
		}
		
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

		let margins_horiz = ganglion_dims.u_size() as usize - dim_sizes[2];
		let margins_vert = ganglion_dims.v_size() as usize - dim_sizes[1];

		let margin_left = margins_horiz / 2;
    	let margin_right = margins_horiz - margin_left;
    	let margin_top = margins_vert / 2;
    	let margin_bottom = margins_vert - margin_top;

		print!("\ndim_sizes: {:?}", dim_sizes);

	    IdxReader {
	    	ganglion_dims: ganglion_dims,
	    	images_count: dim_sizes[0],
	    	image_height: dim_sizes[1],
	    	image_width: dim_sizes[2],
	    	margins: Margins { 
	    		left: margin_left, 
	    		right: margin_right, 
	    		top: margin_top,
	    		bottom: margin_bottom,
    		},
	    	image_counter: 0,
	    	//file: file,
	    	file_path: format!("{}", path.display()),
	    	file_reader: reader,
	    	//dim_sizes: dim_sizes,
    	}
    }

    // NEXT(): TODO - ROTATE IMAGE AND CORRECT ASPECT RATIO
    pub fn next(&mut self, ganglion_image: &mut [u8]) {
    	assert!((self.image_height * self.image_width) <= ganglion_image.len(), 
    		"Ganglion vector size must be greater than or equal to IDX image size");

    	assert!(ganglion_image.len() == self.ganglion_dims.columns() as usize);

    	



  //   	let mut ganglion_image_slice = &mut ganglion_image[margin..(margin + self.len_image)];
  
  //   	match self.file_reader.read(ganglion_image_slice) {
		//     Err(why) => panic!("\ncouldn't read '{}': {}", &self.file_path, Error::description(&why)),
		//     Ok(bytes) => (), //print!("\n{} contains:\n{:?}\n{} bytes read.", display, header_dim_sizes_bytes, bytes),
		// }
	}
}

struct Margins {
	left: usize,
	right: usize,
	top: usize,
	bottom: usize,
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
