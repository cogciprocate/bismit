
use std::collections::{ HashMap, BTreeMap };
use std::ops::{ Index, IndexMut,  };
use std::hash::{ Hasher };

use map::{ self, LayerTags };
use super::{ ProtoareaMap, Protolayer, AxonKind, LayerKind, CellKind, Axonal };


// PROTOLAYERMAP (PROTOREGION) {} <<<<< TODO: SPLIT UP, REDESIGN, AND REFACTOR >>>>>
// - [incomplete] SEPERATE CONCERNS and consolidate similar data structures - GETTING TOO UNRULY
// - redesign using a trait that can handle CellKind and AxonKind both
//    - Also could merge the two and have one or the other dominant
// - [incomplete] (cel, axn)_layer_kind_slc_lists needs to be redone asap
// - should be instances of some new type which manages their lists 

#[derive(Clone)]
pub struct ProtolayerMap {
	pub name: &'static str,
	pub kind: RegionKind,
	layers: HashMap<&'static str, Protolayer>,
	cel_layer_kind_slc_lists: HashMap<CellKind, Vec<&'static str>>,
	cel_layer_kind_base_slc_ids: HashMap<CellKind, u8>,
	axn_layer_kind_slc_lists: HashMap<AxonKind, Vec<&'static str>>,
	axn_layer_kind_base_slc_ids: HashMap<AxonKind, u8>,
	slc_map: BTreeMap<u8, &'static str>,	
	frozen: bool,
	hrz_demarc: u8,
}

impl ProtolayerMap {
	pub fn new (region_name: &'static str, kind: RegionKind)  -> ProtolayerMap {	
		ProtolayerMap { 
			name: region_name,
			kind: kind,
			layers: HashMap::new(),
			cel_layer_kind_slc_lists: HashMap::new(),
			cel_layer_kind_base_slc_ids: HashMap::new(),
			axn_layer_kind_slc_lists: HashMap::new(),
			axn_layer_kind_base_slc_ids: HashMap::new(),
			
			frozen: false,
			slc_map: BTreeMap::new(),
			hrz_demarc: 0,
		}
	}

	pub fn axn_layer(mut self, layer_name: &'static str, tags: LayerTags, axon_kind: AxonKind,
			) -> ProtolayerMap 
	{
		let next_kind_base_slc_id = self.depth_axon_kind(&axon_kind);
		// let layer_depth = if tags.contains(map::OUTPUT) { Some(1) } else { None };
		
		self.add(Protolayer::new(layer_name, Axonal(axon_kind), None, 0,
			next_kind_base_slc_id, tags));
		self
	}

	// [FIXME]: TODO: Change axonal default depth to 'None' so that input source or layer map can set.
	pub fn layer(mut self, layer_name: &'static str, layer_depth: u8, tags: LayerTags, 
			kind: LayerKind) -> ProtolayerMap 
	{
		let (next_kind_base_slc_id, validated_depth) = match kind {
			LayerKind::Cellular(ref protocell) => (self.depth_cell_kind(&protocell.cell_kind), 
				protocell.validate_depth(Some(layer_depth))),
			LayerKind::Axonal(ref axon_kind) => (self.depth_axon_kind(&axon_kind), 
				Some(layer_depth)),
		};
		
		self.add(Protolayer::new(layer_name, kind, validated_depth, 0, next_kind_base_slc_id, tags));
		self
	}

	// [FIXME]: NEED TO CHECK FOR DUPLICATE LAYERS!	
	pub fn add(&mut self, layer: Protolayer) {
		if self.frozen {
			panic!("ProtolayerMap::add(): Cannot add new layers after region is frozen.");
		}

		self.layers.insert(layer.name(), layer);
	}
	

	// SET_LAYER_DEPTH(): ASSUMES PROPER FLAG UNIQUENESS CONSTRAINS ALREADY APPLIED
	fn set_layer_depth(&mut self, tags: LayerTags, depth: u8) {
		for (layer_name, mut layer) in self.layers.iter_mut() {
			if (layer.tags() & tags) == tags {
				//println!(" ##### SETTING LAYER DEPTH FOR LAYER: '{}' TO: {} #####", layer_name, depth);
				layer.set_depth(depth);
			}
		}
	}

