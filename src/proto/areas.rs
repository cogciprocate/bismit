use std::collections::{ HashMap };

use ocl::{ CorticalDimensions };
use proto::regions::{ ProtoregionKind };




/*pub trait Width {
	fn width(&self, cr_type: &ProtoregionKind) -> u32;
}*/

pub trait ProtoareasTrait {
	fn new() -> Protoareas;
	fn add(&mut self, protoarea: Protoarea);
	fn area(mut self, name: &'static str, width: u8, height: u8, region_kind: ProtoregionKind, afferent_area: Option<&'static str>) -> Protoareas;
}


pub type Protoareas = HashMap<&'static str, Protoarea>;

/*impl Width for Protoareas {
	fn width(&self, cr_type: &ProtoregionKind) -> u32 {
		let mut width = 0u32;
		for (area_name, area) in self.iter() {
			if area.region_kind == *cr_type {
				width += area.width;
			}
		}
		width
	}
}*/


impl ProtoareasTrait for Protoareas {
	fn new() -> Protoareas {
		HashMap::new()
	}

	fn add(&mut self, protoarea: Protoarea) {
		let name = protoarea.name;
		//let dims = protoarea.dims;
		self.insert(name, protoarea);
	}

	fn area(mut self, name: &'static str, width_l2: u8, height_l2: u8, region_kind: ProtoregionKind, afferent_area: Option<&'static str>) -> Protoareas {
		let mut new_area = Protoarea { 
			name: name,
			dims: CorticalDimensions::new(width_l2, height_l2, 0, 0),
			region_kind: region_kind,
			afferent_area: afferent_area,
		};

		self.add(new_area);
		self
	}
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Protoarea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	//pub width: u32,
	//pub height: u32,
	pub region_kind: ProtoregionKind,
	pub afferent_area: Option<&'static str>,
}

impl Copy for Protoarea {}

/*impl Protoarea {
	pub fn width(&self) -> u32 {
		self.width
	}
}*/



//struct Protoarea
