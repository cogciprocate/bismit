// CorticalArea for the specifics
// CorticalRegion to define shit

//use protocell::{  };
use cortical_region_layer as layer;
use cortical_region_layer::{ Layer, LayerFlags };
use cortical_region_layer::LayerKind::{ self, Cellular, Axonal };
use protocell::{ CellKind, Protocell, DendriteKind };

use std;
use std::collections::{ self, HashMap, BTreeMap };
use std::collections::hash_state::{ HashState };
use num;
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

	pub fn depth(&self, cr_type: CorticalRegionKind) -> (u8, u8) {
		let mut depth_noncellular_rows = 0u8;		//	Integererregional
		let mut depth_cellular_rows = 0u8;			//	Integererlaminar

		for (region_type, region) in self.hash_map.iter() {					// CHANGE TO FILTER
			if *region_type == cr_type {							//
				let (noncell, cell) = region.depth();
				depth_noncellular_rows += noncell;
				depth_cellular_rows += cell;
				//println!("*** noncell: {}, cell: {} ***", noncell, cell);
			}
		}

		(depth_noncellular_rows, depth_cellular_rows)
	}

	pub fn depth_total(&self, cr_type: CorticalRegionKind) -> u8 {
		let (hacr, hcr) = self.depth(cr_type);
		hacr + hcr
	}

	pub fn add(&mut self, cr: CorticalRegion) {
		self.hash_map.insert(cr.kind.clone(), cr);
	}
}

impl<'b> Index<&'b CorticalRegionKind> for CorticalRegions
{
    type Output = CorticalRegion;

    fn index<'a>(&'a self, index: &'b CorticalRegionKind) -> &'a CorticalRegion {
        self.hash_map.get(index).expect("Invalid region name.")
    }
}

impl<'b> IndexMut<&'b CorticalRegionKind> for CorticalRegions
{
    fn index_mut<'a>(&'a mut self, index: &'b CorticalRegionKind) -> &'a mut CorticalRegion {
        self.hash_map.get_mut(index).expect("Invalid region name.")
    }
}



pub struct CorticalRegion {
	layers: HashMap<&'static str, Layer>,
	cel_layer_kind_row_lists: HashMap<CellKind, Vec<&'static str>>,
	cel_layer_kind_base_row_ids: HashMap<CellKind, u8>,
	row_map: BTreeMap<u8, &'static str>,
	pub kind: CorticalRegionKind,
	finalized: bool,
}

impl CorticalRegion {
	pub fn new (kind: CorticalRegionKind)  -> CorticalRegion {
		/*let mut next_row_id = HashMap::new();
		next_row_id.insert(CellKind::Pyramidals, 0);
		next_row_id.insert(CellKind::PeakColumn, 0);
		next_row_id.insert(CellKind::SpinyStellate, 0);*/
	
		CorticalRegion { 
			layers: HashMap::new(),
			cel_layer_kind_row_lists: HashMap::new(),
			cel_layer_kind_base_row_ids: HashMap::new(),
			kind: kind,
			finalized: false,
			row_map: BTreeMap::new(),
		}
	}

	pub fn layer(
					mut self, 
					layer_name: &'static str,
					layer_depth: u8,
					flags: LayerFlags,
					kind: LayerKind,
	) -> CorticalRegion {
		let (noncell_rows, cell_rows) = self.depth();

		//let next_base_row_pos = self.depth_total();

		let next_kind_base_row_pos = match kind {
			Cellular(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			Axonal(_) => noncell_rows,
		};

		//println!("Layer: {}, layer_depth: {}, base_row_pos: {}, kind_base_row_pos: {}", layer_name, layer_depth, next_base_row_pos, next_kind_base_row_pos);
		
		let cl = Layer {
			name : layer_name,
			kind: kind,
			base_row_pos: 0, 
			kind_base_row_pos: next_kind_base_row_pos,
			depth: layer_depth,
			flags: flags,
		};

		self.add(cl);
		self
	}

	pub fn add(&mut self, mut layer: Layer) {

		/*let ck_tmp = match layer.kind {
			Some(ref kind) 	=> kind.cell_kind.clone(),
			None 			=> CellKind::Nada,
		};*/
		if self.finalized {
			panic!("cortical_regions::CorticalRegion::add(): Cannot add new layers after region is finalized.");
		}
		
		match layer.kind {

			Cellular(ref cell) => {
				let cell_kind = cell.cell_kind.clone();

				let ck_vec_opt: Option<&mut Vec<&'static str>> = if self.cel_layer_kind_row_lists.contains_key(&cell_kind) {
					self.cel_layer_kind_row_lists.get_mut(&cell_kind)
				} else {
					self.cel_layer_kind_row_lists.insert(cell_kind.clone(), Vec::new());
					self.cel_layer_kind_row_lists.get_mut(&cell_kind)
				};

				match ck_vec_opt {

					Some(vec) => {
						
						layer.kind_base_row_pos = vec.len() as u8;
						//layer.kind_base_row_pos = std::num::cast(vec.len()).expect("cortical_regions::CorticalRegion::add()");
						//print!("\n{:?} base_row_pos: {}", cell_kind, layer.kind_base_row_pos);

						for i in 0..layer.depth {							 
							vec.push(layer.name);
							//print!("\nAdding {} to list of {:?}", layer.name, cell_kind);
						}

						//print!("\n{:?} list len: {}", cell_kind, vec.len());
					},
					None => (),
				}
			},
			Axonal(_) => (),
		};

		self.layers.insert(layer.name, layer);

		//print!("\nLooking for cell_kind:{:?}", &ck_tmp);

		/*match self.cel_layer_kind_row_lists.get(&ck_tmp) {
			Some(vec) 	=> print!("\nFound Vector with len: {}",vec.len()),
			None 		=> print!("\nVector NOT FOUND"),
		};*/
	}

