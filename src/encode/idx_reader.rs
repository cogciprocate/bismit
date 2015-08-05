use std::error::{ Error };
use std::fs::{ File };
use std::io::prelude::*;
use std::io::{ BufReader };
use std::path::{ Path };
use std::iter;
use num::{ Float };

use ocl::{ CorticalDimensions };

// IDXREADER: Reads IDX files containing a series of two dimensional matrices of unsigned 
//		bytes (u8) into a ganglion (SDR frame buffer: &[u8])
pub struct IdxReader {
	ganglion_dims: CorticalDimensions,
	images_count: usize,
	image_height: usize, 
	image_width: usize,
	image_len: usize,
	ttl_header_len: usize,
	margins: Margins, // DEPRICATE
	image_counter: usize,
	file_path: String,
	file_reader: BufReader<File>,
	image_buffer: Vec<u8>,
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

		let margins_horiz = ganglion_dims.u_size() as usize - dim_sizes[2];
		let margins_vert = ganglion_dims.v_size() as usize - dim_sizes[1];

		let margin_left = margins_horiz / 2;
    	let margin_right = margins_horiz - margin_left;
    	let margin_top = margins_vert / 2;
    	let margin_bottom = margins_vert - margin_top;

    	//let image_buffer: Vec<u8> = iter::repeat(0).take(dim_sizes[1] * dim_sizes[2]).collect();
    	let mut image_buffer: Vec<u8> = Vec::with_capacity(dim_sizes[0] * dim_sizes[1] * dim_sizes[2]);
    	
    	match reader.read_to_end(&mut image_buffer) {
    		Err(why) => panic!("\ncouldn't read '{}': {}", &path_string, Error::description(&why)),
		    Ok(bytes) => print!("\n{}: {} bytes read.", display, bytes),
		}

		print!("\nIDXREADER: initialized with dimensions: {:?}", dim_sizes);

	    IdxReader {
	    	ganglion_dims: ganglion_dims,
	    	images_count: dim_sizes[0],
	    	image_height: dim_sizes[1],
	    	image_width: dim_sizes[2],
	    	image_len: dim_sizes[1] * dim_sizes[2],
	    	ttl_header_len: ttl_header_len,
	    	margins: Margins { 	// DEPRICATE
	    		left: margin_left, 
	    		right: margin_right, 
	    		top: margin_top,
	    		bottom: margin_bottom,
    		},
	    	image_counter: 0,
	    	//file: file,
	    	file_path: format!("{}", path.display()),
	    	file_reader: reader,
	    	image_buffer: image_buffer,
	    	//dim_sizes: dim_sizes,
    	}
    }

    pub fn next(&mut self, ganglion_image: &mut [u8]) {
    	assert!(ganglion_image.len() == self.ganglion_dims.columns() as usize);
    	assert!((self.image_len) <= ganglion_image.len(), 
    		"Ganglion vector size must be greater than or equal to IDX image size");

  //   	match self.file_reader.read(&mut self.image_buffer[..]) {
		//     Err(why) => panic!("\ncouldn't read '{}': {}", &self.file_path, Error::description(&why)),
		//     Ok(bytes) => assert!(bytes == self.image_buffer.len(), "\n bytes read != buffer length"), 
		//     	//print!("\n{} contains:\n{:?}\n{} bytes read.", display, header_dim_sizes_bytes, bytes),
		// }

		let img_idz = self.image_counter * self.image_len;
		let img_idn = img_idz + self.image_len;

		self.image_pixel_to_hex(&self.image_buffer[img_idz..img_idn], ganglion_image);

		self.image_counter += 1;
	}


	pub fn image_pixel_to_hex(&self, source: &[u8], target: &mut [u8]) {
		let v_size = self.ganglion_dims.v_size() as usize;
		let u_size = self.ganglion_dims.u_size() as usize;

		for v_id in 0..v_size {
			for u_id in 0..u_size {
				let (x, y) = coord_hex_to_pixel(v_size, v_id, u_size, u_id, 
					self.image_height as usize, self.image_width as usize);
				
				let tar_idx = (v_id * u_size) + u_id;
				let src_idx = (y * self.image_width as usize) + x;

				target[tar_idx] = source[src_idx];
				//target[tar_idx] = (x != 0 || y != 0) as u8; // SHOW INPUT SQUARE
			}
		}
	}

	pub fn image_pixel_to_hex_crude(&self, source: &[u8], target: &mut [u8]) {
		for v in 0..self.image_height {
			for u in 0..self.image_width {
				let src_idx = (v * self.image_width as usize) + u;
				let tar_idx = ((v + self.margins.top as usize) * self.ganglion_dims.u_size() as usize) 
					+ (u + self.margins.left as usize);
				target[tar_idx] = source[src_idx];
			}
		}
	}
}


