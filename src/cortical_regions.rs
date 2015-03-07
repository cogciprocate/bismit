// CorticalArea for the specifics
// CorticalRegion to define shit

use ocl;
//use protocell::{  };
use cortical_region_layer::{ CorticalRegionLayer, LayerFlags };
use protocell::{ CellKind, Protocell, DendriteKind };

use std::collections::{ self, HashMap };
use std::collections::hash_state::{ HashState };
use std::num;
use std::ops::{ Index, IndexMut, Range };
use std::borrow::BorrowFrom;
use std::hash::{ self, Hash, SipHasher, Hasher };



//#[derive(Copy)]
pub struct CorticalRegions {
	pub hash_map: HashMap<CorticalRegionType, CorticalRegion>,
}

impl CorticalRegions {
	pub fn new() -> CorticalRegions {
		CorticalRegions {
			hash_map: HashMap::new(),
		}
	}

	pub fn height(&self, cr_type: CorticalRegionType) -> (u8, u8) {
		let mut height_noncellular_rows = 0u8;		//	Interregional
		let mut height_cellular_rows = 0u8;			//	Interlaminar
		for (region_type, region) in self.hash_map.iter() {					// CHANGE TO FILTER
			if *region_type == cr_type {							//
				let (noncell, cell) = region.height();
				height_noncellular_rows += noncell;
				height_cellular_rows += cell;
				//println!("*** noncell: {}, cell: {} ***", noncell, cell);
			}
		}
		(height_noncellular_rows, height_cellular_rows)
	}

	pub fn height_total(&self, cr_type: CorticalRegionType) -> u8 {
		let (hacr, hcr) = self.height(cr_type);
		hacr + hcr
	}

	pub fn add(&mut self, cr: CorticalRegion) {
		self.hash_map.insert(cr.kind.clone(), cr);
	}
}

impl Index<CorticalRegionType> for CorticalRegions
{
    type Output = CorticalRegion;

    fn index<'a>(&'a self, index: &CorticalRegionType) -> &'a CorticalRegion {
        self.hash_map.get(index).expect("Invalid region name.")
    }
}

impl IndexMut<CorticalRegionType> for CorticalRegions
{
    type Output = CorticalRegion;

    fn index_mut<'a>(&'a mut self, index: &CorticalRegionType) -> &'a mut CorticalRegion {
        self.hash_map.get_mut(index).expect("Invalid region name.")
    }
}



pub struct CorticalRegion_new {
	pub layers: HashMap<&'static str, CorticalRegionLayer>,
	pub kind: CorticalRegionType,
}

impl CorticalRegion_new {
	pub fn new (kind: CorticalRegionType)  -> CorticalRegion_new {
		let mut next_row_id = HashMap::new();
		next_row_id.insert(CellKind::Pyramidal, 0);
		next_row_id.insert(CellKind::AspinyStellate, 0);
		next_row_id.insert(CellKind::SpinyStellate, 0);
	
		CorticalRegion_new { 
			layers: HashMap::new(),
			kind: kind,
		}
	}

	pub fn new_layer(
					&mut self, 
					layer_name: &'static str,
					layer_height: u8,
					flags: LayerFlags,
					cell: Option<Protocell>,
	) {
		let (noncell_rows, cell_rows) = self.height();

		let next_base_row_id = self.total_height();

		let next_kind_base_row_pos = match cell {
			Some(ref protocell) => self.cell_kind_row_count(&protocell.cell_kind),
			None => noncell_rows,
		};

		println!("Layer: {}, layer_height: {}, base_row_id: {}, kind_base_row_pos: {}", layer_name, layer_height, next_base_row_id, next_kind_base_row_pos);
		
		let cl = CorticalRegionLayer {
			name : layer_name,
			cell: cell,
			base_row_id: next_base_row_id, 
			kind_base_row_pos: next_kind_base_row_pos,
			height: layer_height,
			flags: flags,
		};

		self.add(cl);
	}

	pub fn add(&mut self, layer: CorticalRegionLayer) {
		self.layers.insert(layer.name, layer);
	}

	pub fn width() -> u8 {
		panic!("not implemented");
	}

	pub fn cell_kind_row_count(&self, cell_kind: &CellKind) -> u8 {
		let mut count = 0u8;
		for (_, layer) in self.layers.iter() {
			match layer.cell {
				Some(ref protocell) => match &protocell.cell_kind {
					ref cell_kind => count += layer.height,
				},
				None => (),
			}
		}
		count
	}

	pub fn total_height(&self) -> u8 {
		let mut total_height = 0u8;
		for (_, layer) in self.layers.iter() {
			total_height += layer.height;
		}
		total_height
	}
 
	pub fn height(&self) -> (u8, u8) {
		let mut noncell_rows = 0u8;
		let mut cell_rows = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.cell {
				None => noncell_rows += layer.height,
				Some(_) => cell_rows += layer.height,
			}
		}
		(noncell_rows, cell_rows)
	}

	pub fn row_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		let mut row_ids = Vec::new();
		for &layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in range(l.base_row_id, l.base_row_id + l.height) {
					row_ids.push(i);
				}
		}
		row_ids
	}

	pub fn src_row_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_layer_names = self[layer_name].src_layer_names(den_type);
		
		self.row_ids(src_layer_names)
 	}

 	pub fn col_input_row(&self) -> u8 {
 		for (layer_name, layer) in self.layers.iter() {

 		}
 		5
 	}
}

