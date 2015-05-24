use ocl::{ CorticalDimensions };
use proto::regions::{ ProtoRegionKind };

use std::collections::{ HashMap };




/*pub trait Width {
	fn width(&self, cr_type: &ProtoRegionKind) -> u32;
}*/

pub trait AddNew {
	fn add(&mut self, protoarea: ProtoArea);
}


pub type ProtoAreas = HashMap<&'static str, ProtoArea>;

/*impl Width for ProtoAreas {
	fn width(&self, cr_type: &ProtoRegionKind) -> u32 {
		let mut width = 0u32;
		for (area_name, area) in self.iter() {
			if area.cort_reg_type == *cr_type {
				width += area.width;
			}
		}
		width
	}
}*/


impl AddNew for ProtoAreas {
	fn add(&mut self, protoarea: ProtoArea) {
		let name = protoarea.name;
		let dims = protoarea.dims;
		self.insert(name, protoarea);
	}
}


pub struct ProtoArea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	//pub width: u32,
	//pub height: u32,
	pub cort_reg_type: ProtoRegionKind,
}

/*impl ProtoArea {
	pub fn width(&self) -> u32 {
		self.width
	}
}*/



//struct ProtoArea
