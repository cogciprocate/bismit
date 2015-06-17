use std::collections::{ HashMap };

use ocl::{ CorticalDimensions };
use proto::regions::{ ProtoregionKind };




/*pub trait Width {
	fn width(&self, cr_type: &ProtoregionKind) -> u32;
}*/

pub trait ProtoareasTrait {
	fn new() -> Protoareas;
	fn add(&mut self, protoarea: Protoarea);
	fn area(mut self, name: &'static str, width: u8, height: u8, region_kind: ProtoregionKind, afferent_area: Option<Vec<&'static str>>) -> Protoareas;
}


pub type Protoareas = HashMap<&'static str, Protoarea>;

impl ProtoareasTrait for Protoareas {
	fn new() -> Protoareas {
		HashMap::new()
	}

	fn add(&mut self, protoarea: Protoarea) {
		let name = protoarea.name;
		//let dims = protoarea.dims;
		self.insert(name, protoarea);
	}

	fn area(mut self, name: &'static str, width_l2: u8, height_l2: u8, region_kind: ProtoregionKind, afferent_areas: Option<Vec<&'static str>>) -> Protoareas {
		let mut new_area = Protoarea { 
			name: name,
			dims: CorticalDimensions::new(width_l2, height_l2, 0, 0),
			region_kind: region_kind,
			afferent_areas: afferent_areas,
		};

		self.add(new_area);
		self
	}
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Protoarea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	pub region_kind: ProtoregionKind,
	pub afferent_areas: Option<Vec<&'static str>>,
}

//impl Copy for Protoarea {}

/*impl Protoarea {
	pub fn width(&self) -> u32 {
		self.width
	}
}*/



//struct Protoarea
