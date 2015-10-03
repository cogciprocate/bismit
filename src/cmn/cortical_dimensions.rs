use ocl::{ OclProgQueue, EnvoyDimensions };

/*	CorticalDimensions: Dimensions of a cortical area in units of cells
		- Used to define both the volume and granularity of a cortical area.
		- Also contains extra information such as opencl kernel workgroup size


		<<<<< THIS DESCRIPTION IS WAY OUT OF DATE >>>>>
		<<<<< TODO: ADD INFORMATION ABOUT TUFTS (OUR 5TH DIMENSION?) >>>>>

		Stored in log base 2 as a constraint (for simplicity and computational efficiency in OpenCL kernels). 

		Cells are hexagonal prisms

		Dimensions are in the context of bismit where: 
			- Column is 1 x 1 x N (a rod)
			- Slice (unfortunately coincident with rust terminology) has dimensions N x M x 1 (a plane)
			- Row has no meaning
		
		So, u_size * v_size determines number of columns

		The 4th parameter, per_cel_l2, is basically components or divisions per cell and can also be thought of as a 4th dimension. It can be positive or negative reflecting whether or not it's bigger or smaller than a cell and it's stored inverted. Don't think too hard about it.
*/

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct CorticalDimensions {
	//u_size_l2: u8, // in cell-edges (log base 2) (WxHxD: 1x1xN)
	//v_size_l2: u8, // in cell-edges (log2) (1x1xN)
	u_size: u32,
	v_size: u32,
	depth: u8, // in cell-edges (NxMx1)
	tufts_per_cel: u32, // dendritic tufts per cell
	per_tuft_l2: i8, // divisions per cell-tuft (log2)
	physical_increment: Option<u32>,
}

impl CorticalDimensions {
	pub fn new(u_size: u32, v_size: u32, depth: u8, per_tuft_l2: i8, physical_increment: Option<u32>) -> CorticalDimensions {
	//pub fn new(u_size_l2: u8, v_size_l2: u8,	depth: u8, per_tuft_l2: i8,) -> CorticalDimensions {
		
		//assert!(super::OPENCL_PREFERRED_VECTOR_MULTIPLE == 4);
		//println!("\n\n##### v_size: {}, u_size: {}", v_size, u_size);
		//let physical_increment = resolve_physical_increment(ocl);
		//assert!(v_size % 4 == 0, "CorticalDimensions::new(): Size of dimension 'v' must be a multiple of 4.");
		//assert!(u_size % 4 == 0, "CorticalDimensions::new(): Size of dimension 'u' must be a multiple of 4.");

		CorticalDimensions { 
			u_size: u_size,
			v_size: v_size,
			/*u_size_l2: u_size_l2,
			v_size_l2: v_size_l2,*/
			depth: depth,
			tufts_per_cel: 1,
			per_tuft_l2: per_tuft_l2,
			physical_increment: physical_increment, // <<<<< PENDING RENAME
		}
	}

	pub fn physical_increment(&self) -> u32 {
		match self.physical_increment {
			Some(pi) => pi,
			None => panic!("\ncortical_dimensions::CorticalDimensions::physical_increment(): Physical Increment not set!"),
		}
	}

	pub fn u_size(&self) -> u32 {
		self.u_size
	}

	pub fn v_size(&self) -> u32 {
		self.v_size
	}

	/*pub fn u_size_l2(&self) -> u8 {
		self.u_size_l2
	}

	pub fn v_size_l2(&self) -> u8 {
		self.v_size_l2
	}

	pub fn u_size(&self) -> u32 {
		1 << self.u_size_l2 as u32
	}

	pub fn v_size(&self) -> u32 {
		1 << self.v_size_l2 as u32
	}
	*/

	pub fn depth(&self) -> u8 {
		self.depth
	}

	// TUFTS_PER_CEL(): Dendrite tufts per cell
	pub fn tufts_per_cel(&self) -> u32 {
		self.tufts_per_cel
	}

	pub fn per_tuft_l2(&self) -> i8 {
		self.per_tuft_l2
	}


	// COLUMNS(): 2D Area of a slc measured in cell sides
	pub fn columns(&self) -> u32 {
		self.v_size * self.u_size
		//1 << (self.v_size_l2 + self.u_size_l2) as u32
	}

	// CELLS(): 3D Volume of area measured in cells
	pub fn cells(&self) -> u32 {
		self.columns() * self.depth as u32
	}

	// TUFTS(): 4D Volume of area measured in (dendrite-tuft * cells)
	pub fn cel_tufts(&self) -> u32 {
		self.cells() * self.tufts_per_cel
	}