	fn set_layer_depths(&mut self, pamap: &ProtoareaMap) {
		// FEEDFORWARD INPUT COMES FROM EFF AREAS, FEEDBACK INPUT COMES FROM AFF AREAS
		self.set_layer_depth(map::FF_IN, pamap.eff_areas().len() as u8);
		self.set_layer_depth(map::FB_IN, pamap.aff_areas().len() as u8);
	} 

 	// 	PROTOREGION::FREEZE():
 	// 		- What a mess...
	// 		- Need to revamp how axon_types and cell_types are stored before we can do much with it
	// 		- cel_layer_kind_slc_lists being a vector needs to change asap
	//		- this probably needs to be handled by the new AreaMap and its ilk
	//
	// 	[FIXME] TODO: VERIFY FLAG UNIQUENESS, APPROPRIATENESS 	
	pub fn freeze(&mut self, pamap: &ProtoareaMap) {
		println!("\nPROTOLAYERMAP: Assembling layer map for area '{}'...", pamap.name);

		if self.frozen {
			return;
		} else {			
			self.frozen = true;
		}

		self.set_layer_depths(pamap);

		self.gen_slc_lists();

		self.build_kind_base_slc_ids();

		self.set_base_slc_ids();
	}


	fn gen_slc_lists(&mut self) {
		for (layer_name, layer) in self.layers.iter_mut() {
			match layer.kind().clone() {
				LayerKind::Cellular(ref cell) => {
					let cell_kind = cell.cell_kind.clone();

					let ck_vec_opt: Option<&mut Vec<&'static str>>
							= if self.cel_layer_kind_slc_lists.contains_key(&cell_kind) 
					{
						self.cel_layer_kind_slc_lists.get_mut(&cell_kind)
					} else {
						self.cel_layer_kind_slc_lists.insert(cell_kind.clone(), Vec::new());
						self.cel_layer_kind_slc_lists.get_mut(&cell_kind)
					};

					match ck_vec_opt {
						Some(vec) => {							
							layer.set_kind_base_slc_id(vec.len() as u8);

							for i in 0..layer.depth_old_tmp() {							 
								vec.push(layer.name());
							}
						},
						None => panic!(),
					}
				},

				LayerKind::Axonal(ref axon_kind) => {
					let ck_vec_opt: Option<&mut Vec<&'static str>>
							= if self.axn_layer_kind_slc_lists.contains_key(&axon_kind) 
					{
						self.axn_layer_kind_slc_lists.get_mut(&axon_kind)
					} else {
						self.axn_layer_kind_slc_lists.insert(axon_kind.clone(), Vec::new());
						self.axn_layer_kind_slc_lists.get_mut(&axon_kind)
					};

					match ck_vec_opt {
						Some(vec) => {							
							layer.set_kind_base_slc_id(vec.len() as u8);

							for i in 0..layer.depth_old_tmp() {							 
								vec.push(layer.name());
							}
						},
						None => panic!(),
					}
				},
			}
		}
	}


