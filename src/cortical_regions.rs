// CorticalArea for the specifics
// CorticalRegion to define shit

//use protocell::{  };
use cortical_region_layer as layer;
use cortical_region_layer::{ CorticalRegionLayer, LayerFlags };
use protocell::{ CellKind, Protocell, DendriteKind };

use std::collections::{ self, HashMap };
use std::collections::hash_state::{ HashState };
use std::num;
use std::ops::{ Index, IndexMut, Range };
use std::hash::{ self, Hash, SipHasher, Hasher };



//#[derive(Copy)]
pub struct CorticalRegions {
	pub hash_map: HashMap<CorticalRegionKind, CorticalRegion>,
}

impl CorticalRegions {
	pub fn new() -> CorticalRegions {
		CorticalRegions {
			hash_map: HashMap::new(),
		}
	}

	pub fn height(&self, cr_type: CorticalRegionKind) -> (u8, u8) {
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

	pub fn height_total(&self, cr_type: CorticalRegionKind) -> u8 {
		let (hacr, hcr) = self.height(cr_type);
		hacr + hcr
	}

	pub fn add(&mut self, cr: CorticalRegion) {
		self.hash_map.insert(cr.kind.clone(), cr);
	}
}

impl Index<CorticalRegionKind> for CorticalRegions
{
    type Output = CorticalRegion;

    fn index<'a>(&'a self, index: &CorticalRegionKind) -> &'a CorticalRegion {
        self.hash_map.get(index).expect("Invalid region name.")
    }
}

impl IndexMut<CorticalRegionKind> for CorticalRegions
{
    fn index_mut<'a>(&'a mut self, index: &CorticalRegionKind) -> &'a mut CorticalRegion {
        self.hash_map.get_mut(index).expect("Invalid region name.")
    }
}



pub struct CorticalRegion {
	layers: HashMap<&'static str, CorticalRegionLayer>,
	cellular_layer_kind_lists: HashMap<CellKind, Vec<&'static str>>,
	cellular_layer_kind_base_rows: HashMap<CellKind, u8>,
	pub kind: CorticalRegionKind,
	finalized: bool,
}

impl CorticalRegion {
	pub fn new (kind: CorticalRegionKind)  -> CorticalRegion {
		/*let mut next_row_id = HashMap::new();
		next_row_id.insert(CellKind::Pyramidal, 0);
		next_row_id.insert(CellKind::AspinyStellate, 0);
		next_row_id.insert(CellKind::SpinyStellate, 0);*/
	
		CorticalRegion { 
			layers: HashMap::new(),
			cellular_layer_kind_lists: HashMap::new(),
			cellular_layer_kind_base_rows: HashMap::new(),
			kind: kind,
			finalized: false,
		}
	}

	pub fn layer(
					mut self, 
					layer_name: &'static str,
					layer_height: u8,
					flags: LayerFlags,
					cell: Option<Protocell>,
	) -> CorticalRegion {
		let (noncell_rows, cell_rows) = self.height();

		//let next_base_row_pos = self.height_total();

		let next_kind_base_row_pos = match cell {
			Some(ref protocell) => self.height_cell_kind(&protocell.cell_kind),
			None => noncell_rows,
		};

		//println!("Layer: {}, layer_height: {}, base_row_pos: {}, kind_base_row_pos: {}", layer_name, layer_height, next_base_row_pos, next_kind_base_row_pos);
		
		let cl = CorticalRegionLayer {
			name : layer_name,
			cell: cell,
			base_row_pos: 0, 
			kind_base_row_pos: next_kind_base_row_pos,
			height: layer_height,
			flags: flags,
		};

		self.add(cl);
		self
	}

	pub fn add(&mut self, mut layer: CorticalRegionLayer) {

		/*let ck_tmp = match layer.cell {
			Some(ref cell) 	=> cell.cell_kind.clone(),
			None 			=> CellKind::Nada,
		};*/
		if self.finalized {
			panic!("cortical_regions::CorticalRegion::add(): Cannot add new layers after region is finalized.");
		}
		
		match layer.cell {

			Some(ref cell) => {
				let cell_kind = cell.cell_kind.clone();

				let ck_vec_opt: Option<&mut Vec<&'static str>> = if self.cellular_layer_kind_lists.contains_key(&cell_kind) {
					self.cellular_layer_kind_lists.get_mut(&cell_kind)
				} else {
					self.cellular_layer_kind_lists.insert(cell_kind.clone(), Vec::new());
					self.cellular_layer_kind_lists.get_mut(&cell_kind)
				};

				match ck_vec_opt {

					Some(vec) => {
						
						layer.kind_base_row_pos = num::cast(vec.len()).expect("cortical_regions::CorticalRegion::add()");
						//print!("\n{:?} base_row_pos: {}", cell_kind, layer.kind_base_row_pos);

						for i in range(0, layer.height) {							 
							vec.push(layer.name);
							//print!("\nAdding {} to list of {:?}", layer.name, cell_kind);
						}

						//print!("\n{:?} list len: {}", cell_kind, vec.len());
					},
					None => (),
				}
			},
			None => (),
		};

		self.layers.insert(layer.name, layer);

		//print!("\nLooking for cell_kind:{:?}", &ck_tmp);

