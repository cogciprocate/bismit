// use std::path::{Path, PathBuf};
// use find_folder::Search;
use cmn::{CorticalDims, Sdr};
use input_source::InputTract;
use super::IdxData;

//	IDXREADER: Reads IDX files containing a series of two dimensional matrices of unsigned 
//	bytes (u8) into a ganglion (SDR frame buffer: &Sdr)
pub struct IdxStreamer {
	ganglion_dims: CorticalDims,
	cycles_per_frame: usize,
	scale_factor: f32,
	repeat_counter: usize,
	frame_counter: usize,
	frames_count: usize,
	loop_frames: Option<u32>,
	// image_width: usize,
	// image_height: usize,
	image_dims: (usize, usize),
	// image_len: usize,
	idx_data: IdxData,
}

impl IdxStreamer {
	/// # Panics
	/// All sorts of reasons...
	pub fn new(ganglion_dims: CorticalDims, file_path_str: &str, cycles_per_frame: usize, 
				scale_factor: f32) -> IdxStreamer 
	{
		let idx_data = IdxData::new(file_path_str, false);
		let dim_count = idx_data.dims().len();

		assert!(dim_count <= 3, "IdxStreamer::new(): Cannot handle idx files with more than \
			three dimensions. [file: '{}']", file_path_str);
		// let image_width = if dim_count > 1 { idx_data.dims()[1] } else { 1 };
		// let image_height = if dim_count > 2 { idx_data.dims()[2] } else { 1 };

		let image_dims = (if dim_count > 1 { idx_data.dims()[1] } else { 1 },
			if dim_count > 2 { idx_data.dims()[2] } else { 1 });

		// let image_len = image_dims.0 * image_dims.1;

		println!("IDXREADER: initialized with dimensions: {:?}", idx_data.dims());

	    IdxStreamer {
	    	ganglion_dims: ganglion_dims,
	    	cycles_per_frame: cycles_per_frame,
	    	scale_factor: scale_factor,
	    	repeat_counter: 0,
	    	frame_counter: 0,
	    	frames_count: idx_data.dims()[0],
	    	loop_frames: None,
	    	// image_width: image_width,
	    	// image_height: image_height,	    	
	    	image_dims: image_dims,
	    	// image_len: image_width * image_height,
	    	// image_len: image_len,
	    	idx_data: idx_data,
    	}
    }

    pub fn loop_frames(mut self, frames_to_loop: u32) -> IdxStreamer {
    	self.loop_frames = Some(frames_to_loop);
    	self
	}    

	pub fn get_raw_frame(&self, frame_idx: usize, ganglion_frame: &mut Sdr) -> usize {
		assert!(ganglion_frame.len() == self.ganglion_dims.columns() as usize);
		assert!(frame_idx < self.frames_count);

		let img_idz = frame_idx * self.image_len();

		for idx in 0..self.image_len() {
			ganglion_frame[idx] = self.idx_data.data()[img_idz + idx];
		}

		return self.image_len();
	}

	pub fn get_first_byte(&self, frame_idx: usize) -> u8 {
		assert!(frame_idx < self.frames_count);
		let img_idz = frame_idx * self.image_len();

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
		// [FIXME]: NOT HOOKED UP
		super::encode_scalar();
	}


	// ENCODE_2D_IMAGE(): Horribly unoptimized.
	pub fn encode_2d_image(&self, source: &Sdr, target: &mut Sdr) {
		super::encode_2d_image(self.ganglion_dims.v_size() as usize, 
			// self.ganglion_dims.u_size() as usize, self.image_width, self.image_height,
			self.ganglion_dims.u_size() as usize, self.image_dims.0, self.image_dims.0,
			self.scale_factor, source, target);
	}	

	pub fn image_len(&self) -> usize {
		self.image_dims.0 * self.image_dims.1
	}

	pub fn dims(&self) -> &CorticalDims {
		&self.ganglion_dims
	}
}

impl InputTract for IdxStreamer {
	fn cycle(&mut self, ganglion_frame: &mut Sdr) -> usize {
    	assert!(ganglion_frame.len() == self.ganglion_dims.columns() as usize);
    	assert!((self.image_len()) <= ganglion_frame.len(), 
    		"Ganglion vector size must be greater than or equal to IDX image size");    	

  		//   	match self.file_reader.read(&mut self.idx_data.data()[..]) {
		//     Err(why) => panic!("\ncouldn't read '{}': {}", &self.file_path, Error::description(&why)),
		//     Ok(bytes) => assert!(bytes == self.idx_data.data().len(), "\n bytes read != buffer length"), 
		//     	//println!("{} contains:\n{:?}\n{} bytes read.", display, header_dim_sizes_bytes, bytes),
		// }

		let img_idz = self.frame_counter * self.image_len();
		let img_idn = img_idz + self.image_len();

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

