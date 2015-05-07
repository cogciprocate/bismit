use protoregions::{ ProtoRegionKind };

use std::collections::{ HashMap };




pub trait Width {
	fn width(&self, cr_type: &ProtoRegionKind) -> u32;
}

pub trait AddNew {
	fn add_new(&mut self, name: &'static str, cortical_area: ProtoArea) -> u32;
}


pub type ProtoAreas = HashMap<&'static str, ProtoArea>;

impl Width for ProtoAreas {
	fn width(&self, cr_type: &ProtoRegionKind) -> u32 {
		let mut width = 0u32;
		for (area_name, area) in self.iter() {
			if area.cort_reg_type == *cr_type {
				width += area.width;
			}
		}
		width
	}
}

impl AddNew for ProtoAreas {
	fn add_new(&mut self, name: &'static str, cortical_area: ProtoArea) -> u32 {
		let width = cortical_area.width;
		self.insert(name, cortical_area);
		width
	}
}


pub struct ProtoArea {
	pub width: u32,
	pub offset: u32,
	pub cort_reg_type: ProtoRegionKind,
}

impl ProtoArea {
	pub fn width(&self) -> u32 {
		self.width
	}
}



//struct ProtoArea
