use std;
use std::collections::{ self, HashMap, BTreeMap };
use std::collections::hash_state::{ HashState };
use num;
use std::ops::{ Index, IndexMut, Range };
use std::hash::{ self, Hash, SipHasher, Hasher };

//use proto::cell::{  };
//use super::layer as layer;
use super::layer::{ self, Protolayer, ProtolayerFlags, ProtoaxonKind, ProtolayerKind };
	//use super::layer::ProtolayerKind::{ self, Cellular, Axonal };
use super::cell::{ ProtocellKind, Protocell, DendriteKind };
use super::{ Protoregion, RegionKind };




//#[derive(Copy)]
pub struct Protoregions {
	map: HashMap<&'static str, Protoregion>,
}

impl Protoregions {
	pub fn new() -> Protoregions {
		Protoregions {
			map: HashMap::new(),
		}
	}

	pub fn r(mut self, pr: Protoregion) -> Protoregions {
		self.add(pr);
		self
	}	

	pub fn add(&mut self, pr: Protoregion) {
		self.map.insert(pr.name.clone(), pr);
	}

	// pub fn freeze(&mut self) {
	// 	for (prk, pr) in self.map.iter_mut() {
	// 		pr.freeze();
	// 	}
	// }
}

impl<'b> Index<&'b str> for Protoregions
{
    type Output = Protoregion;

    fn index<'a>(&'a self, region_name: &'b str) -> &'a Protoregion {
        self.map.get(region_name).expect(&format!("proto::regions::Protoregions::index(): \
        	Invalid region name: '{}'.", region_name))
    }
}

impl<'b> IndexMut<&'b str> for Protoregions
{
    fn index_mut<'a>(&'a mut self, region_name: &'b str) -> &'a mut Protoregion {
        self.map.get_mut(region_name).expect(&format!("proto::regions::Protoregions::index_mut(): \
        	Invalid region name: '{}'.", region_name))
    }
}



