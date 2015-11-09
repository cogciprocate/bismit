use std::collections::{ /*self,*/ HashMap };

use proto::{ /*layer, RegionKind,*/ Protofilter, Protoinput };
use cmn::{ self, CorticalDims };


// pub trait ProtoAreaMapsTrait {
// 	fn new() -> ProtoAreaMaps;
// 	fn add(&mut self, protoarea: ProtoAreaMap);
// 	fn area(mut self, name: &'static str, width: u32, height: u32, 
// 		region_kind: &'static str, filters: Option<Vec<Protofilter>>, 
// 		aff_areas: Option<Vec<&'static str>>,
// 	) -> ProtoAreaMaps;
// 	fn freeze(&mut self);
// }

pub struct ProtoAreaMaps {
	maps: HashMap<&'static str, ProtoAreaMap>,
}

impl <'a>ProtoAreaMaps {
	pub fn new() -> ProtoAreaMaps {
		ProtoAreaMaps { maps: HashMap::new() }
	}

	fn add(&mut self, protoarea: ProtoAreaMap) {
		let name = protoarea.name;
		//let dims = protoarea.dims;
		self.maps.insert(name, protoarea);
	}

	pub fn area_ext(mut self, 
				name: &'static str, 
				region_name: &'static str,
				width: u32, 
				height: u32, 
				protoinput: Protoinput,				
				filters: Option<Vec<Protofilter>>,
				aff_areas_opt: Option<Vec<&'static str>>,
	) -> ProtoAreaMaps {
		self.add(ProtoAreaMap::new(name, region_name, width, height, protoinput, 
			filters, aff_areas_opt));

		self
	}

	pub fn area(mut self, 
				name: &'static str,
				region_name: &'static str,
				width: u32, 
				height: u32, 
				filters: Option<Vec<Protofilter>>,
				aff_areas_opt: Option<Vec<&'static str>>,
	) -> ProtoAreaMaps {
		self.add(ProtoAreaMap::new(name, region_name, width, height, Protoinput::None, filters, aff_areas_opt));
		self
	}

	// 	FREEZE(): CURRENTLY NO CHECKS TO MAKE SURE THIS HAS BEEN CALLED! -
	// 		- [Done] NEED TO RESTRUCTURE PROTOAREAS, LITERALLY
	pub fn freeze(&mut self) {
		let mut eff_list: Vec<(&'static str, &'static str)> = Vec::with_capacity(5);

		for (area_name, area) in self.maps.iter() {
			for aff_area_name in &area.aff_areas {
				eff_list.push((aff_area_name, area_name));
			}

			// match area.aff_areas {
			// 	Some(ref aff_area_names) => {
			// 		for aff_area_name in aff_area_names {
			// 			eff_list.push((aff_area_name, area_name));
			// 		}
			// 	},
			// 	None => (),
			// }
		}

		if eff_list.len() > cmn::MAX_EFFERENT_AREAS { 
			panic!("areas::ProtoAreaMaps::freeze(): \
				An area cannot have more than {} efferent areas.", cmn::MAX_EFFERENT_AREAS); 
		}

		for (area_name, eff_area_name) in eff_list {
			let emsg = format!("proto::areas::ProtoAreaMaps::freeze(): Area: '{}' not found. ", area_name);
			self.maps.get_mut(area_name).expect(&emsg).eff_areas.push(eff_area_name);
		}
	}

	pub fn maps(&self) -> &HashMap<&'static str, ProtoAreaMap> {
		&self.maps
	}
}

// impl Iterator for ProtoAreaMaps {
//     type Item = ProtoAreaMap;

//     fn next(&self) -> Option<&ProtoAreaMap> {
//     		return self.maps.next();
//         }
//         None
//     }
// }



#[derive(PartialEq, Debug, Clone)]
pub struct ProtoAreaMap {
	pub name: &'static str,
	pub region_name: &'static str,
	pub dims: CorticalDims,	
	//pub region_kind: RegionKind,
	pub input: Protoinput,
	pub filters: Option<Vec<Protofilter>>,
	pub aff_areas: Vec<&'static str>,
	pub eff_areas: Vec<&'static str>,
}

impl ProtoAreaMap {
	pub fn new(
				name: &'static str, 
				region_name: &'static str,
				width: u32, 
				height: u32, 				
				input: Protoinput,
				filters: Option<Vec<Protofilter>>,
				aff_areas_opt: Option<Vec<&'static str>>,
	) -> ProtoAreaMap {
		assert!(width >= cmn::SYNAPSE_REACH * 2);
		assert!(height >= cmn::SYNAPSE_REACH * 2);

		let aff_areas = match aff_areas_opt {
			Some(ae) => ae,
			None => Vec::with_capacity(0),
		};

		ProtoAreaMap { 
			name: name,
			region_name: region_name,
			dims: CorticalDims::new(width, height, 0, 0, None),
			//dims: CorticalDims::new(width_l2, height_l2, 0, 0),
			//region_kind: region_kind,
			input: input,
			filters: filters,
			aff_areas: aff_areas,
			eff_areas: Vec::with_capacity(5),
		}
	}

	pub fn dims(&self) -> &CorticalDims {
		&self.dims
	}

	pub fn input(&self) -> &Protoinput {
		&self.input
	}
}


