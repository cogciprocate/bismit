// ProtoArea for the specifics
// ProtoRegion to define prototypical shit

//use proto::cell::{  };
use proto::layer as layer;
use proto::layer::{ ProtoLayer, ProtoLayerFlags, AxonKind };
use proto::layer::ProtoLayerKind::{ self, Cellular, Axonal };
use proto::cell::{ CellKind, Protocell, DendriteKind };

use std;
use std::collections::{ self, HashMap, BTreeMap };
use std::collections::hash_state::{ HashState };
use num;
use std::ops::{ Index, IndexMut, Range };
use std::hash::{ self, Hash, SipHasher, Hasher };



//#[derive(Copy)]
pub struct ProtoRegions {
	pub hash_map: HashMap<ProtoRegionKind, ProtoRegion>,
}

impl ProtoRegions {
	pub fn new() -> ProtoRegions {
		ProtoRegions {
			hash_map: HashMap::new(),
		}
	}

	/*pub fn depth(&self, cr_type: ProtoRegionKind) -> (u8, u8) {
		let mut depth_axonal_rows = 0u8;		//	Integererregional
		let mut depth_cellular_rows = 0u8;			//	Integererlaminar

		for (region_type, region) in self.hash_map.iter() {					// CHANGE TO FILTER
			if *region_type == cr_type {							//
				let (noncell, cell) = region.depth_axonal_and_cellular();
				depth_axonal_rows += noncell;
				depth_cellular_rows += cell;
				//println!("*** noncell: {}, cell: {} ***", noncell, cell);
			}
		}

		(depth_axonal_rows, depth_cellular_rows)
	}*/

	/*pub fn depth_total(&self, cr_type: ProtoRegionKind) -> u8 {
		let (hacr, hcr) = self.depth(cr_type);
		hacr + hcr
	}*/

	pub fn add(&mut self, cr: ProtoRegion) {
		self.hash_map.insert(cr.kind.clone(), cr);
	}
}

impl<'b> Index<&'b ProtoRegionKind> for ProtoRegions
{
    type Output = ProtoRegion;

    fn index<'a>(&'a self, index: &'b ProtoRegionKind) -> &'a ProtoRegion {
        self.hash_map.get(index).expect("Invalid region name.")
    }
}

impl<'b> IndexMut<&'b ProtoRegionKind> for ProtoRegions
{
    fn index_mut<'a>(&'a mut self, index: &'b ProtoRegionKind) -> &'a mut ProtoRegion {
        self.hash_map.get_mut(index).expect("Invalid region name.")
    }
}



/* CORTICALREGION {}
	- [incomplete] THIS NEEDS TO BE STORED IN A DATABASE OR SOMETHING - GETTING TOO UNRULY
		- Or... redesign using a trait that can handle CellKind and AxonKind both
			- Also could merge the two and have one or the other dominant
	- [incomplete] (cel, axn)_layer_kind_row_lists needs to be redone asap
*/
pub struct ProtoRegion {
	layers: HashMap<&'static str, ProtoLayer>,
	cel_layer_kind_row_lists: HashMap<CellKind, Vec<&'static str>>,
	cel_layer_kind_base_row_ids: HashMap<CellKind, u8>,
	axn_layer_kind_row_lists: HashMap<AxonKind, Vec<&'static str>>,
	axn_layer_kind_base_row_ids: HashMap<AxonKind, u8>,
	row_map: BTreeMap<u8, &'static str>,
	pub kind: ProtoRegionKind,
	frozen: bool,
	hrz_demarc: u8,
}

impl ProtoRegion {
	pub fn new (kind: ProtoRegionKind)  -> ProtoRegion {
		/*let mut next_row_id = HashMap::new();
		next_row_id.insert(CellKind::Pyramidal, 0);
		next_row_id.insert(CellKind::PeakColumn, 0);
		next_row_id.insert(CellKind::SpinyStellate, 0);*/
	
		ProtoRegion { 
			layers: HashMap::new(),
			cel_layer_kind_row_lists: HashMap::new(),
			cel_layer_kind_base_row_ids: HashMap::new(),
			axn_layer_kind_row_lists: HashMap::new(),
			axn_layer_kind_base_row_ids: HashMap::new(),
			kind: kind,
			frozen: false,
			row_map: BTreeMap::new(),
			hrz_demarc: 0,
		}
	}