impl Index<&'static str> for CorticalRegion_new
{
    type Output = CorticalRegionLayer;

    fn index<'a>(&'a self, index: &&'static str) -> &'a CorticalRegionLayer {
        self.layers.get(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}

impl IndexMut<&'static str> for CorticalRegion_new
{
    type Output = CorticalRegionLayer;

    fn index_mut<'a>(&'a mut self, index: &&'static str) -> &'a mut CorticalRegionLayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}



pub struct CorticalRegion {
	pub layers: HashMap<&'static str, CorticalRegionLayer>,
	pub kind: CorticalRegionType,
}

impl CorticalRegion {
	pub fn new (kind: CorticalRegionType)  -> CorticalRegion {
		let mut next_row_id = HashMap::new();
		next_row_id.insert(CellKind::Pyramidal, 0);
		next_row_id.insert(CellKind::AspinyStellate, 0);
		next_row_id.insert(CellKind::SpinyStellate, 0);
	
		CorticalRegion { 
			layers: HashMap::new(),
			kind: kind,
		}
	}

	pub fn new_layer(
					&mut self, 
					layer_name: &'static str,
					layer_height: u8,
					flags: LayerFlags,
					cell: Option<Protocell>,
	) {
		let (noncell_rows, cell_rows) = self.height();

		let next_base_row_id = self.total_height();

		let next_kind_base_row_pos = match cell {
			Some(ref protocell) => self.cell_kind_row_count(&protocell.cell_kind),
			None => noncell_rows,
		};

		println!("Layer: {}, layer_height: {}, base_row_id: {}, kind_base_row_pos: {}", layer_name, layer_height, next_base_row_id, next_kind_base_row_pos);
		
		let cl = CorticalRegionLayer {
			name : layer_name,
			cell: cell,
			base_row_id: next_base_row_id, 
			kind_base_row_pos: next_kind_base_row_pos,
			height: layer_height,
			flags: flags,
		};

		self.add(cl);
	}

	pub fn add(&mut self, layer: CorticalRegionLayer) {
		self.layers.insert(layer.name, layer);
	}

	pub fn width() -> u8 {
		panic!("not implemented");
	}

	pub fn cell_kind_row_count(&self, cell_kind: &CellKind) -> u8 {
		let mut count = 0u8;
		for (_, layer) in self.layers.iter() {
			match layer.cell {
				Some(ref protocell) => match &protocell.cell_kind {
					ref cell_kind => count += layer.height,
				},
				None => (),
			}
		}
		count
	}

	pub fn total_height(&self) -> u8 {
		let mut total_height = 0u8;
		for (_, layer) in self.layers.iter() {
			total_height += layer.height;
		}
		total_height
	}
 
	pub fn height(&self) -> (u8, u8) {
		let mut noncell_rows = 0u8;
		let mut cell_rows = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.cell {
				None => noncell_rows += layer.height,
				Some(_) => cell_rows += layer.height,
			}
		}
		(noncell_rows, cell_rows)
	}

	pub fn row_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		let mut row_ids = Vec::new();
		for &layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in range(l.base_row_id, l.base_row_id + l.height) {
					row_ids.push(i);
				}
		}
		row_ids
	}

	pub fn src_row_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_layer_names = self[layer_name].src_layer_names(den_type);
		
		self.row_ids(src_layer_names)
 	}

 	pub fn col_input_row(&self) -> u8 {
 		for (layer_name, layer) in self.layers.iter() {

 		}
 		5
 	}

 	/*pub fn kind_row_ids(&self, layer_name: &'static str) -> Vec<u8> {

		let l = &self[layer_name];
		let mut row_ids = Vec::new();
			for i in range(l.base_row_id, l.base_row_id + l.height) {
				row_ids.push(i);
			}
		return row_ids;
	}

	pub fn kind_src_row_ids(&self, layer_name: &'static str) -> Vec<u8> {
		let src_layer_names = self[layer_name].src_layer_names();
		
		let mut src_row_ids = Vec::new();

		for &src_row_name in src_layer_names.iter() {
			src_row_ids.push_all(self.kind_row_ids(src_row_name).as_slice());
		}

		//println!("CorticalRegion::layer_srcs_row_ids(): (name:sources:idxs) [{}]:{:?}:{:?}", layer_name, src_layer_names, src_row_ids);
		
		src_row_ids
 	}*/

}

impl Index<&'static str> for CorticalRegion
{
    type Output = CorticalRegionLayer;

    fn index<'a>(&'a self, index: &&'static str) -> &'a CorticalRegionLayer {
        self.layers.get(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}

impl IndexMut<&'static str> for CorticalRegion
{
    type Output = CorticalRegionLayer;

    fn index_mut<'a>(&'a mut self, index: &&'static str) -> &'a mut CorticalRegionLayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}





#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum CorticalRegionType {
	Associational,
	Sensory,
	Motor,
}



/* AxonScope 
	
	Interlaminar(
		Distal Dendrite Input Layers,
		Proximal Dendrite Input Layers,
		Cell Type
	)

*/


/*fn increment_row_index(mut cri: u8, by: u8) -> u8 {
	cri += by;
	cri
}*/