		/*match self.cellular_layer_kind_lists.get(&ck_tmp) {
			Some(vec) 	=> print!("\nFound Vector with len: {}",vec.len()),
			None 		=> print!("\nVector NOT FOUND"),
		};*/
	}

	pub fn base_row_cell_kind(&self, cell_kind: &CellKind) -> u8 {
		match self.cellular_layer_kind_base_rows.get(cell_kind) {
			Some(base_row) 	=> base_row.clone(),
			None 			=> panic!("CorticalRegion::base_row_cell_king(): Base row for type not found"),
		}
	}

	pub fn height_cell_kind(&self, cell_kind: &CellKind) -> u8 {
		let mut count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.cell {
				Some(ref protocell) => {
					if &protocell.cell_kind == cell_kind {
						count += layer.height;
					} else {
						//print!("\n{:?} didn't match {:?}", protocell.cell_kind, cell_kind);
					}
				},
				None => (),
			}
		}

		let mut count2 = match self.cellular_layer_kind_lists.get(cell_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		//print!("\nCKRC: kind: {:?} -> count = {}, count2 = {}", &cell_kind, count, count2);

		assert!(count as usize == count2, "cortical_regions::CorticalRegion::height_cell_kind(): mismatch");

		count
	}

	pub fn finalize(&mut self) {
		if self.finalized {
			return;
		} else {
			self.finalized = true;
		}

		let (mut base_cel_row, _) = self.height();

		for (cell_kind, list) in &self.cellular_layer_kind_lists {
			self.cellular_layer_kind_base_rows.insert(cell_kind.clone(), base_cel_row);
			print!("\n 	Finalize: adding cell type: '{:?}', len: {}, base_cel_row: {}", cell_kind, list.len(), base_cel_row);
			assert!(list.len() == self.height_cell_kind(&cell_kind) as usize);
			base_cel_row += num::cast(list.len()).expect("cortical_region::CorticalRegion::finalize()");
		}

		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.cell {
				&Some(ref protocell) => {
					layer.base_row_pos = self.cellular_layer_kind_base_rows[protocell.cell_kind] + layer.kind_base_row_pos;
					print!("\n 	Finalize: adding layer: {}, kind: {:?}, base_row_id: {}", layer_name, &protocell.cell_kind, layer.base_row_pos);
				},
				&None => {
					layer.base_row_pos = layer.kind_base_row_pos;
					print!("\n 	Finalize: adding layer: {}, kind: {}, base_row_id: {}", layer_name, "Axon", layer.base_row_pos);
				},
			}
		}
	}


	pub fn height_total(&self) -> u8 {
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

	pub fn layers(&self) -> &HashMap<&'static str, CorticalRegionLayer> {
		&self.layers
	}

	pub fn rows_by_layer_name(&self, cell_kind: &CellKind) -> Option<&Vec<&'static str>> {
		self.cellular_layer_kind_lists.get(cell_kind)
	}

	pub fn row_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		if !self.finalized {
			panic!("CorticalRegion must be finalized with finalize() before row_ids can be called.");
		}
		let mut row_ids = Vec::new();

		for &layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in range(l.base_row_pos, l.base_row_pos + l.height) {
					row_ids.push(i);
				}
		}

		row_ids
	}

	pub fn src_row_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_layer_names = self[layer_name].src_layer_names(den_type);
		
		self.row_ids(src_layer_names)
 	}

 	pub fn col_input_layer(&self) -> Option<CorticalRegionLayer> {
 		let mut input_layer: Option<CorticalRegionLayer> = None;
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.flags & layer::COLUMN_INPUT) == layer::COLUMN_INPUT {
 				input_layer = Some(layer.clone());
 			}
 		}

		input_layer		
 	}

 	pub fn col_output_rows(&self) -> Vec<u8> {
 		let mut output_rows: Vec<u8> = Vec::with_capacity(4);
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.flags & layer::COLUMN_OUTPUT) == layer::COLUMN_OUTPUT {
 				let v = self.row_ids(vec![layer.name]);
 				output_rows.push_all(v.as_slice());
 			}
 		}

		output_rows		
 	}
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
    fn index_mut<'a>(&'a mut self, index: &&'static str) -> &'a mut CorticalRegionLayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum CorticalRegionKind {
	Associational,
	Sensory,
	Motor,
}


/*pub struct CorticalRegion {
	pub layers: HashMap<&'static str, CorticalRegionLayer>,
	pub kind: CorticalRegionKind,
}

impl CorticalRegion {
	pub fn new (kind: CorticalRegionKind)  -> CorticalRegion {
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

		let next_base_row_pos = self.total_height();

		let next_kind_base_row_pos = match cell {
			Some(ref protocell) => self.height_cell_kind(&protocell.cell_kind),
			None => noncell_rows,
		};

		println!("Layer: {}, layer_height: {}, base_row_pos: {}, kind_base_row_pos: {}", layer_name, layer_height, next_base_row_pos, next_kind_base_row_pos);
		
		let cl = CorticalRegionLayer {
			name : layer_name,
			cell: cell,
			base_row_pos: next_base_row_pos, 
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

	pub fn height_cell_kind(&self, cell_kind: &CellKind) -> u8 {
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
				for i in range(l.base_row_pos, l.base_row_pos + l.height) {
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
}*/



 	/*pub fn kind_row_ids(&self, layer_name: &'static str) -> Vec<u8> {

		let l = &self[layer_name];
		let mut row_ids = Vec::new();
			for i in range(l.base_row_pos, l.base_row_pos + l.height) {
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
