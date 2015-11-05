use ocl::{ EnvoyDimensions };

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
	tfts_per_cel: u32, // dendritic tufts per cell
	per_tft_l2: i8, // divisions per cell-tuft (log2)
	physical_increment: Option<u32>,
}

impl CorticalDimensions {
	pub fn new(u_size: u32, v_size: u32, depth: u8, per_tft_l2: i8, physical_increment: Option<u32>) -> CorticalDimensions {
	//pub fn new(u_size_l2: u8, v_size_l2: u8,	depth: u8, per_tft_l2: i8,) -> CorticalDimensions {
		
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
			tfts_per_cel: 1,
			per_tft_l2: per_tft_l2,
			physical_increment: physical_increment, // <<<<< PENDING RENAME
		}
	}

	pub fn u_size(&self) -> u32 {
		self.u_size
	}

	pub fn v_size(&self) -> u32 {
		self.v_size
	}

	pub fn depth(&self) -> u8 {
		self.depth
	}

	// TUFTS_PER_CEL(): Dendrite tufts per cell
	pub fn tfts_per_cel(&self) -> u32 {
		self.tfts_per_cel
	}

	pub fn per_tft_l2(&self) -> i8 {
		self.per_tft_l2
	}

	// PHYSICAL_INCREMENT(): 
	// 		TODO: improve this description
	pub fn physical_increment(&self) -> Result<u32, &'static str> {
		match self.physical_increment {
			Some(pi) => Ok(pi),
			None => Err("physical increment not set"),
		}
	}

	// SCALED_PHYSICAL_INCREMENT(): Represents the increment of the columns, not simply the dens/syns/whatever
	// 		i.e. if cel_phys_incr == 256, syns must have an phys_incr of cel_phys_incr * syns_per_cel
	//
	// 		TODO: DEPRICATE
	// pub fn scaled_physical_increment(&self) -> Result<u32, &'static str> {
	// 	match self.physical_increment {
	// 		Some(pi) => {
	// 			let phys_incr = (pi << self.per_tft_l2_left()) * self.tfts_per_cel;
	// 			Ok(phys_incr)
	// 		},

	// 		None => Err("physical increment not set"),
	// 	}
	// }

	// COLUMNS(): 2D Area of a slc measured in cell sides
	pub fn columns(&self) -> u32 {
		self.v_size * self.u_size
	}

	// CELLS(): 3D Volume of area measured in cells
	pub fn cells(&self) -> u32 {
		self.columns() * self.depth as u32
	}

	// TUFTS(): 4D Volume of area measured in (dendrite-tuft * cells)
	pub fn cel_tfts(&self) -> u32 {
		self.cells() * self.tfts_per_cel
	}

	pub fn per_tft_l2_left(&self) -> u32 {
		if self.per_tft_l2 >= 0 {
			self.per_tft_l2 as u32
		} else {
			panic!("\nocl::CorticalDimensions::per_tft_l2_left(): may only be called if per_tft_l2 is positive");
		}
	}

	pub fn per_tft_l2_right(&self) -> u32 {
		if self.per_tft_l2 < 0 {
			(0 - self.per_tft_l2) as u32
		} else {
			panic!("\nocl::CorticalDimensions::per_tft_l2_right(): may only be called if per_tft_l2 is negative");
		}
	}

	pub fn per_cel(&self) -> u32 {
		len_components(1, self.per_tft_l2, self.tfts_per_cel)
	}

	pub fn per_tft(&self) -> u32 {
		len_components(1, self.per_tft_l2, 1)
	}

	pub fn per_slc_per_tft(&self) -> u32 {
		len_components(self.columns(), self.per_tft_l2, 1)
	}

	// PER_SLICE(): 2D Area of a slc measured in divisions/components/whatever
	pub fn per_slc(&self) -> u32 {
		len_components(self.columns(), self.per_tft_l2, self.tfts_per_cel)
	}

	pub fn per_subgrp(&self, subgroup_count: u32) -> Result<u32, &'static str> {
		let physical_len = try!(self.physical_len());

		if physical_len % subgroup_count == 0 {
			return Ok(physical_len / subgroup_count) 
		} else {
			return Err("Invalid subgroup size.");
		}
	}	

	pub fn clone_with_ptl2(&self, per_tft_l2: i8) -> CorticalDimensions {
		CorticalDimensions { per_tft_l2: per_tft_l2, .. *self }
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

	pub fn with_tfts(mut self, tfts_per_cel: u32) -> CorticalDimensions {
		self.tfts_per_cel = tfts_per_cel;
		self
	}

	// PHYSICAL_LEN(): Length of array required to hold the section of cortex represented by these dimensions
	// 		- Rounded based on columns and is therefore safe for 
	pub fn physical_len(&self) -> Result<u32, &'static str> {
		let cols = self.columns();
		let phys_incr = try!(self.physical_increment());

		let len_mod = cols % phys_incr;

		if len_mod == 0 {
			Ok(len_components(cols * self.depth as u32, self.per_tft_l2, self.tfts_per_cel))
		} else {
			let pad = phys_incr - len_mod;
			debug_assert_eq!((cols + pad) % phys_incr, 0);
			Ok(len_components((cols + pad) * self.depth as u32, self.per_tft_l2, self.tfts_per_cel))
		}
	}
}

impl Copy for CorticalDimensions {}

impl EnvoyDimensions for CorticalDimensions {
	// PHYSICAL_LEN(): ROUND CORTICAL_LEN() UP TO THE NEXT PHYSICAL_INCREMENT 
	// TODO: RETURN A RESULT<>
	fn physical_len(&self) -> u32 {
		self.physical_len().expect("EnvoyDimensions::len()")
	}
}


// fn resolve_physical_increment(ocl: Option<&ProQueue>) -> Option<u32> {
// 	match ocl {
// 		Some(ocl) => Some(ocl.get_max_work_group_size()),
// 		None => None,
// 	}
// }

fn len_components(cells: u32, per_tft_l2: i8, tfts_per_cel: u32) -> u32 {
	//println!("\n\n##### TOTAL_LEN(): cells: {}, pcl2: {}", cells, per_tft_l2);
	let tufts = cells * tfts_per_cel;

	if per_tft_l2 >= 0 {
		tufts << per_tft_l2
	} else {
		tufts >> (0 - per_tft_l2)
	}
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

// #[cfg(test)]
// pub mod tests {
// 	use super::*;

// 	#[test]
// 	fn test_len() {

// 		// Actually test the damn thing...

// 	}
// }



	// LEN(): 4D Volume - Total linear length if stretched out - measured in cell-piece-whatevers
	/* TEMPORARY */
	/*pub fn len(&self) -> u32 {
		self.len()
	}*/

	/* CORTICAL_LEN(): 'VIRTUAL' CORTEX SIZE, NOT TO BE CONFUSED WITH THE PHYSICAL IN-MEMORY SIZE */
	/*fn cortical_len(&self) -> u32 {
		len_components(self.cells(), self.per_tuft_l2)
	}*/	