	pub fn layer(
					mut self, 
					layer_name: &'static str,
					layer_depth: u8,
					flags: ProtoLayerFlags,
					kind: ProtoLayerKind,
	) -> ProtoRegion {

		let next_kind_base_row_pos = match kind {
			Cellular(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			Axonal(ref axon_kind) => self.depth_axon_kind(&axon_kind),
		};
		
		let cl = ProtoLayer {
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

	/* PROTOREGION::ADD()
		- [incomplete] NEED TO CHECK FOR DUPLICATE LAYERS!
	*/
	pub fn add(&mut self, mut layer: ProtoLayer) {

		/*let ck_tmp = match layer.kind {
			Some(ref kind) 	=> kind.cell_kind.clone(),
			None 			=> CellKind::Nada,
		};*/
		if self.frozen {
			panic!("protoregions::ProtoRegion::add(): Cannot add new layers after region is frozen.");
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
						//layer.kind_base_row_pos = std::num::cast(vec.len()).expect("protoregions::ProtoRegion::add()");
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

			Axonal(ref axon_kind) => {
				let ck_vec_opt: Option<&mut Vec<&'static str>> = if self.axn_layer_kind_row_lists.contains_key(&axon_kind) {
					self.axn_layer_kind_row_lists.get_mut(&axon_kind)
				} else {
					self.axn_layer_kind_row_lists.insert(axon_kind.clone(), Vec::new());
					self.axn_layer_kind_row_lists.get_mut(&axon_kind)
				};

				match ck_vec_opt {

					Some(vec) => {
						
						layer.kind_base_row_pos = vec.len() as u8;

						for i in 0..layer.depth {							 
							vec.push(layer.name);
						}

					},
					None => (),
				}
			},
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
			None 			=> panic!("ProtoRegion::base_row_cell_king(): Base row for type not found"),
		}
	}

	/*pub fn depth_axonal_and_cellular(&self) -> (u8, u8) {
		let mut axon_rows = 0u8;
		let mut cell_rows = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				Axonal(_) => axon_rows += layer.depth,
				Cellular(_) => cell_rows += layer.depth,
			}
		}

		(axon_rows, cell_rows)
	}*/

	pub fn depth_axonal_spatial(&self) -> u8 {
		let mut axon_rows = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				Axonal(ref axon_kind) => {
					match axon_kind {
						&AxonKind::Spatial => axon_rows += layer.depth,
						_	=> (),
					}
				},
				Cellular(_) => (),
			}
		}

		axon_rows
	}