	fn build_kind_base_slc_ids(&mut self) {
		// (0) START COUNTER FOR ABSOLUTE BASE ROWS 
		let mut next_base_slc = 0u8;

		// (1) ADD ABSOLUTE BASE_ROW_IDS FOR ALL CELLULAR LAYER KINDS 
		for (cell_kind, list) in &self.cel_layer_kind_slc_lists {
			self.cel_layer_kind_base_slc_ids.insert(cell_kind.clone(), next_base_slc);
			println!("    Adding Cell Kind: '{:?}', len: {}, kind_base_slc: {}", cell_kind, list.len(), next_base_slc);
			assert!(list.len() == self.depth_cell_kind(&cell_kind) as usize);
			next_base_slc += list.len() as u8;
			//next_base_slc += std::num::cast::<usize, u8>(list.len()).expect("cortical_region::ProtolayerMap::freeze()");
		}

		// (2) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL SPATIAL LAYER KINDS 	
		for (axon_kind, list) in &self.axn_layer_kind_slc_lists {
			match axon_kind {
				&AxonKind::Spatial => {
					self.axn_layer_kind_base_slc_ids.insert(axon_kind.clone(), next_base_slc);
					println!("    Adding Axon Kind: '{:?}', len: {}, kind_base_slc: {}", axon_kind, list.len(), next_base_slc);
					assert!(list.len() == self.depth_axon_kind(&axon_kind) as usize);
					next_base_slc += list.len() as u8;
				},
				_ => ()
			}
		}

		// (2b) SAVE DEMARCATION BETWEEN VERTICAL (SPATIAL) AND HORIZONTAL ROWS 
		self.hrz_demarc = next_base_slc;

		// (3) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL HORIZONTAL LAYER KINDS 	
		for (axon_kind, list) in &self.axn_layer_kind_slc_lists {
			match axon_kind {
				&AxonKind::Horizontal => {
					self.axn_layer_kind_base_slc_ids.insert(axon_kind.clone(), next_base_slc);
					println!("    Adding Axon Kind: '{:?}', len: {}, kind_base_slc: {}", axon_kind, list.len(), next_base_slc);
					assert!(list.len() == self.depth_axon_kind(&axon_kind) as usize);
					next_base_slc += list.len() as u8;
				},
				_ => ()
			}
		}

		print!("\n");
	}

	fn set_base_slc_ids(&mut self) {
		// (4) SET BASE ROW POSITION ON INDIVIDUAL NON-HORIZONTAL LAYERS 
		for (layer_name, layer) in self.layers.iter_mut() {
			let kind_base = layer.kind_base_slc_id();

			match layer.kind().clone() {
				LayerKind::Cellular(ref protocell) => {					
					layer.set_base_slc_id(self.cel_layer_kind_base_slc_ids[&protocell.cell_kind] 
						+ kind_base);
					print!("    <{}>: CellKind::{:?} ", layer_name, &protocell.cell_kind);
				},

				LayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&AxonKind::Horizontal => continue,

						_ => {
							layer.set_base_slc_id(self.axn_layer_kind_base_slc_ids[axon_kind] 
								+ kind_base);
							print!("    <{}>: AxonKind::{:?} ", layer_name, axon_kind);
						},
					}
				},
			}

			for i in layer.base_slc_id()..(layer.base_slc_id() + layer.depth_old_tmp()) {
				self.slc_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
			print!("\n");
		}

