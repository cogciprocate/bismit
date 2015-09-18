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
use super::{ Protoregion, ProtoregionKind };




//#[derive(Copy)]
pub struct Protoregions {	// <<<<< SLATED FOR REMOVAL
	pub hash_map: HashMap<ProtoregionKind, Protoregion>,
}

impl Protoregions {
	pub fn new() -> Protoregions {
		Protoregions {
			hash_map: HashMap::new(),
		}
	}

	pub fn region(mut self, pr: Protoregion) -> Protoregions {
		self.add(pr);
		self
	}	

	pub fn add(&mut self, pr: Protoregion) {
		self.hash_map.insert(pr.kind.clone(), pr);
	}

	// pub fn freeze(&mut self) {
	// 	for (prk, pr) in self.hash_map.iter_mut() {
	// 		pr.freeze();
	// 	}
	// }
}

impl<'b> Index<&'b ProtoregionKind> for Protoregions
{
    type Output = Protoregion;

    fn index<'a>(&'a self, index: &'b ProtoregionKind) -> &'a Protoregion {
        self.hash_map.get(index).expect("proto::regions::Protoregions::index(): Invalid region kind.")
    }
}

impl<'b> IndexMut<&'b ProtoregionKind> for Protoregions
{
    fn index_mut<'a>(&'a mut self, index: &'b ProtoregionKind) -> &'a mut Protoregion {
        self.hash_map.get_mut(index).expect("proto::regions::Protoregions::index_mut(): Invalid region kind.")
    }
}