	pub fn depth_axonal_horizontal(&self) -> u8 {
		let mut axon_rows = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				Axonal(ref axon_kind) => {
					match axon_kind {
						&AxonKind::Horizontal => axon_rows += layer.depth,
						_	=> (),
					}
				},
				Cellular(_) => (),
			}
		}

		axon_rows
	} 

	pub fn depth_cellular(&self) -> u8 {
		let mut cell_rows = 0u8;

		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				Axonal(_) => (),
				Cellular(_) => cell_rows += layer.depth,
			}
		}

		cell_rows
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

		assert!(count as usize == count2, "protoregions::ProtoRegion::depth_cell_kind(): mismatch");

		count
	}

	pub fn depth_axon_kind(&self, axon_kind: &AxonKind) -> u8 {
		let mut count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.kind {

				Axonal(ref ak) => {
					if ak == axon_kind {
						count += layer.depth;
					}
				},

				Cellular(_) => {}
			}
		}

		let mut count2 = match self.axn_layer_kind_row_lists.get(axon_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		assert!(count as usize == count2, "protoregions::ProtoRegion::depth_axon_kind(): mismatch");

		count
	}

	/*pub fn depth_row_total(&self) -> u8 {
		let mut total_depth = 0u8;

		for (_, layer) in self.layers.iter() {
			total_depth += layer.depth;
		}

		total_depth
	}*/
 

 	/* PROTOREGION::FREEZE():
 		- What a mess...
		- Need to revamp how axon_types and cell_types are stored before we can do much with it
			- cel_layer_kind_row_lists being a vector needs to change asap
 	*/
	pub fn freeze(&mut self) {
		if self.frozen {
			return;
		} else {
			self.frozen = true;
		}


		/* (0) START COUNTER FOR ABSOLUTE BASE ROWS */
		let mut next_base_row = 0u8;

		/* (1) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL SPATIAL LAYER KINDS */	
		for (axon_kind, list) in &self.axn_layer_kind_row_lists {
			match axon_kind {
				&AxonKind::Spatial => {
					self.axn_layer_kind_base_row_ids.insert(axon_kind.clone(), next_base_row);
					print!("\n    Adding Axon Kind: '{:?}', len: {}, kind_base_row: {}", axon_kind, list.len(), next_base_row);
					assert!(list.len() == self.depth_axon_kind(&axon_kind) as usize);
					next_base_row += list.len() as u8;
				},
				_ => ()
			}
		}

		/* (2) ADD ABSOLUTE BASE_ROW_IDS FOR ALL CELLULAR LAYER KINDS */
		for (cell_kind, list) in &self.cel_layer_kind_row_lists {
			self.cel_layer_kind_base_row_ids.insert(cell_kind.clone(), next_base_row);
			print!("\n    Adding Cell Kind: '{:?}', len: {}, kind_base_row: {}", cell_kind, list.len(), next_base_row);
			assert!(list.len() == self.depth_cell_kind(&cell_kind) as usize);
			next_base_row += list.len() as u8;
			//next_base_row += std::num::cast::<usize, u8>(list.len()).expect("cortical_region::ProtoRegion::freeze()");
		}

		/* (2b) SAVE DEMARCATION BETWEEN VERTICAL (SPATIAL) AND HORIZONTAL ROWS */
		self.hrz_demarc = next_base_row;

		/* (3) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL HORIZONTAL LAYER KINDS */	
		for (axon_kind, list) in &self.axn_layer_kind_row_lists {
			match axon_kind {
				&AxonKind::Horizontal => {
					self.axn_layer_kind_base_row_ids.insert(axon_kind.clone(), next_base_row);
					print!("\n    Adding Axon Kind: '{:?}', len: {}, kind_base_row: {}", axon_kind, list.len(), next_base_row);
					assert!(list.len() == self.depth_axon_kind(&axon_kind) as usize);
					next_base_row += list.len() as u8;
				},
				_ => ()
			}
		}

		print!("\n");

		/* (4) SET BASE ROW POSITION ON INDIVIDUAL NON-HORIZONTAL LAYERS */
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {

				&Cellular(ref protocell) => {
					layer.base_row_pos = self.cel_layer_kind_base_row_ids[&protocell.cell_kind] + layer.kind_base_row_pos;
					print!("\n    <{}>: CellKind::{:?} ", layer_name, &protocell.cell_kind);
				},

				&Axonal(ref axon_kind) => {
					match axon_kind {
						&AxonKind::Horizontal => continue,

						_ => {
							layer.base_row_pos = self.axn_layer_kind_base_row_ids[axon_kind] + layer.kind_base_row_pos;
							print!("\n    <{}>: AxonKind::{:?} ", layer_name, axon_kind);
						},
					}
				},
			}

			for i in layer.base_row_pos..(layer.base_row_pos + layer.depth()) {
				self.row_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
		}

		/* (5) SET BASE ROW POSITION ON INDIVIDUAL HORIZONTAL LAYERS */
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {
				&Cellular(ref protocell) => continue,

				&Axonal(ref axon_kind) => {
					match axon_kind {
						&AxonKind::Horizontal => {
							layer.base_row_pos = self.axn_layer_kind_base_row_ids[axon_kind] + layer.kind_base_row_pos;
							print!("\n    <{}>: AxonKind::{:?} ", layer_name, axon_kind);
						},

						_ => continue,
					}
				},
			}

			for i in layer.base_row_pos..(layer.base_row_pos + layer.depth()) {
				self.row_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
		}

		/* (6) MARVEL AT THE MOST CONVOLUTED FUNCTION EVER */
		print!("\n");
	}


	pub fn layers(&self) -> &HashMap<&'static str, ProtoLayer> {
		&self.layers
	}

	pub fn rows_by_layer_name(&self, cell_kind: &CellKind) -> Option<&Vec<&'static str>> {
		self.cel_layer_kind_row_lists.get(cell_kind)
	}

	pub fn row_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		if !self.frozen {
			panic!("ProtoRegion must be frozen with freeze() before row_ids can be called.");
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

 	pub fn col_input_layer(&self) -> Option<ProtoLayer> {
 		let mut input_layer: Option<ProtoLayer> = None;
 		
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

	pub fn hrz_demarc(&self) -> u8 {
		self.hrz_demarc
	}
}

impl<'b> Index<&'b&'static str> for ProtoRegion
{
    type Output = ProtoLayer;

    fn index<'a>(&'a self, index: &'b&'static str) -> &'a ProtoLayer {
        self.layers.get(index).unwrap_or_else(|| panic!("[protoregions::ProtoRegion::index(): invalid layer name: \"{}\"]", index))
    }
}