		// (5) SET BASE ROW POSITION ON INDIVIDUAL HORIZONTAL LAYERS 
		for (layer_name, layer) in self.layers.iter_mut() {
			let kind_base = layer.kind_base_slc_id();

			match layer.kind().clone() {
				LayerKind::Cellular(ref protocell) => continue,

				LayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&AxonKind::Horizontal => {
							layer.set_base_slc_id(self.axn_layer_kind_base_slc_ids[axon_kind] 
								+ kind_base);
							print!("    <{}>: AxonKind::{:?} ", layer_name, axon_kind);
						},

						_ => continue,
					}
				},
			}

			for i in layer.base_slc_id()..(layer.base_slc_id() + layer.depth_old_tmp()) {
				self.slc_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
			print!("\n");
		}

		// (6) MARVEL AT THE MOST CONVOLUTED FUNCTION EVER 
		print!("\n");
	}
	

	// ##### DEPTHS #####

	pub fn depth_total(&self) -> u8 {
		self.depth_axonal_spatial() + self.depth_cellular() + self.depth_axonal_horizontal()
	}

	pub fn depth_axonal_spatial(&self) -> u8 {
		let mut axon_slcs = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind() {
				&LayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&AxonKind::Spatial => axon_slcs += layer.depth_old_tmp(),
						_	=> (),
					}
				},
				&LayerKind::Cellular(_) => (),
			}
		}

		axon_slcs
	}

	pub fn depth_axonal_horizontal(&self) -> u8 {
		let mut axon_slcs = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind() {
				&LayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&AxonKind::Horizontal => axon_slcs += layer.depth_old_tmp(),
						_	=> (),
					}
				},
				&LayerKind::Cellular(_) => (),
			}
		}

		axon_slcs
	} 

	pub fn depth_cellular(&self) -> u8 {
		let mut cell_slcs = 0u8;

		for (layer_name, layer) in self.layers.iter() {
			match layer.kind() {
				&LayerKind::Axonal(_) => (),
				&LayerKind::Cellular(_) => cell_slcs += layer.depth_old_tmp(),
			}
		}

		cell_slcs
	}

	pub fn depth_cell_kind(&self, cell_kind: &CellKind) -> u8 {
		let mut count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.kind() {
				&LayerKind::Cellular(ref protocell) => {
					if &protocell.cell_kind == cell_kind {
						count += layer.depth_old_tmp();
					} else {
						//println!("{:?} didn't match {:?}", protocell.cell_kind, cell_kind);
					}
				},
				&LayerKind::Axonal(_) => (),
			}
		}

		let count2 = match self.cel_layer_kind_slc_lists.get(cell_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		//println!("CKRC: kind: {:?} -> count = {}, count2 = {}", &cell_kind, count, count2);

		assert!(count as usize == count2, "ProtolayerMap::depth_cell_kind(): mismatch");

		count
	}

	pub fn depth_axon_kind(&self, axon_kind: &AxonKind) -> u8 {
		let mut axonal_layer_count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.kind() {
				&LayerKind::Axonal(ref ak) => {
					if ak == axon_kind {
						axonal_layer_count += layer.depth_old_tmp();
					}
				},

				&LayerKind::Cellular(_) => {}
			}
		}

		let layer_kind_slc_lists_len = match self.axn_layer_kind_slc_lists.get(axon_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		assert!(axonal_layer_count as usize == layer_kind_slc_lists_len || !self.frozen, 
			"ProtolayerMap::depth_axon_kind(): mismatch");

		axonal_layer_count
	}	


	pub fn layers(&self) -> &HashMap<&'static str, Protolayer> {
		&self.layers
	}	

 	/// Returns all layers containing 'tags'.
 	pub fn layers_with_tags(&self, tags: LayerTags) -> Vec<&Protolayer> {
 		let mut layers: Vec<&Protolayer> = Vec::with_capacity(16);
 		 		
 		for (_, layer) in self.layers.iter().filter(|&(_, layer)| 
 				layer.tags().meshes(tags))
 		{
 			layers.push(&layer);
 		}

 		layers
 	}

 	pub fn slc_map(&self) -> BTreeMap<u8, &'static str> {
 		self.slc_map.clone()
	}

	pub fn hrz_demarc(&self) -> u8 {
		self.hrz_demarc
	}
}

impl<'b> Index<&'b&'static str> for ProtolayerMap
{
    type Output = Protolayer;

    fn index<'a>(&'a self, index: &'b&'static str) -> &'a Protolayer {
        self.layers.get(index).unwrap_or_else(|| panic!("ProtolayerMap::index(): invalid layer name: '{}'", index))
    }
}

impl<'b> IndexMut<&'b&'static str> for ProtolayerMap
{
    fn index_mut<'a>(&'a mut self, index: &'b&'static str) -> &'a mut Protolayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[ProtolayerMap::index(): invalid layer name: '{}'", index))
    }
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum RegionKind {
	Associational,
	Sensory,
	Motor,
	Thalamic,
}



pub struct ProtolayerMaps {
	map: HashMap<&'static str, ProtolayerMap>,
}

impl ProtolayerMaps {
	pub fn new() -> ProtolayerMaps {
		ProtolayerMaps {
			map: HashMap::new(),
		}
	}

	pub fn lm(mut self, pr: ProtolayerMap) -> ProtolayerMaps {
		self.add(pr);
		self
	}	

	pub fn add(&mut self, pr: ProtolayerMap) {
		self.map.insert(pr.name.clone(), pr);
	}
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
