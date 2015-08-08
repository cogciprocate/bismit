use std::collections::{ HashMap };

use ocl::{ CorticalDimensions };
use proto::regions::{ ProtoregionKind };
use proto::filter::{ Protofilter };
use cmn;


pub trait ProtoareasTrait {
	fn new() -> Protoareas;
	fn add(&mut self, protoarea: Protoarea);
	fn area(mut self, name: &'static str, width: u32, height: u32, 
		region_kind: ProtoregionKind, filters: Option<Vec<Protofilter>>, 
		afferent_area: Option<Vec<&'static str>>
	) -> Protoareas;
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

	fn area(
				mut self, 
				name: &'static str, 
				width: u32, 
				height: u32, 
				/*width_l2: u8, 
				height_l2: u8, */
				region_kind: ProtoregionKind, 
				filters: Option<Vec<Protofilter>>,
				afferent_areas: Option<Vec<&'static str>>,
	) -> Protoareas {

		assert!(width > cmn::SYNAPSE_SPAN_GEO);
		assert!(height > cmn::SYNAPSE_SPAN_GEO);

		let mut new_area = Protoarea { 
			name: name,
			dims: CorticalDimensions::new(width, height, 0, 0, None),
			//dims: CorticalDimensions::new(width_l2, height_l2, 0, 0),
			region_kind: region_kind,
			filters: filters,
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
	pub filters: Option<Vec<Protofilter>>,
	pub afferent_areas: Option<Vec<&'static str>>,
}

//impl Copy for Protoarea {}