impl<'b> IndexMut<&'b&'static str> for ProtoRegion
{
    fn index_mut<'a>(&'a mut self, index: &'b&'static str) -> &'a mut ProtoLayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[protoregions::ProtoRegion::index(): invalid layer name: \"{}\"]", index))
    }
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ProtoRegionKind {
	Associational,
	Sensory,
	Motor,
}


/*pub struct ProtoRegion {
	pub layers: HashMap<&'static str, ProtoLayer>,
	pub kind: ProtoRegionKind,
}

impl ProtoRegion {
	pub fn new (kind: ProtoRegionKind)  -> ProtoRegion {
		let mut next_row_id = HashMap::new();
		next_row_id.insert(CellKind::Pyramidal, 0);
		next_row_id.insert(CellKind::PeakColumn, 0);
		next_row_id.insert(CellKind::SpinyStellate, 0);
	
		ProtoRegion { 
			layers: HashMap::new(),
			kind: kind,
		}
	}

	pub fn new_layer(
					&mut self, 
					layer_name: &'static str,
					layer_depth: u8,
					flags: ProtoLayerFlags,
					cell: Option<Protocell>,
	) {
		let (noncell_rows, cell_rows) = self.depth();

		let next_base_row_pos = self.total_depth();

		let next_kind_base_row_pos = match cell {
			Some(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			None => noncell_rows,
		};

		println!("ProtoLayer: {}, layer_depth: {}, base_row_pos: {}, kind_base_row_pos: {}", layer_name, layer_depth, next_base_row_pos, next_kind_base_row_pos);
		
		let cl = ProtoLayer {
			name : layer_name,
			cell: cell,
			base_row_pos: next_base_row_pos, 
			kind_base_row_pos: next_kind_base_row_pos,
			depth: layer_depth,
			flags: flags,
		};

		self.add(cl);
	}

	pub fn add(&mut self, layer: ProtoLayer) {
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

impl Index<&'static str> for ProtoRegion
{
    type Output = ProtoLayer;

    fn index<'a>(&'a self, index: &&'static str) -> &'a ProtoLayer {
        self.layers.get(index).unwrap_or_else(|| panic!("[protoregions::ProtoRegion::index(): invalid layer name: \"{}\"]", index))
    }
}

impl IndexMut<&'static str> for ProtoRegion
{
    type Output = ProtoLayer;

    fn index_mut<'a>(&'a mut self, index: &&'static str) -> &'a mut ProtoLayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[protoregions::ProtoRegion::index(): invalid layer name: \"{}\"]", index))
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

		//println!("ProtoRegion::layer_srcs_row_ids(): (name:sources:idxs) [{}]:{:?}:{:?}", layer_name, src_layer_names, src_row_ids);
		
		src_row_ids
 	}*/


/* AxonScope 
	
	Integererlaminar(
		Distal Dendrite Input ProtoLayers,
		Proximal Dendrite Input ProtoLayers,
		Cell Type
	)

*/


/*fn increment_row_index(mut cri: u8, by: u8) -> u8 {
	cri += by;
	cri
}*/
