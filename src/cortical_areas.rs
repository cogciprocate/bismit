use cortical_regions::{ CorticalRegionType };

use std::collections::{ HashMap };

pub trait Width {
	fn width(&self, cr_type: CorticalRegionType) -> u32;
}

trait AddNew {
	fn add_new(&mut self, name: &'static str, cortical_area: CorticalArea) -> u32;
}




pub type CorticalAreas = HashMap<&'static str, CorticalArea>;

impl Width for CorticalAreas {
	fn width(&self, cr_type: CorticalRegionType) -> u32 {
		let mut width = 0u32;
		for (area_name, area) in self.iter() {
			if area.cort_reg_type == cr_type {
				width += area.width;
			}
		}
		width
	}
}

impl AddNew for CorticalAreas {
	fn add_new(&mut self, name: &'static str, cortical_area: CorticalArea) -> u32 {
		let width = cortical_area.width;
		self.insert(name, cortical_area);
		width
	}
}


struct CorticalArea {
	pub width: u32,
	pub offset: u32,
	pub cort_reg_type: CorticalRegionType,
}


pub fn define() -> CorticalAreas {
	let mut cortical_areas  = HashMap::new();
	let mut curr_offset: u32 = 128;

	curr_offset += cortical_areas.add_new("v1", CorticalArea { width: 1024, offset: curr_offset, cort_reg_type: CorticalRegionType::Sensory });

	cortical_areas
}


//struct CorticalArea
