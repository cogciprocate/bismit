use std::collections::{ HashMap };

use ocl::{ CorticalDimensions };
use proto::region::{ ProtoregionKind };
use proto::filter::{ Protofilter };
use cmn;


pub trait ProtoareasTrait {
	fn new() -> Protoareas;
	fn add(&mut self, protoarea: Protoarea);
	fn area(mut self, name: &'static str, width: u32, height: u32, 
		region_kind: ProtoregionKind, filters: Option<Vec<Protofilter>>, 
		afferent_areas: Option<Vec<&'static str>>,
	) -> Protoareas;
	fn freeze(&mut self);
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
				afferent_areas_opt: Option<Vec<&'static str>>,
	) -> Protoareas {

		assert!(width > cmn::SYNAPSE_SPAN_GEO);
		assert!(height > cmn::SYNAPSE_SPAN_GEO);

		let afferent_areas = match afferent_areas_opt {
			Some(ae) => ae,
			None => Vec::with_capacity(0),
		};

		let mut new_area = Protoarea { 
			name: name,
			dims: CorticalDimensions::new(width, height, 0, 0, None),
			//dims: CorticalDimensions::new(width_l2, height_l2, 0, 0),
			region_kind: region_kind,
			filters: filters,
			afferent_areas: afferent_areas,
			efferent_areas: Vec::with_capacity(5),
		};

		self.add(new_area);
		self
	}

	// 	FREEZE(): CURRENTLY NO CHECKS TO MAKE SURE THIS HAS BEEN CALLED! -
	// 		- NEED TO RESTRUCTURE PROTOAREAS, LITERALLY
	fn freeze(&mut self) {
		let mut eff_list: Vec<(&'static str, &'static str)> = Vec::with_capacity(50);

		for (area_name, area) in self.iter() {
			for aff_area_name in &area.afferent_areas {
				eff_list.push((aff_area_name, area_name));
			}

			// match area.afferent_areas {
			// 	Some(ref aff_area_names) => {
			// 		for aff_area_name in aff_area_names {
			// 			eff_list.push((aff_area_name, area_name));
			// 		}
			// 	},
			// 	None => (),
			// }
		}

		if eff_list.len() > cmn::MAX_EFFERENT_AREAS { 
			panic!("areas::Protoareas::freeze(): \
				An area cannot have more than {} efferent areas.", cmn::MAX_EFFERENT_AREAS); 
		}

		for (area_name, eff_area_name) in eff_list {
			let emsg = format!("proto::areas::Protoareas::freeze(): Area: '{}' not found. ", area_name);
			self.get_mut(area_name).expect(&emsg).efferent_areas.push(eff_area_name);
		}
	}
}

#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Protoarea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	pub region_kind: ProtoregionKind,
	pub filters: Option<Vec<Protofilter>>,
	pub afferent_areas: Vec<&'static str>,
	pub efferent_areas: Vec<&'static str>,
}

//impl Copy for Protoarea {}

