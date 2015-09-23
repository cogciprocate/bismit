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
use super::{ ProtolayerMap, RegionKind };




//#[derive(Copy)]
pub struct ProtolayerMaps {
	map: HashMap<&'static str, ProtolayerMap>,
}

impl ProtolayerMaps {
	pub fn new() -> ProtolayerMaps {
		ProtolayerMaps {
			map: HashMap::new(),
		}
	}

	pub fn r(mut self, pr: ProtolayerMap) -> ProtolayerMaps {
		self.add(pr);
		self
	}	

	pub fn add(&mut self, pr: ProtolayerMap) {
		self.map.insert(pr.name.clone(), pr);
	}

	// pub fn freeze(&mut self) {
	// 	for (prk, pr) in self.map.iter_mut() {
	// 		pr.freeze();
	// 	}
	// }
}

impl<'b> Index<&'b str> for ProtolayerMaps
{
    type Output = ProtolayerMap;

    fn index<'a>(&'a self, region_name: &'b str) -> &'a ProtolayerMap {
        self.map.get(region_name).expect(&format!("proto::regions::ProtolayerMaps::index(): \
        	Invalid region name: '{}'.", region_name))
    }
}

impl<'b> IndexMut<&'b str> for ProtolayerMaps
{
    fn index_mut<'a>(&'a mut self, region_name: &'b str) -> &'a mut ProtolayerMap {
        self.map.get_mut(region_name).expect(&format!("proto::regions::ProtolayerMaps::index_mut(): \
        	Invalid region name: '{}'.", region_name))
    }
}