	pub fn base_row(&self, layer_name: &'static str) -> u8 {
		let ref layer = self.layers[layer_name];
		layer.base_row_pos
	}

	pub fn base_row_cell_kind(&self, cell_kind: &CellKind) -> u8 {
		match self.cel_layer_kind_base_row_ids.get(cell_kind) {
			Some(base_row) 	=> base_row.clone(),
			None 			=> panic!("CorticalRegion::base_row_cell_king(): Base row for type not found"),
		}
	}

	pub fn depth_cell_kind(&self, cell_kind: &CellKind) -> u8 {
		let mut count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.kind {
				Cellular(ref protocell) => {
					if &protocell.cell_kind == cell_kind {
						count += layer.depth;
					} else {
						//print!("\n{:?} didn't match {:?}", protocell.cell_kind, cell_kind);
					}
				},
				Axonal(_) => (),
			}
		}

		let mut count2 = match self.cel_layer_kind_row_lists.get(cell_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		//print!("\nCKRC: kind: {:?} -> count = {}, count2 = {}", &cell_kind, count, count2);

		assert!(count as usize == count2, "cortical_regions::CorticalRegion::depth_cell_kind(): mismatch");

		count
	}

	pub fn finalize(&mut self) {
		if self.finalized {
			return;
		} else {
			self.finalized = true;
		}

		let (mut base_cel_row, _) = self.depth();

		/* (1) ADD A BASE AXON ROW FOR EACH CELL KIND (PYR, SPINY, ETC.) */
		for (cell_kind, list) in &self.cel_layer_kind_row_lists {
			self.cel_layer_kind_base_row_ids.insert(cell_kind.clone(), base_cel_row);
			print!("\n   Finalize: adding cell type: '{:?}', len: {}, base_cel_row: {}", cell_kind, list.len(), base_cel_row);
			assert!(list.len() == self.depth_cell_kind(&cell_kind) as usize);
			base_cel_row += list.len() as u8;
			//base_cel_row += std::num::cast::<usize, u8>(list.len()).expect("cortical_region::CorticalRegion::finalize()");
		}

		/* (2) SET BASE ROW POSITION ON NON-HORIZONTAL LAYERS */
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {

				&Cellular(ref protocell) => {
					layer.base_row_pos = self.cel_layer_kind_base_row_ids[&protocell.cell_kind] + layer.kind_base_row_pos;
					print!("\n\tFinalize: adding layer: {}, kind: {:?}, base_row_id: {}", layer_name, &protocell.cell_kind, layer.base_row_pos);
				},

				&Axonal(_) => {
					if (layer.flags & layer::HORIZONTAL) != layer::HORIZONTAL {
						layer.base_row_pos = layer.kind_base_row_pos;
						print!("\n\tFinalize: adding layer: {}, kind: {}, base_row_id: {}", layer_name, "Axon", layer.base_row_pos);
					}
				},

			}

			for i in layer.base_row_pos..(layer.base_row_pos + layer.depth()) {
				self.row_map.insert(i, layer_name);
			}
		}

		/* (3) SET BASE ROW POSITION ON HORIZONTAL LAYERS */
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {
				&Cellular(ref protocell) => { continue; },

				&Axonal(_) => {
					if (layer.flags & layer::HORIZONTAL) == layer::HORIZONTAL {
						layer.base_row_pos = layer.kind_base_row_pos;
						print!("\n\tFinalize: adding layer: {}, kind: {}, base_row_id: {}", layer_name, "Axon", layer.base_row_pos);
					}
				},

			}