	pub fn per_tuft_l2_left(&self) -> u32 {
		if self.per_tuft_l2 >= 0 {
			self.per_tuft_l2 as u32
		} else {
			panic!("\nocl::CorticalDimensions::per_tuft_l2_left(): may only be called if per_tuft_l2 is positive");
		}
	}

	pub fn per_tuft_l2_right(&self) -> u32 {
		if self.per_tuft_l2 < 0 {
			(0 - self.per_tuft_l2) as u32
		} else {
			panic!("\nocl::CorticalDimensions::per_tuft_l2_right(): may only be called if per_tuft_l2 is negative");
		}
	}

	pub fn per_cel(&self) -> u32 {
		//(1 << self.per_tuft_l2_left()) as u32
		len_components(1, self.per_tuft_l2, self.tufts_per_cel)
	}

	pub fn per_tuft(&self) -> u32 {
		len_components(1, self.per_tuft_l2, 1)
	}

	pub fn per_slc_per_tuft(&self) -> u32 {
		len_components(self.columns(), self.per_tuft_l2, 1)
	}

	// PER_SLICE(): 2D Area of a slc measured in divisions/components/whatever
	pub fn per_slc(&self) -> u32 {
		len_components(self.columns(), self.per_tuft_l2, self.tufts_per_cel)
	}


	pub fn cols_per_subgrp(&self, subgroup_count: u32) -> Result<u32, &'static str> {
		if self.columns() % subgroup_count == 0 {
			return Ok(self.physical_len() / subgroup_count) 
		} else {
			return Err("Invalid subgroup size.");
		}
	}	

	pub fn clone_with_ptl2(&self, per_tuft_l2: i8) -> CorticalDimensions {
		CorticalDimensions { per_tuft_l2: per_tuft_l2, .. *self }
	}

	pub fn clone_with_depth(&self, depth: u8) -> CorticalDimensions {
		CorticalDimensions { depth: depth, .. *self }
	}

	pub fn clone_with_physical_increment(&self, physical_increment: u32) -> CorticalDimensions {
		CorticalDimensions { physical_increment: Some(physical_increment), .. *self } 
	}

	pub fn set_physical_increment(&mut self, physical_increment: u32) {
		self.physical_increment = Some(physical_increment);
	}

	pub fn with_physical_increment(mut self, physical_increment: u32) -> CorticalDimensions {
		self.set_physical_increment(physical_increment);
		self
	}

	pub fn with_tufts(mut self, tufts_per_cel: u32) -> CorticalDimensions {
		self.tufts_per_cel = tufts_per_cel;
		self
	}
}

impl Copy for CorticalDimensions {}

impl EnvoyDimensions for CorticalDimensions {
	/* PHYSICAL_LEN(): ROUND CORTICAL_LEN() UP TO THE NEXT PHYSICAL_INCREMENT */
	fn physical_len(&self) -> u32 {
		let cols = self.columns();
		let phys_inc = self.physical_increment();

		let len_mod = cols % phys_inc;

		if len_mod == 0 {
			len_components(cols * self.depth as u32, self.per_tuft_l2, self.tufts_per_cel)
		} else {
			let pad = self.physical_increment() - len_mod;
			len_components((cols + pad) * self.depth as u32, self.per_tuft_l2, self.tufts_per_cel)
		}
	}
}


fn resolve_physical_increment(ocl: Option<&OclProgQueue>) -> Option<u32> {
	match ocl {
		Some(ocl) => Some(ocl.get_max_work_group_size()),
		None => None,
	}
}

fn len_components(cells: u32, per_tuft_l2: i8, tufts_per_cel: u32) -> u32 {
	//println!("\n\n##### TOTAL_LEN(): cells: {}, pcl2: {}", cells, per_tuft_l2);
	let tufts = cells * tufts_per_cel;

	if per_tuft_l2 >= 0 {
		tufts << per_tuft_l2
	} else {
		tufts >> (0 - per_tuft_l2)
	}
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_len() {

		// Actually test the damn thing...

	}
}



	// LEN(): 4D Volume - Total linear length if stretched out - measured in cell-piece-whatevers
	/* TEMPORARY */
	/*pub fn len(&self) -> u32 {
		self.physical_len()
	}*/

	/* CORTICAL_LEN(): 'VIRTUAL' CORTEX SIZE, NOT TO BE CONFUSED WITH THE PHYSICAL IN-MEMORY SIZE */
	/*fn cortical_len(&self) -> u32 {
		len_components(self.cells(), self.per_tuft_l2)
	}*/	