const HEX_SIDE: f64 = 0.5f64;
//const C1_OFS: f64 = 0f64 * HEX_SIDE;
//const C2_OFS: f64 = 0f64 * HEX_SIDE;	
//const V_OFS: f64 = 0f64;
//const W_OFS: f64 = 0f64;
const Y_OFS: f64 = 29f64 * HEX_SIDE;
const X_OFS: f64 = 43f64 * HEX_SIDE;

const SQRT_3: f64 = 1.73205080756f64;


// V_ID: Index of v ... implied to be inverted
// V: Geometric 

// COORD_HEX_TO_PIXEL(): Eventually either move this to GPU or at least use SIMD
pub fn coord_hex_to_pixel(v_size: usize, v_id: usize, u_size: usize, u_id: usize, 
				y_size: usize, x_size: usize,
) -> (usize, usize) {
	let u = u_id as f64;
	let u_inv = 0f64 - u;
	let v = v_id as f64;
	//let v_inv = 0f64 - v;
	//let w = u_inv + v_inv;
	let w_inv = v + u;
	//let s = HEX_SIDE;

	//let c1 = w_inv - C1_OFS;
	//let c2 = u_inv - C2_OFS;

	let mut x = w_inv * 1.5f64 * HEX_SIDE;
	let mut y = (u_inv + (w_inv / 2f64)) * SQRT_3 * HEX_SIDE;

	//let mut y = u * 1.5f64 * s;
	//let mut x = (v_inv + (u / 2f64)) * SQRT_3 * s;

	y += Y_OFS;
	x -= X_OFS;
	
	//x = x_size as f64 - x;
	//y = y_size as f64 - y;

	let valid = (y >= 0f64 && y < y_size as f64 && x >= 0f64 && x < x_size as f64) as usize;

	(x as usize * valid, y as usize * valid)
}


struct Margins {
	left: usize,
	right: usize,
	top: usize,
	bottom: usize,
}

// function hex_to_pixel(hex):
	//     x = size * 3/2 * hex.q
	//     y = size * sqrt(3) * (hex.r + hex.q/2)
	//     return Point(x, y)

// pub fn point_pixel_to_hex_incomplete(x_int: usize, y_int: usize) -> (usize, usize) {
// 	let sqrt3 = 3f64.sqrt();

// 	let edge_size = 0.5f64;
// 	let hex_width = edge_size * 2f64;
// 	let height = (sqrt3 / 2f64) * hex_width;

// 	let x = (x_int as f64 - edge_size) / hex_width;
// 	let y = y_int as f64;	

// 	(0, 0)
// }


// public Coord PointToCoord(double x, double z) {
// 	x = (x - halfHexWidth) / hexWidth;

// 	double t1 = z / hexRadius, t2 = Math.Floor(x + t1);
// 	double r = Math.Floor((Math.Floor(t1 - x) + t2) / 3); 
// 	double q = Math.Floor((Math.Floor(2 * x + 1) + t2) / 3) - r;

// 	return new Coord((int) q, (int) r); 
// }

// function pixel_to_hex(x, y):
//     q = x * 2/3 / size
//     r = (-x / 3 + sqrt(3)/3 * y) / size
//     return hex_round(Hex(q, r))


// function hex_to_pixel(hex):
//     x = size * 3/2 * hex.q
//     y = size * sqrt(3) * (hex.r + hex.q/2)
//     return Point(x, y)



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

