use ocl;
use envoy::{ Envoy };
use common;


use std::ops::{ Range };
use std::collections::{ HashMap, HashSet };
use std::num;




pub struct AxonSpace {
	pub states: Envoy<ocl::cl_char>,
}

impl AxonSpace {
	pub fn new(len: usize, ocl: ocl::Ocl) -> AxonSpace {
		AxonSpace { 
			states: Envoy::<ocl::cl_char>::new(len, 0i8, ocl),
		}
	}
}


/*

	Axon Space Redesign:

	-Axon Space will exist as it's own Envoy.

	-It will have 2 parts:::: Scratch that.... it will have one part only
		-vals (char)
		not necessary:			(-somata_idx (uint) -> somata)

	-Synapses will write directly to their corresponding axon space.

	-The role of "Soma" is unclear.
		- It may be necessary for storing some learning data.
		- It might be necessary to store information regarding inhibition characteristics such as negative weights though this should be managable through the normal systems -- still unclear.
		- Looks like this may become a temporary holding area while data from distal + proximal synapses is collected and inhibition taken into account.

	-Long distance communication will be handled by a "relay" (white matter) kernel which will copy data based on a table.
		-That table will contain source and destination coordinates in the form:
			(5, 6), (2, -2); read as: source (segment 5, row 6), destination (segment 2, row -2).
		-Segments will be somewhere between 256 - 65k in size. There will be a maximum of 65k of them
		-Layers will be the size of the number of total columns. There will be a maximum of 256.
		-The white_matter kernel will simply read the i8 from the source segment and row and write to the destination.

	-Sensory data will be written by a "Thalamus" kernel directly to axon space

	-Axon Space will be referred to as a two dimensional space.
		-Width (idx) of an axon space will be equal to the number of columns.
		-Height (lvl) will be broken down as follows:
			- Row 12:			Input from ultra-cortical region (feed-back)
			- Row 13:			Motor Input (copy of current motor commands)
			- Row 14:			Sensory Input (Thal.)
			- Row 15:			Input from sub-cortical region (feed forward)
			- Rows 16 - 255:	Local Columnar Cells
						//	[-8] --> [0], reserved for inputs / connections from sensory and foreign cortical areas
						//	[1] --> [16+] local cell outputs

	-Synapses will now have:
		-one u8 to describe absolute row and 
		-one i8 to describe relative column (horizontal) of an axon.


				//	-Synapses will continue to have a 16bit signed idx value.
				//		-8 LSBs will refer to horizontal position.
				//		-8 MSBs will refer to vertical.
				//		-this will constrain all synapses to horizontal positions within 128 column distances (-128 --> 127).


	Other Stuff:
	- Need to redesign kernel args, probably to accept a tuple or something simple.
	- Set up proximal dendrites: seperate Envoy, seperate kernel. Probably 16 synapses for now.


*/



/*pub struct Axons {
	
}
impl Axons {
	pub fn new(size: usize, ocl: &ocl::Ocl) -> Axons {

		Axons {
			
		}
	}
}*/



/*
pub struct AxonSpace {
	//pub regions: HashMap<&'static str, AxonRegion>,
	pub rows: HashMap<&'static str, AxonRegion>,
	pub states: Envoy<ocl::cl_char>,
	pub depth: ocl::cl_char,
	pub width: usize,
}

impl AxonSpace {
	pub fn new(anterows: ocl::cl_char, ocl: ocl::Ocl) -> AxonSpace {
		let depth = anterows + num::cast(common::LAYERS_PER_SEGMENT);
		let width = common::CELLS_PER_LAYER;
		let rows = HashMap::with_capacity(depth);
		//let regions = HashMap::

		Range { start: anterows, end: (16 + common::LAYERS_PER_SEGMENT) };


		let mut states = Envoy::<ocl::cl_char>::new(depth * width, 0i8, ocl);


		AxonSpace { 
			//regions: HashMap::new(),
			rows: rows,
			rows: rows,
			states: states,
		}
	}

	pub fn new_region<T>(&mut self, name: &'static str, start: usize, end: usize, origin: &Envoy<T>) -> &AxonRegion {
		let origin_buf = self.insert_origin(origin);
		self.insert_region(name, AxonRegion::new(name, start, end, origin_buf));
		self.get(name)
	}

	pub fn insert_region(&mut self, name: &'static str, ar: AxonRegion) {
		let opt = self.map.insert(name, ar);
		match opt {
			Some(x)		=> panic!("Cannot insert duplicate AxonRegions into AxonSpace"),
			None		=> (),
		}
	}

	pub fn insert_origin<T>(&mut self, origin: &Envoy<T>) -> ocl::cl_mem {
		self.origins.insert(origin.buf);
		origin.buf
	}

	pub fn get(&self, name: &'static str) -> &AxonRegion {
		self.map.get(name).unwrap()
	}

	pub fn get_mut(&mut self, name: &'static str) -> &mut AxonRegion {
		self.map.get_mut(name).unwrap()
	}

	pub fn len() {
		// Add up all the regions
	}
}


pub struct AxonRegion {
	pub range: Range<usize>,
	pub name: &'static str,
	pub origin_buf: ocl::cl_mem,
}

impl AxonRegion {
	pub fn new(name: &'static str, start: usize, end: usize, origin_buf: ocl::cl_mem) -> AxonRegion {
		AxonRegion {
			range: Range { start: start, end: end },
			name: name,
			origin_buf: origin_buf,
		} 
	}
}


pub struct AxonLayer {
	range: Range<ocl::cl_uchar>,
}

impl AxonLayer {
	pub fn len(&self) -> usize{
		self.range.len()
	}
	pub fn range(&self) -> Range<ocl::cl_uchar> {
		self.range
	}
}
*/
