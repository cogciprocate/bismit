use cmn::{CorticalDims, Sdr};
use input_source::InputGanglion;
use super::IdxData;

const SQRT_3: f32 = 1.73205080756f32;


//	IDXREADER: Reads IDX files containing a series of two dimensional matrices of unsigned 
//	bytes (u8) into a ganglion (SDR frame buffer: &Sdr)
pub struct IdxReader {
	ganglion_dims: CorticalDims,
	cycles_per_frame: usize,
	scale_factor: f32,
	repeat_counter: usize,
	frame_counter: usize,
	frames_count: usize,
	loop_frames: Option<u32>,
	image_width: usize,
	image_height: usize,	
	image_len: usize,
	idx_data: IdxData,
}

impl IdxReader {
	pub fn new(ganglion_dims: CorticalDims, file_name: &str, cycles_per_frame: usize, scale_factor: f32) -> IdxReader {
		let idx_data = IdxData::new(file_name);
		let dim_count = idx_data.dims().len();

		let image_width = if dim_count > 1 { idx_data.dims()[1] } else { 1 };
		let image_height = if dim_count > 2 { idx_data.dims()[2] } else { 1 };

		println!("IDXREADER: initialized with dimensions: {:?}", idx_data.dims());

	    IdxReader {
	    	ganglion_dims: ganglion_dims,
	    	cycles_per_frame: cycles_per_frame,
	    	scale_factor: scale_factor,
	    	repeat_counter: 0,
	    	frame_counter: 0,
	    	frames_count: idx_data.dims()[0],
	    	loop_frames: None,
	    	image_width: image_width,
	    	image_height: image_height,	    	
	    	image_len: image_width * image_height,
	    	idx_data: idx_data,
    	}
    }

    pub fn loop_frames(mut self, frames_to_loop: u32) -> IdxReader {
    	self.loop_frames = Some(frames_to_loop);
    	self
	}    

	pub fn get_raw_frame(&self, frame_idx: usize, ganglion_frame: &mut Sdr) -> usize {
		assert!(ganglion_frame.len() == self.ganglion_dims.columns() as usize);
		assert!(frame_idx < self.frames_count);

		let img_idz = frame_idx * self.image_len;

		for idx in 0..self.image_len {
			ganglion_frame[idx] = self.idx_data.data()[img_idz + idx];
		}

		return self.image_len;
	}

	pub fn get_first_byte(&self, frame_idx: usize) -> u8 {
		assert!(frame_idx < self.frames_count);
		let img_idz = frame_idx * self.image_len;

		return self.idx_data.data()[img_idz];

	}

	fn increment_frame(&mut self) {		
		self.repeat_counter += 1;

		if self.repeat_counter >= self.cycles_per_frame {
			self.repeat_counter = 0;
			self.frame_counter += 1;

			match self.loop_frames {
				Some(frames_to_loop) => {
					if self.frame_counter >= frames_to_loop as usize {
						self.frame_counter = 0;
					}
				},

				None => (),
			}

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

		let hex_side = (x_size + y_size) as f32 / 
			(self.scale_factor * (v_size + u_size) as f32);

		let (x_ofs, y_ofs) = calc_offs(v_size, u_size, x_size, y_size, hex_side);

		for v_id in 0..v_size  {
			for u_id in 0..u_size {
				let (x, y, valid) = coord_hex_to_pixel(v_id as f32, u_id as f32, x_size as f32, 
					y_size as f32, hex_side, x_ofs, y_ofs);
				
				if valid {
					let tar_idx = (v_id * u_size) + u_id;
					let src_idx = (y as usize * x_size) + x as usize;

					unsafe { *target.get_unchecked_mut(tar_idx) = *source.get_unchecked(src_idx); }

					// SHOW INPUT SQUARE
					// target[tar_idx] = 1 as u8;
				}
			}
		}
	}
	

	pub fn dims(&self) -> &CorticalDims {
		&self.ganglion_dims
	}
}

impl InputGanglion for IdxReader {
	fn cycle(&mut self, ganglion_frame: &mut Sdr) -> usize {
    	assert!(ganglion_frame.len() == self.ganglion_dims.columns() as usize);
    	assert!((self.image_len) <= ganglion_frame.len(), 
    		"Ganglion vector size must be greater than or equal to IDX image size");    	

  		//   	match self.file_reader.read(&mut self.idx_data.data()[..]) {
		//     Err(why) => panic!("\ncouldn't read '{}': {}", &self.file_path, Error::description(&why)),
		//     Ok(bytes) => assert!(bytes == self.idx_data.data().len(), "\n bytes read != buffer length"), 
		//     	//println!("{} contains:\n{:?}\n{} bytes read.", display, header_dim_sizes_bytes, bytes),
		// }

		let img_idz = self.frame_counter * self.image_len;
		let img_idn = img_idz + self.image_len;

		match self.idx_data.dims().len() {
			3 => self.encode_2d_image(&self.idx_data.data()[img_idz..img_idn], ganglion_frame),
			2 => panic!("\nOne dimensional (linear) idx images not yet supported."),
			1 => self.encode_scalar(&self.idx_data.data()[img_idz..img_idn], ganglion_frame),
			_ => panic!("\nIdx files with more than three or less than one dimension(s) not supported."),
		}

		let prev_frame = self.frame_counter;
		self.increment_frame();
		return prev_frame;
	}
}


// COORD_HEX_TO_PIXEL(): Eventually either move this to GPU or at least use SIMD.
pub fn coord_hex_to_pixel(v_id: f32, u_id: f32, x_size: f32, y_size: f32, hex_side: f32, 
			x_ofs: f32, y_ofs: f32, 
	) -> (f32, f32, bool) 
{
	let u = u_id;
	let u_inv = 0.0 - u;
	let v = v_id;
	let w_inv = v + u;

	let mut x = w_inv * 1.5 * hex_side;
	let mut y = (u_inv + (w_inv / 2.0)) * SQRT_3 * hex_side;

	x -= x_ofs;
	y += y_ofs;

	let valid = (y >= 0.0 && y < y_size) && (x >= 0.0 && x < x_size);

	(x, y, valid)
}


struct Margins {
	left: usize,
	right: usize,
	top: usize,
	bottom: usize,
}


fn calc_offs(v_size: usize, u_size: usize, y_size: usize, x_size: usize, hex_side: f32) -> (f32, f32) {
	let v_mid = v_size >> 1;
	let u_mid = u_size >> 1;

	let (x_ofs_inv, y_ofs_inv, _) = coord_hex_to_pixel(v_mid as f32, u_mid as f32, 
		x_size as f32, y_size as f32, hex_side, 0.0, 0.0);

	let x_mid = x_size >> 1;
	let y_mid = y_size >> 1;	

	((x_ofs_inv - x_mid as f32), (y_mid as f32 - y_ofs_inv))
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

