use std::collections::{ self, HashMap };

use ocl::{ CorticalDimensions };
use proto::region::{ RegionKind };
use proto::filter::{ Protofilter };
use cmn;


// pub trait ProtoareasTrait {
// 	fn new() -> Protoareas;
// 	fn add(&mut self, protoarea: Protoarea);
// 	fn area(mut self, name: &'static str, width: u32, height: u32, 
// 		region_kind: &'static str, filters: Option<Vec<Protofilter>>, 
// 		aff_areas: Option<Vec<&'static str>>,
// 	) -> Protoareas;
// 	fn freeze(&mut self);
// }

pub struct Protoareas {
	map: HashMap<&'static str, Protoarea>,
}

impl <'a>Protoareas {
	pub fn new() -> Protoareas {
		Protoareas { map: HashMap::new() }
	}

	fn add(&mut self, protoarea: Protoarea) {
		let name = protoarea.name;
		//let dims = protoarea.dims;
		self.map.insert(name, protoarea);
	}

	pub fn area_ext(mut self, 
				name: &'static str, 
				region_name: &'static str,
				width: u32, 
				height: u32, 
				protoinput: Protoinput,				
				filters: Option<Vec<Protofilter>>,
				aff_areas_opt: Option<Vec<&'static str>>,
	) -> Protoareas {
		self.add(Protoarea::new(name, region_name, width, height, protoinput, 
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
	) -> Protoareas {
		self.add(Protoarea::new(name, region_name, width, height, Protoinput::None, filters, aff_areas_opt));
		self
	}

	// 	FREEZE(): CURRENTLY NO CHECKS TO MAKE SURE THIS HAS BEEN CALLED! -
	// 		- [Done] NEED TO RESTRUCTURE PROTOAREAS, LITERALLY
	pub fn freeze(&mut self) {
		let mut eff_list: Vec<(&'static str, &'static str)> = Vec::with_capacity(5);

		for (area_name, area) in self.map.iter() {
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
			panic!("areas::Protoareas::freeze(): \
				An area cannot have more than {} efferent areas.", cmn::MAX_EFFERENT_AREAS); 
		}

		for (area_name, eff_area_name) in eff_list {
			let emsg = format!("proto::areas::Protoareas::freeze(): Area: '{}' not found. ", area_name);
			self.map.get_mut(area_name).expect(&emsg).eff_areas.push(eff_area_name);
		}
	}

	pub fn map(&self) -> &HashMap<&'static str, Protoarea> {
		&self.map
	}
}

// impl Iterator for Protoareas {
//     type Item = Protoarea;

//     fn next(&self) -> Option<&Protoarea> {
//     		return self.map.next();
//         }
//         None
//     }
// }



#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Protoarea {
	pub name: &'static str,
	pub region_name: &'static str,
	pub dims: CorticalDimensions,	
	//pub region_kind: RegionKind,
	pub input: Protoinput,
	pub filters: Option<Vec<Protofilter>>,
	pub aff_areas: Vec<&'static str>,
	pub eff_areas: Vec<&'static str>,
}

impl Protoarea {
	pub fn new(
				name: &'static str, 
				region_name: &'static str,
				width: u32, 
				height: u32, 				
				input: Protoinput,
				filters: Option<Vec<Protofilter>>,
				aff_areas_opt: Option<Vec<&'static str>>,
	) -> Protoarea {
		assert!(width > cmn::SYNAPSE_SPAN_GEO);
		assert!(height > cmn::SYNAPSE_SPAN_GEO);

		let aff_areas = match aff_areas_opt {
			Some(ae) => ae,
			None => Vec::with_capacity(0),
		};

		Protoarea { 
			name: name,
			region_name: region_name,
			dims: CorticalDimensions::new(width, height, 0, 0, None),
			//dims: CorticalDimensions::new(width_l2, height_l2, 0, 0),
			//region_kind: region_kind,
			input: input,
			filters: filters,
			aff_areas: aff_areas,
			eff_areas: Vec::with_capacity(5),
		}
	}
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Protoinput {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader { file_name: &'static str, repeats: usize },
	None,
}

