use std::collections::{ /*self,*/ HashMap };

use proto::{ /*layer, RegionKind,*/ Protofilter, Protoinput };
use cmn::{ self, CorticalDims };
// use map;


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
				side: u32, 
				protoinput: Protoinput,				
				filters: Option<Vec<Protofilter>>,
				eff_areas_opt: Option<Vec<&'static str>>,
			) -> ProtoAreaMaps 
	{
		self.add(ProtoAreaMap::new(name, region_name, side, protoinput, 
			filters, eff_areas_opt));

		self
	}

	pub fn area(mut self, 
				name: &'static str,
				region_name: &'static str,
				side: u32, 
				filters: Option<Vec<Protofilter>>,
				eff_areas_opt: Option<Vec<&'static str>>,
			) -> ProtoAreaMaps 
	{
		self.add(ProtoAreaMap::new(name, region_name, side, Protoinput::None, filters, eff_areas_opt));
		self
	}


	// 	FREEZE(): CURRENTLY NO CHECKS TO MAKE SURE THIS HAS BEEN CALLED! -
	pub fn freeze(&mut self) {
		let mut aff_list: Vec<(&'static str, &'static str)> = Vec::with_capacity(5);

		for (area_name, area) in self.maps.iter() {
			for eff_area_name in &area.eff_areas {
				aff_list.push((eff_area_name, area_name));
			}
		}

		assert!(aff_list.len() <= cmn::MAX_AFFERENT_AREAS, "areas::ProtoAreaMaps::freeze(): \
				An area cannot have more than {} afferent areas.", cmn::MAX_AFFERENT_AREAS);

		for (area_name, aff_area_name) in aff_list {
			let emsg = format!("proto::areas::ProtoAreaMaps::freeze(): Area: '{}' not found. ", area_name);
			self.maps.get_mut(area_name).expect(&emsg).aff_areas.push(aff_area_name);
		}
	}


	// OLD -- DEPRICATE
	// pub fn freeze_old(&mut self) {
	// 	let mut eff_list: Vec<(&'static str, &'static str)> = Vec::with_capacity(5);

	// 	for (area_name, area) in self.maps.iter() {
	// 		for aff_area_name in &area.aff_areas {
	// 			eff_list.push((aff_area_name, area_name));
	// 		}
	// 	}

	// 	assert!(eff_list.len() <= cmn::MAX_EFFERENT_AREAS, "areas::ProtoAreaMaps::freeze(): \
	// 			An area cannot have more than {} efferent areas.", cmn::MAX_EFFERENT_AREAS);

	// 	for (area_name, eff_area_name) in eff_list {
	// 		let emsg = format!("proto::areas::ProtoAreaMaps::freeze(): Area: '{}' not found. ", area_name);
	// 		self.maps.get_mut(area_name).expect(&emsg).eff_areas.push(eff_area_name);
	// 	}
	// }

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
				side: u32,
				input: Protoinput,
				filters: Option<Vec<Protofilter>>,
				eff_areas_opt: Option<Vec<&'static str>>,
			) -> ProtoAreaMap 
	{
		// [FIXME] TODO: This is out of date. Need to instead verify that 'side' is > Protocell::den_*_syn_reach.
		assert!(side >= cmn::SYNAPSE_REACH * 2);

		let eff_areas = match eff_areas_opt {
			Some(ea) => ea,
			None => Vec::with_capacity(0),
		};

		ProtoAreaMap { 
			name: name,
			region_name: region_name,
			dims: CorticalDims::new(side, side, 0, 0, None),
			//dims: CorticalDims::new(width_l2, height_l2, 0, 0),
			//region_kind: region_kind,
			input: input,
			filters: filters,
			aff_areas: Vec::with_capacity(4),
			eff_areas: eff_areas,
		}
	}

	pub fn dims(&self) -> &CorticalDims {
		&self.dims
	}

	pub fn input(&self) -> &Protoinput {
		&self.input
	}
}


