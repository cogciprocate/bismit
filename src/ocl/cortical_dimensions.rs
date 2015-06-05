/*	CorticalDimensions: Dimensions of a cortical area in units of cells
		Stored in log base 2 as a constraint (for simplicity and computational efficiency in OpenCL kernels). 

		Cells are cubes 1(W,x) x 1(H,y) x 1(D,z)

		Dimensions are in the context of bismit where: 
			- Column is 1 x 1 x N (a rod)
			- Slice has dimensions N x M x 1 (a plane)
			- Row has no meaning
		
		So, width * height determines number of columns

		The 4th parameter, per_cel_l2, is basically elements or divisions per cell and can also be thought of as a 4th dimension if you want to get all metaphysical about it. It can be positive or negative reflecting whether or not it's bigger or smaller than a cell and it's stored inverted. Don't think too hard about it.
*/

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct CorticalDimensions {
	width_l2: u8, // in cell-edges (log base 2) (WxHxD: 1x1xN)
	height_l2: u8, // in cell-edges (log2) (1x1xN)
	depth: u8, // in cell-edges (NxMx1)
	per_cel_l2: i8, // divisions per cell (log2)
}

impl CorticalDimensions {
	pub fn new(width_l2: u8, height_l2: u8,	depth: u8, per_cel_l2: i8,) -> CorticalDimensions {
		CorticalDimensions { 
			width_l2: width_l2,
			height_l2: height_l2,
			depth: depth,
			per_cel_l2: per_cel_l2,
		}
	}

	pub fn width_l2(&self) -> u8 {
		self.width_l2
	}

	pub fn height_l2(&self) -> u8 {
		self.height_l2
	}

	pub fn depth(&self) -> u8 {
		self.depth
	}

	pub fn per_cel_l2(&self) -> i8 {
		self.per_cel_l2
	}

	pub fn width(&self) -> u32 {
		1 << self.width_l2 as u32
	}

	pub fn height(&self) -> u32 {
		1 << self.height_l2 as u32
	}

	pub fn per_cel_l2_left(&self) -> Option<u32> {
		if self.per_cel_l2 >= 0 {
			Some(self.per_cel_l2 as u32)
		} else {
			None
			//panic!("ocl::CorticalDimensions::per_cel_l2_left(): may only be called if per_cel_l2 is positive");
		}
	}

	pub fn per_cel_l2_right(&self) -> Option<u32> {
		if self.per_cel_l2 < 0 {
			Some((0 - self.per_cel_l2) as u32)
		} else {
			None
			//panic!("ocl::CorticalDimensions::per_cel_l2_right(): may only be called if per_cel_l2 is negative");
		}
	}

	pub fn per_cel(&self) -> Option<u32> {
		match self.per_cel_l2_left() {
			Some(pcl2) => Some((1 << pcl2) as u32),
			None => None,
		}
	}

	// PER_SLICE(): 2D Area of a slice measured in divisions/elements/whatever
	pub fn per_slice(&self) -> u32 {
		 if self.per_cel_l2 >= 0 {
			self.columns() << self.per_cel_l2_left().expect("cortical_dimensions.rs")
		} else {
			self.columns() >> self.per_cel_l2_right().expect("cortical_dimensions.rs")
		}
	}

	// COLUMNS(): 2D Area of a slice measured in cells
	pub fn columns(&self) -> u32 {
		1 << (self.height_l2 + self.width_l2) as u32
	}

	// CELLS(): 3D Volume measured in cells
	pub fn cells(&self) -> u32 {
		self.columns() * self.depth as u32
	}

	// LEN(): 4D Volume - Total linear length if stretched out - measured in cell-piece-whatevers
	pub fn len(&self) -> u32 {
		if self.per_cel_l2 >= 0 {
			self.cells() << self.per_cel_l2_left().expect("cortical_dimensions.rs")
		} else {
			self.cells() >> self.per_cel_l2_right().expect("cortical_dimensions.rs")
		}
	}

	pub fn clone_with_pcl2(&self, per_cel_l2: i8) -> CorticalDimensions {
		CorticalDimensions { per_cel_l2: per_cel_l2, .. *self }
	}

	pub fn clone_with_depth(&self, depth: u8) -> CorticalDimensions {
		CorticalDimensions { depth: depth, .. *self }
	}

}

impl Copy for CorticalDimensions {}
