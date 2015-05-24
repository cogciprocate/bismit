/*	CorticalDimensions: Dimensions of a cortical area in units of cells
		Cells are cubes 1(W,x) x 1(H,y) x 1(D,z)

		Dimensions are in the context of bismit where: 
			- Column is 1 x 1 x N (a rod)
			- Slice has dimensions N x M x 1 (a plane)
			- Row has no meaning
		
		So, width * height determines number of columns

		The 4th parameter, per_cel_l2, is basically elements or divisions per cell and can also be thought of as a 4th dimension if you want to get all metaphysical about it. It's given in log base 2 for simplicity and computational efficiency.
*/

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct CorticalDimensions {
	width: u32, // in cell-edges (WxHxD: 1x1xN)
	height: u32, // in cell-edges (1x1xN)
	depth: u8, // in cell-edges (NxMx1)
	per_cel_l2: i32, // divisions per cell (log2)
}

impl CorticalDimensions {
	pub fn new(width: u32, height: u32,	depth: u8, per_cel_l2: i32,) -> CorticalDimensions {
		CorticalDimensions { 
			width: width,
			height: height,
			depth: depth,
			per_cel_l2: per_cel_l2,
		}
	}

	pub fn height(&self) -> u32 {
		self.height
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn depth(&self) -> u8 {
		self.depth
	}

	pub fn per_cel_l2_left(&self) -> u32 {
		if self.per_cel_l2 >= 0 {
			self.per_cel_l2 as u32
		} else {
			panic!("ocl::CorticalDimensions::per_cel_l2_left(): may only be called if per_cel_l2 is positive");
		}
	}

	pub fn per_cel_l2_right(&self) -> u32 {
		if self.per_cel_l2 < 0 {
			(0 - self.per_cel_l2) as u32
		} else {
			panic!("ocl::CorticalDimensions::per_cel_l2_right(): may only be called if per_cel_l2 is negative");
		}
	}

	pub fn per_cel(&self) -> u32 {
		1 << self.per_cel_l2_left()
	}

	pub fn per_slice(&self) -> u32 {
		(self.height * self.width) << self.per_cel_l2_left()
	}

	// COLUMNS(): 2d Area of a slice measured in cell-sides
	pub fn columns(&self) -> u32 {
		self.height * self.width
	}

	// CELLS(): Volume measured in cells
	pub fn cells(&self) -> u32 {
		self.height * self.width * self.depth as u32
	}

	// LEN(): 4D Volume - Total linear length if stretched out - measured in cell-piece-whatevers
	pub fn len(&self) -> u32 {
		if self.per_cel_l2 >= 0 {
			self.cells() << self.per_cel_l2_left()
		} else {
			self.cells() >> self.per_cel_l2_right()
		}
	}

	pub fn clone_with_pcl2(&self, per_cel_l2: i32) -> CorticalDimensions {
		CorticalDimensions { per_cel_l2: per_cel_l2, .. *self }
	}

	pub fn clone_with_depth(&self, depth: u8) -> CorticalDimensions {
		CorticalDimensions { depth: depth, .. *self }
	}

}

impl Copy for CorticalDimensions {}
