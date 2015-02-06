// CorticalArea for the specifics
// CorticalRegion to define shit

use ocl;
use cell_type::{ CellType };

use std::collections::{ self, HashMap };
use std::collections::hash_state::{ HashState };
use std::num;
use std::ops::{ Index, IndexMut, Range };
use std::borrow::BorrowFrom;
use std::hash::{ self, Hash, SipHasher, Hasher };



pub fn define() -> CorticalRegions {
	use self::CorticalLayerClass::*;
	use self::CorticalAxonScope::*;
	use cell_type::CellType::*;

	let mut cort_regs: CorticalRegions = CorticalRegions::new();

	//let mut cri: = 0u8;

	let mut sen = CorticalRegion::new();

	sen.add_new_layer("thal", Interregional(Thalamocortical), 1);
	sen.add_new_layer("iv", Interlaminar(vec!["thal"], Pyramidal), 3);
	sen.add_new_layer("iii", Interlaminar(vec!["iv"], Pyramidal), 3);
	sen.add_new_layer("ii", Interlaminar(vec!["iii"], Pyramidal), 2);
	//sen.add_new_layer("test", Interlaminar(vec!["iii"], Pyramidal), 4);

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

	pub fn height_total(&self, cr_type: CorticalRegionType) -> u8 {
		let (hacr, hcr) = self.height(cr_type);
		hacr + hcr
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
	cell_type_row_initial: HashMap<CellType, u8>,
}

impl CorticalRegion {
	pub fn new ()  -> CorticalRegion {

		let mut hct = HashMap::new();
		hct.insert(CellType::Pyramidal, 0);
		hct.insert(CellType::AspinyStellate, 0);
		hct.insert(CellType::SpinyStellate, 0);
	
		CorticalRegion { 
			layers: HashMap::new(),
			height: 0,
			cell_type_row_initial: hct,
		}
	}

	pub fn add_new_layer(&mut self, ln: &'static str, clc: CorticalLayerClass, height: u8) {

		let mut cell_type_row_initial = 0u8;

		match &clc {
			&CorticalLayerClass::Interlaminar(_, ref cct) => {
				cell_type_row_initial = self.cell_type_row_initial[*cct];
				//println!("Layer: {}, cell_type({:?}): row_initial: {}", ln, cct, cell_type_row_initial);
				self.cell_type_row_initial[*cct] += height;
			}
			_ => (),
		};
		
		let cl = CorticalLayer { 
			class: clc, 
			row_initial: self.height, 
			cell_type_row_initial: cell_type_row_initial,
			height: height,
		};

		self.height += height;

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

	pub fn layer_row_ids(&self, layer_name: &'static str) -> Vec<u8> {

		let l = &self[layer_name];
		let mut row_ids = Vec::new();
			for i in range(l.row_initial, l.row_initial + l.height) {
				row_ids.push(i);
			}
		return row_ids;

		/*for (&ln, l) in self.layers.iter() {			// CHANGE TO FILTER
			if ln == layer_name {						//
				let mut row_ids = Vec::new();
				for i in range(l.row_initial, l.row_initial + l.height) {
					row_ids.push(i);
				}
				return row_ids;
			}
		}
		panic!("cortical_regions::CorticalRegion::layer_interval(): Layer ({}) not found in region.", layer_name);*/
	}

	pub fn layer_src_row_ids(&self, layer_name: &'static str) -> Vec<u8> {
		let src_row_names = self[layer_name].src_row_names();
		
		let mut src_row_ids = Vec::new();

		for &src_row_name in src_row_names.iter() {
			src_row_ids.push_all(self.layer_row_ids(src_row_name).as_slice());
		}

		//println!("CorticalRegion::layer_srcs_row_ids(): (name:sources:idxs) [{}]:{:?}:{:?}", layer_name, src_row_names, src_row_ids);
		
		src_row_ids
 	}

 	pub fn layer_row_ids_ct(&self, layer_name: &'static str) -> Vec<u8> {

		let l = &self[layer_name];
		let mut row_ids = Vec::new();
			for i in range(l.cell_type_row_initial, l.cell_type_row_initial + l.height) {
				row_ids.push(i);
			}
		return row_ids;
	}

	pub fn layer_src_row_ids_ct(&self, layer_name: &'static str) -> Vec<u8> {
		let src_row_names = self[layer_name].src_row_names();
		
		let mut src_row_ids = Vec::new();

		for &src_row_name in src_row_names.iter() {
			src_row_ids.push_all(self.layer_row_ids_ct(src_row_name).as_slice());
		}

		//println!("CorticalRegion::layer_srcs_row_ids(): (name:sources:idxs) [{}]:{:?}:{:?}", layer_name, src_row_names, src_row_ids);
		
		src_row_ids
 	}

}

impl Index<&'static str> for CorticalRegion
{
    type Output = CorticalLayer;

    fn index<'a>(&'a self, index: &&'static str) -> &'a CorticalLayer {
        self.layers.get(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}


impl IndexMut<&'static str> for CorticalRegion
{
    type Output = CorticalLayer;

    fn index_mut<'a>(&'a mut self, index: &&'static str) -> &'a mut CorticalLayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}



struct CorticalLayer {
	//pub name: &'static str,
	pub class: CorticalLayerClass,
	pub row_initial: u8,
	pub cell_type_row_initial: u8,
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
pub enum CorticalAxonScope {
	Corticocortical,
	Thalamocortical,
	Corticothalamic,
	Corticospinal,
}


#[derive(PartialEq, Debug, Clone)]
pub enum CorticalLayerClass {
	Interregional (CorticalAxonScope),
	Interlaminar (Vec<&'static str>, CellType),
}


fn increment_row_index(mut cri: u8, by: u8) -> u8 {
	cri += by;
	cri
}
