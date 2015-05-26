use ocl::{ CorticalDimensions };
use proto::regions::{ ProtoRegionKind };

use std::collections::{ HashMap };




/*pub trait Width {
	fn width(&self, cr_type: &ProtoRegionKind) -> u32;
}*/

pub trait ProtoAreasTrait {
	fn new() -> ProtoAreas;
	fn add(&mut self, protoarea: ProtoArea);
	fn area(mut self, name: &'static str, width: u8, height: u8, region_kind: ProtoRegionKind) -> ProtoAreas;
}


pub type ProtoAreas = HashMap<&'static str, ProtoArea>;

/*impl Width for ProtoAreas {
	fn width(&self, cr_type: &ProtoRegionKind) -> u32 {
		let mut width = 0u32;
		for (area_name, area) in self.iter() {
			if area.region_kind == *cr_type {
				width += area.width;
			}
		}
		width
	}
}*/


impl ProtoAreasTrait for ProtoAreas {
	fn new() -> ProtoAreas {
		HashMap::new()
	}

	fn add(&mut self, protoarea: ProtoArea) {
		let name = protoarea.name;
		//let dims = protoarea.dims;
		self.insert(name, protoarea);
	}

	fn area(mut self, name: &'static str, width_l2: u8, height_l2: u8, region_kind: ProtoRegionKind) -> ProtoAreas {
		let mut new_area = ProtoArea { 
			name: name,
			dims: CorticalDimensions::new(width_l2, height_l2, 0, 0),
			region_kind: region_kind,
		};

		self.add(new_area);
		self
	}
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct ProtoArea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	//pub width: u32,
	//pub height: u32,
	pub region_kind: ProtoRegionKind,
}

impl Copy for ProtoArea {}

/*impl ProtoArea {
	pub fn width(&self) -> u32 {
		self.width
	}
}*/



//struct ProtoArea
