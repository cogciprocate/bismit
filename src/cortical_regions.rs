// CorticalArea for the specifics
// CorticalRegion to define shit

use ocl;

use std::collections::{ self, HashMap };
use std::collections::hash_state::{ HashState };
use std::num;
use std::ops::{ Index, IndexMut, Range };
use std::borrow::BorrowFrom;
use std::hash::{ self, Hash, SipHasher, Hasher };



pub fn define() -> CorticalRegions {
	use self::CorticalLayerClass::*;
	use self::CorticalAxonScope::*;
	use self::CorticalCellType::*;

	let mut cort_regs: CorticalRegions = CorticalRegions::new();

	//let mut cri: = 0u8;

	let mut sen = CorticalRegion::new();

	sen.add_new_layer("thal", Interregional(Thalamocortical), 1);
	sen.add_new_layer("iv", Interlaminar(vec!["thal"], Pyramidal), 4);
	sen.add_new_layer("iii", Interlaminar(vec!["iv"], Pyramidal), 3);
	sen.add_new_layer("test", Interlaminar(vec!["iv", "thal"], Pyramidal), 3);

	cort_regs.add(CorticalRegionType::Sensory, sen);

	cort_regs
}

//#[derive(Copy)]
pub struct CorticalRegions {
	pub hash_map: HashMap<CorticalRegionType, CorticalRegion>,
}

impl CorticalRegions {
	fn new() -> CorticalRegions {
		CorticalRegions {
			hash_map: HashMap::new(),
		}
	}

	pub fn height(&self, cr_type: CorticalRegionType) -> (u8, u8) {
		let mut height_antecellular_rows = 0u8;		//	Interregional
		let mut height_cellular_rows = 0u8;			//	Interlaminar
		for (region_type, region) in self.hash_map.iter() {					// CHANGE TO FILTER
			if *region_type == cr_type {							//
				let (antecell, cell) = region.height();
				height_antecellular_rows += antecell;
				height_cellular_rows += cell;
				//println!("*** antecell: {}, cell: {} ***", antecell, cell);
			}
		}
		(height_antecellular_rows, height_cellular_rows)
	}

	fn add(&mut self, crt: CorticalRegionType, cr: CorticalRegion) {
		self.hash_map.insert(crt, cr);
	}
}

impl Index<CorticalRegionType> for CorticalRegions
{
    type Output = CorticalRegion;

    fn index<'a>(&'a self, index: &CorticalRegionType) -> &'a CorticalRegion {
        self.hash_map.get(index).expect("no entry found for key")
    }
}

impl IndexMut<CorticalRegionType> for CorticalRegions
{
    type Output = CorticalRegion;

    fn index_mut<'a>(&'a mut self, index: &CorticalRegionType) -> &'a mut CorticalRegion {
        self.hash_map.get_mut(index).expect("no entry found for key")
    }
}



pub struct CorticalRegion {
	pub layers: HashMap<&'static str, CorticalLayer>,
	height: u8,
}

impl CorticalRegion {
	pub fn new ()  -> CorticalRegion {
		CorticalRegion { 
			layers: HashMap::new(),
			height: 0,
		}
	}

	pub fn add_new_layer(&mut self, ln: &'static str, clc: CorticalLayerClass, height: u8) {
		let cl = CorticalLayer { class: clc, row_initial: self.height, height: height };
		self.height += cl.height;
		self.layers.insert(ln, cl);
	}

	pub fn width() -> u8 {
		panic!("not implemented");
	}

	pub fn height(&self) -> (u8, u8) {
		let mut antecell_rows = 0u8;
		let mut cell_rows = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.class {
				CorticalLayerClass::Interregional(_) => antecell_rows += layer.height,
				CorticalLayerClass::Interlaminar(_, _) => cell_rows += layer.height,
			}
		}
		assert!(antecell_rows + cell_rows == self.height);
		(antecell_rows, cell_rows)
	}

	pub fn layer_row_idxs(&self, layer_name: &'static str) -> Vec<u8> {

		let l = &self.layers[layer_name];
		let mut row_idxs = Vec::new();
			for i in range(l.row_initial, l.row_initial + l.height) {
				row_idxs.push(i);
			}
		return row_idxs;

		/*for (&ln, l) in self.layers.iter() {			// CHANGE TO FILTER
			if ln == layer_name {						//
				let mut row_idxs = Vec::new();
				for i in range(l.row_initial, l.row_initial + l.height) {
					row_idxs.push(i);
				}
				return row_idxs;
			}
		}
		panic!("cortical_regions::CorticalRegion::layer_interval(): Layer ({}) not found in region.", layer_name);*/
	}

	pub fn layer_src_row_idxs(&self, layer_name: &'static str) -> Vec<u8> {
		let src_row_names = self.layers[layer_name].src_row_names();
		
		let mut src_row_idxs = Vec::new();

		for &src_row_name in src_row_names.iter() {
			src_row_idxs.push_all(self.layer_row_idxs(src_row_name).as_slice());
		}

		//println!("CorticalRegion::layer_srcs_row_idxs(): (name:sources:idxs) [{}]:{:?}:{:?}", layer_name, src_row_names, src_row_idxs);
		
		src_row_idxs
 	}

}


struct CorticalLayer {
	//pub name: &'static str,
	pub class: CorticalLayerClass,
	pub row_initial: u8,
	pub height: u8,
	//pub layer_srcs: &'static str,
}

impl CorticalLayer {
	pub fn height(&self) -> ocl::cl_uchar {
		self.height
	}

	pub fn src_row_names(&self) -> Vec<&'static str> {
		match self.class {
			CorticalLayerClass::Interlaminar (ref slns, _) => slns.clone(),
			_ => panic!("Layer must be Interlaminar to determine source layers"),
		}
	}
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum CorticalRegionType {
	Associational,
	Sensory,
	Motor,
}


#[derive(PartialEq, Debug, Clone)]
pub enum CorticalCellType {
	Pyramidal,
	SpinyStellate,
	AspinyStellate,
}
// excitatory spiny stellate
// inhibitory aspiny stellate 


#[derive(PartialEq, Debug, Clone)]
pub enum CorticalAxonScope {
	Corticocortical,
	Thalamocortical,
	Corticothalamic,
	Corticospinal,
}


#[derive(PartialEq, Debug, Clone)]
pub enum CorticalLayerClass {
	Interregional (CorticalAxonScope),
	Interlaminar (Vec<&'static str>, CorticalCellType),
}


fn increment_row_index(mut cri: u8, by: u8) -> u8 {
	cri += by;
	cri
}
