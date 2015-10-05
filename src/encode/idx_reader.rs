use std::error::{ Error };
use std::fs::{ File };
use std::io::prelude::*;
use std::io::{ BufReader };
use std::path::{ Path };
use std::iter;
// use num::{ Float };

use cmn::{ CorticalDimensions, Sdr };


// const HEX_SIDE: f64 = 0.5f64;
// //const C1_OFS: f64 = 0f64 * HEX_SIDE;
// //const C2_OFS: f64 = 0f64 * HEX_SIDE;	
// //const V_OFS: f64 = 0f64;
// //const W_OFS: f64 = 0f64;
// const Y_OFS: f64 = 29f64 * HEX_SIDE;
// const X_OFS: f64 = 43f64 * HEX_SIDE;

const SQRT_3: f64 = 1.73205080756f64;


//	IDXREADER: Reads IDX files containing a series of two dimensional matrices of unsigned 
//	bytes (u8) into a ganglion (SDR frame buffer: &Sdr)
//		- TODO: CONVERT FROM STORING FILE IN MEMORY TO STREAMING FILE (WITH LARGE BUFFER)
//			- TEST DIFFERENT BUFFERING STRATEGIES (see notes)
pub struct IdxReader {
	ganglion_dims: CorticalDimensions,
	repeats_per_image: usize,
	scale_factor: f64,
	repeat_counter: usize,
	frame_counter: usize,
	frames_count: usize,
	image_dim_count: usize,
	image_width: usize,
	image_height: usize,	
	image_len: usize,
	ttl_header_len: usize,
	margins: Margins, // DEPRICATE
	file_path: String,
	file_reader: BufReader<File>,
	image_buffer: Vec<u8>,
	// len_file: usize,
	// len_image: usize,
	//dim_sizes: Vec<usize>,
}

impl IdxReader {
	pub fn new(ganglion_dims: CorticalDimensions, file_name: &str, repeats_per_image: usize, scale_factor: f64) -> IdxReader {
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

		let image_width = if magic_dims > 1 { dim_sizes[1] } else { 1 };
		let image_height = if magic_dims > 2 { dim_sizes[2] } else { 1 };

		let margins_horiz = ganglion_dims.u_size() as usize - image_width;
		let margins_vert = ganglion_dims.v_size() as usize - image_height;

		let margin_left = margins_horiz / 2;
    	let margin_right = margins_horiz - margin_left;
    	let margin_top = margins_vert / 2;
    	let margin_bottom = margins_vert - margin_top;

    	//let image_buffer: Vec<u8> = iter::repeat(0).take(dim_sizes[1] * dim_sizes[2]).collect();
    	let mut buffer_cap: usize = 1;

    	for &size in &dim_sizes {
    		buffer_cap *= size as usize;
		}

    	let mut image_buffer: Vec<u8> = Vec::with_capacity(buffer_cap);
    	
    	// TODO: CONVERT TO STREAM
    	match reader.read_to_end(&mut image_buffer) {
    		Err(why) => panic!("\ncouldn't read '{}': {}", &path_string, Error::description(&why)),
		    Ok(bytes) => println!("{}: {} bytes read.", display, bytes),
		}

		println!("IDXREADER: initialized with dimensions: {:?}", dim_sizes);

	    IdxReader {
	    	ganglion_dims: ganglion_dims,
	    	repeats_per_image: repeats_per_image,
	    	scale_factor: scale_factor,
	    	repeat_counter: 0,
	    	frame_counter: 0,
	    	frames_count: dim_sizes[0],
	    	image_dim_count: magic_dims,
	    	image_width: image_width,
	    	image_height: image_height,	    	
	    	image_len: image_width * image_height,
	    	ttl_header_len: ttl_header_len,
	    	margins: Margins { 	// DEPRICATE
	    		left: margin_left, 
	    		right: margin_right, 
	    		top: margin_top,
	    		bottom: margin_bottom,
    		},	    	
	    	//file: file,
	    	file_path: format!("{}", path.display()),
	    	file_reader: reader,
	    	image_buffer: image_buffer,
	    	//dim_sizes: dim_sizes,
    	}
    }

    pub fn next(&mut self, ganglion_frame: &mut Sdr) -> usize {
    	assert!(ganglion_frame.len() == self.ganglion_dims.columns() as usize);
    	assert!((self.image_len) <= ganglion_frame.len(), 
    		"Ganglion vector size must be greater than or equal to IDX image size");    	

  		//   	match self.file_reader.read(&mut self.image_buffer[..]) {
		//     Err(why) => panic!("\ncouldn't read '{}': {}", &self.file_path, Error::description(&why)),
		//     Ok(bytes) => assert!(bytes == self.image_buffer.len(), "\n bytes read != buffer length"), 
		//     	//println!("{} contains:\n{:?}\n{} bytes read.", display, header_dim_sizes_bytes, bytes),
		// }

		let img_idz = self.frame_counter * self.image_len;
		let img_idn = img_idz + self.image_len;

		match self.image_dim_count {
			3 => self.encode_2d_image(&self.image_buffer[img_idz..img_idn], ganglion_frame),
			2 => panic!("\nOne dimensional (linear) idx images not yet supported."),
			1 => self.encode_scalar(&self.image_buffer[img_idz..img_idn], ganglion_frame),
			_ => panic!("\nIdx files with more than three or less than one dimension(s) not supported."),
		}

		let prev_frame = self.frame_counter;
		self.increment_frame();
		return prev_frame;
	}