			for i in layer.base_row_pos..(layer.base_row_pos + layer.depth()) {
				self.row_map.insert(i, layer_name);
			}
		}
	}


	pub fn depth_total(&self) -> u8 {
		let mut total_depth = 0u8;

		for (_, layer) in self.layers.iter() {
			total_depth += layer.depth;
		}

		total_depth
	}
 
	pub fn depth(&self) -> (u8, u8) {
		let mut noncell_rows = 0u8;
		let mut cell_rows = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				Axonal(_) => noncell_rows += layer.depth,
				Cellular(_) => cell_rows += layer.depth,
			}
		}

		(noncell_rows, cell_rows)
	}

	pub fn layers(&self) -> &HashMap<&'static str, Layer> {
		&self.layers
	}

	pub fn rows_by_layer_name(&self, cell_kind: &CellKind) -> Option<&Vec<&'static str>> {
		self.cel_layer_kind_row_lists.get(cell_kind)
	}

	pub fn row_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		if !self.finalized {
			panic!("CorticalRegion must be finalized with finalize() before row_ids can be called.");
		}

		let mut row_ids = Vec::new();

		for layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in l.base_row_pos..(l.base_row_pos + l.depth) {
					row_ids.push(i);
				}
		}

		row_ids
	}

	pub fn src_row_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_layer_names = self[&layer_name].src_layer_names(den_type);
		
		self.row_ids(src_layer_names)
 	}

 	pub fn col_input_layer(&self) -> Option<Layer> {
 		let mut input_layer: Option<Layer> = None;
 		
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
 				output_rows.push_all(&v);
 			}
 		}

		output_rows		
 	}

 	pub fn row_map(&self) -> BTreeMap<u8, &'static str> {
 		self.row_map.clone()
	}

 	pub fn layer_name(&self, row_id: u8) -> &'static str {
 		match self.row_map.get(&row_id) {
 			Some(ln) 	=> ln,
 			None 		=> "[INVALID LAYER]",
		}

	}
}

impl<'b> Index<&'b&'static str> for CorticalRegion
{
    type Output = Layer;

    fn index<'a>(&'a self, index: &'b&'static str) -> &'a Layer {
        self.layers.get(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}

impl<'b> IndexMut<&'b&'static str> for CorticalRegion
{
    fn index_mut<'a>(&'a mut self, index: &'b&'static str) -> &'a mut Layer {
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
	pub layers: HashMap<&'static str, Layer>,
	pub kind: CorticalRegionKind,
}

impl CorticalRegion {
	pub fn new (kind: CorticalRegionKind)  -> CorticalRegion {
		let mut next_row_id = HashMap::new();
		next_row_id.insert(CellKind::Pyramidals, 0);
		next_row_id.insert(CellKind::PeakColumn, 0);
		next_row_id.insert(CellKind::SpinyStellate, 0);
	
		CorticalRegion { 
			layers: HashMap::new(),
			kind: kind,
		}
	}

	pub fn new_layer(
					&mut self, 
					layer_name: &'static str,
					layer_depth: u8,
					flags: LayerFlags,
					cell: Option<Protocell>,
	) {
		let (noncell_rows, cell_rows) = self.depth();

		let next_base_row_pos = self.total_depth();

		let next_kind_base_row_pos = match cell {
			Some(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			None => noncell_rows,
		};

		println!("Layer: {}, layer_depth: {}, base_row_pos: {}, kind_base_row_pos: {}", layer_name, layer_depth, next_base_row_pos, next_kind_base_row_pos);
		
		let cl = Layer {
			name : layer_name,
			cell: cell,
			base_row_pos: next_base_row_pos, 
			kind_base_row_pos: next_kind_base_row_pos,
			depth: layer_depth,
			flags: flags,
		};

		self.add(cl);
	}

	pub fn add(&mut self, layer: Layer) {
		self.layers.insert(layer.name, layer);
	}

	pub fn width() -> u8 {
		panic!("not implemented");
	}

	pub fn depth_cell_kind(&self, cell_kind: &CellKind) -> u8 {
		let mut count = 0u8;
		for (_, layer) in self.layers.iter() {
			match layer.cell {
				Some(ref protocell) => match &protocell.cell_kind {
					ref cell_kind => count += layer.depth,
				},
				None => (),
			}
		}
		count
	}

	pub fn total_depth(&self) -> u8 {
		let mut total_depth = 0u8;
		for (_, layer) in self.layers.iter() {
			total_depth += layer.depth;
		}
		total_depth
	}
 
	pub fn depth(&self) -> (u8, u8) {
		let mut noncell_rows = 0u8;
		let mut cell_rows = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.cell {
				None => noncell_rows += layer.depth,
				Some(_) => cell_rows += layer.depth,
			}
		}
		(noncell_rows, cell_rows)
	}

	pub fn row_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		let mut row_ids = Vec::new();
		for &layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in range(l.base_row_pos, l.base_row_pos + l.depth) {
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
    type Output = Layer;

    fn index<'a>(&'a self, index: &&'static str) -> &'a Layer {
        self.layers.get(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}

impl IndexMut<&'static str> for CorticalRegion
{
    type Output = Layer;

    fn index_mut<'a>(&'a mut self, index: &&'static str) -> &'a mut Layer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[cortical_regions::CorticalRegion::index(): invalid layer name: \"{}\"]", index))
    }
}*/



 	/*pub fn kind_row_ids(&self, layer_name: &'static str) -> Vec<u8> {

		let l = &self[layer_name];
		let mut row_ids = Vec::new();
			for i in range(l.base_row_pos, l.base_row_pos + l.depth) {
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
	
	Integererlaminar(
		Distal Dendrite Input Layers,
		Proximal Dendrite Input Layers,
		Cell Type
	)

*/


/*fn increment_row_index(mut cri: u8, by: u8) -> u8 {
	cri += by;
	cri
}*/