	pub fn get_raw_frame(&self, frame_idx: usize, ganglion_frame: &mut Sdr) -> usize {
		assert!(ganglion_frame.len() == self.ganglion_dims.columns() as usize);
		assert!(frame_idx < self.frames_count);
		//let mut bytes_copied = 0;

		let img_idz = frame_idx * self.image_len;
		//let img_idn = img_idz + self.image_len;

		for idx in 0..self.image_len {
			ganglion_frame[idx] = self.image_buffer[img_idz + idx];
		}

		return self.image_len;
	}

	pub fn get_first_byte(&self, frame_idx: usize) -> u8 {
		assert!(frame_idx < self.frames_count);
		let img_idz = frame_idx * self.image_len;

		return self.image_buffer[img_idz];

	}

	fn increment_frame(&mut self) {		
		self.repeat_counter += 1;

		if self.repeat_counter >= self.repeats_per_image {
			self.repeat_counter = 0;
			self.frame_counter += 1;

			if self.frame_counter >= self.frames_count {
				self.frame_counter = 0;
			}
		}
	}


	pub fn encode_scalar(&self, source: &Sdr, target: &mut Sdr) {
		let v_size = self.ganglion_dims.v_size() as usize;
		let u_size = self.ganglion_dims.u_size() as usize;

		// for v_id in 0..v_size {
		// 	for u_id in 0..u_size {
		// 		let (x, y, valid) = coord_hex_to_pixel(v_size, v_id, u_size, u_id, 
		// 			self.image_height as usize, self.image_width as usize);
				
		// 		if valid {
		// 			let tar_idx = (v_id * u_size) + u_id;
		// 			let src_idx = (y * self.image_width as usize) + x;

		// 			target[tar_idx] = source[src_idx]; 
		// 		}
		// 		//target[tar_idx] = (x != 0 || y != 0) as u8; // SHOW INPUT SQUARE
		// 	}
		// }
	}


	// ENCODE_2D_IMAGE(): Horribly unoptimized.
	pub fn encode_2d_image(&self, source: &Sdr, target: &mut Sdr) {
		let v_size = self.ganglion_dims.v_size() as usize;
		let u_size = self.ganglion_dims.u_size() as usize;

		let x_size = self.image_width;
		let y_size = self.image_height;

		let hex_side = (x_size + y_size) as f64 / 
			(self.scale_factor * (v_size + u_size) as f64);

		let (x_ofs, y_ofs) = calc_offs(v_size, u_size, x_size, y_size, hex_side);

		for v_id in 0..v_size  {
			for u_id in 0..u_size {
				let (x, y, valid) = coord_hex_to_pixel(v_id as f64, u_id as f64, x_size as f64, 
					y_size as f64, hex_side, x_ofs, y_ofs);
				
				if valid {
					let tar_idx = (v_id * u_size) + u_id;
					let src_idx = (y as usize * x_size) + x as usize;

					target[tar_idx] = source[src_idx];
					// target[tar_idx] = 1 as u8; // SHOW INPUT SQUARE
				}
			}
		}
	}
	

	pub fn dims(&self) -> &CorticalDimensions {
		&self.ganglion_dims
	}
}


// COORD_HEX_TO_PIXEL(): Eventually either move this to GPU or at least use SIMD.
pub fn coord_hex_to_pixel(v_id: f64, u_id: f64, x_size: f64, y_size: f64, hex_side: f64, 
			x_ofs: f64, y_ofs: f64, 
	) -> (f64, f64, bool) 
{
	let u = u_id;
	let u_inv = 0.0 - u;
	let v = v_id;
	//let v_inv = 0f64 - v;
	//let w = u_inv + v_inv;
	let w_inv = v + u;
	//let s = HEX_SIDE;

	//let c1 = w_inv - C1_OFS;
	//let c2 = u_inv - C2_OFS;

	let mut x = w_inv * 1.5 * hex_side;
	let mut y = (u_inv + (w_inv / 2.0)) * SQRT_3 * hex_side;

	//let mut y = u * 1.5f64 * s;
	//let mut x = (v_inv + (u / 2f64)) * SQRT_3 * s;

	x -= x_ofs;
	y += y_ofs;	
	
	//x = x_size as f64 - x;
	//y = y_size as f64 - y;

	let valid = (y >= 0.0 && y < y_size) && (x >= 0.0 && x < x_size);

	(x, y, valid)
}


struct Margins {
	left: usize,
	right: usize,
	top: usize,
	bottom: usize,
}


fn calc_offs(v_size: usize, u_size: usize, y_size: usize, x_size: usize, hex_side: f64) -> (f64, f64) {
	let v_mid = v_size >> 1;
	let u_mid = u_size >> 1;

	let (x_ofs_inv, y_ofs_inv, _) = coord_hex_to_pixel(v_mid as f64, u_mid as f64, 
		x_size as f64, y_size as f64, hex_side, 0.0, 0.0);

	let x_mid = x_size >> 1;
	let y_mid = y_size >> 1;	

	((x_ofs_inv - x_mid as f64), (y_mid as f64 - y_ofs_inv))
}




	// pub fn encode_2d_image_crude(&self, source: &Sdr, target: &mut Sdr) {
	// 	for v in 0..self.image_height {
	// 		for u in 0..self.image_width {
	// 			let src_idx = (v * self.image_width as usize) + u;
	// 			let tar_idx = ((v + self.margins.top as usize) * self.ganglion_dims.u_size() as usize) 
	// 				+ (u + self.margins.left as usize);
	// 			target[tar_idx] = source[src_idx];
	// 		}
	// 	}
	// }


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

