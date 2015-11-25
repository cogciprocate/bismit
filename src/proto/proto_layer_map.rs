
use std::collections::{ HashMap, BTreeMap };
use std::ops::{ Index, IndexMut,  };
use std::hash::{ Hasher };

use map::{ self, LayerTags };
use super::{ ProtoareaMap, Protolayer, ProtoaxonKind, ProtolayerKind, ProtocellKind, DendriteKind, Axonal };


// PROTOLAYERMAP (PROTOREGION) {} <<<<< TODO: SPLIT UP, REDESIGN, AND REFACTOR >>>>>
// - [incomplete] SEPERATE CONCERNS and consolidate similar data structures - GETTING TOO UNRULY
// - redesign using a trait that can handle ProtocellKind and ProtoaxonKind both
//    - Also could merge the two and have one or the other dominant
// - [incomplete] (cel, axn)_layer_kind_slc_lists needs to be redone asap
// - should be instances of some new type which manages their lists 

#[derive(Clone)]
pub struct ProtolayerMap {
	pub name: &'static str,
	pub kind: RegionKind,
	layers: HashMap<&'static str, Protolayer>,
	cel_layer_kind_slc_lists: HashMap<ProtocellKind, Vec<&'static str>>,
	cel_layer_kind_base_slc_ids: HashMap<ProtocellKind, u8>,
	axn_layer_kind_slc_lists: HashMap<ProtoaxonKind, Vec<&'static str>>,
	axn_layer_kind_base_slc_ids: HashMap<ProtoaxonKind, u8>,
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

	pub fn axn_layer(mut self, layer_name: &'static str, tags: LayerTags, axon_kind: ProtoaxonKind,
			) -> ProtolayerMap 
	{
		let next_kind_base_slc_id = self.depth_axon_kind(&axon_kind);
		let layer_depth = if tags.contains(map::OUTPUT) { 1 } else { 0 };
		
		self.add(Protolayer::new(layer_name, Axonal(axon_kind), layer_depth, 0,
			next_kind_base_slc_id, tags));
		self
	}

	pub fn layer(mut self, layer_name: &'static str, layer_depth: u8, tags: LayerTags, 
			kind: ProtolayerKind) -> ProtolayerMap 
	{
		let next_kind_base_slc_id = match kind {
			ProtolayerKind::Cellular(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			ProtolayerKind::Axonal(ref axon_kind) => self.depth_axon_kind(&axon_kind),
		};
		
		self.add(Protolayer::new(layer_name, kind, layer_depth, 0, next_kind_base_slc_id, tags));
		self
	}

	// PROTOREGION::ADD()
	// [FIXME]: NEED TO CHECK FOR DUPLICATE LAYERS!	
	pub fn add(&mut self, layer: Protolayer) {
		if self.frozen {
			panic!("ProtolayerMap::add(): Cannot add new layers after region is frozen.");
		}

		self.layers.insert(layer.name, layer);
	}

	fn gen_slc_lists(&mut self) {
		for (layer_name, layer) in self.layers.iter_mut() {
			match layer.kind {
				ProtolayerKind::Cellular(ref cell) => {
					let cell_kind = cell.cell_kind.clone();

					let ck_vec_opt: Option<&mut Vec<&'static str>> = if self.cel_layer_kind_slc_lists.contains_key(&cell_kind) {
						self.cel_layer_kind_slc_lists.get_mut(&cell_kind)
					} else {
						self.cel_layer_kind_slc_lists.insert(cell_kind.clone(), Vec::new());
						self.cel_layer_kind_slc_lists.get_mut(&cell_kind)
					};

					match ck_vec_opt {

						Some(vec) => {
							
							layer.kind_base_slc_id = vec.len() as u8;
							//layer.kind_base_slc_id = std::num::cast(vec.len()).expect("ProtolayerMap::add()");
							//println!("{:?} base_slc_id: {}", cell_kind, layer.kind_base_slc_id);

							for i in 0..layer.depth {							 
								vec.push(layer.name);
								//println!("Adding {} to list of {:?}", layer.name, cell_kind);
							}

							//println!("{:?} list len: {}", cell_kind, vec.len());
						},
						None => (),
					}
				},

				ProtolayerKind::Axonal(ref axon_kind) => {
					let ck_vec_opt: Option<&mut Vec<&'static str>> = if self.axn_layer_kind_slc_lists.contains_key(&axon_kind) {
						self.axn_layer_kind_slc_lists.get_mut(&axon_kind)
					} else {
						self.axn_layer_kind_slc_lists.insert(axon_kind.clone(), Vec::new());
						self.axn_layer_kind_slc_lists.get_mut(&axon_kind)
					};

					match ck_vec_opt {

						Some(vec) => {
							
							layer.kind_base_slc_id = vec.len() as u8;

							for i in 0..layer.depth {							 
								vec.push(layer.name);
							}

						},
						None => (),
					}
				},
			}
		}
	}

	// SET_LAYERS_DEPTH(): ASSUMES PROPER FLAG UNIQUENESS CONSTRAINS ALREADY APPLIED
	pub fn set_layer_depth(&mut self, tags: LayerTags, depth: u8) {
		if self.frozen { 
			panic!("region::ProtolayerMap::set_layer_depth(): \
				Cannot set layer depth after region has been frozen."); 
		} 

		for (layer_name, mut layer) in self.layers.iter_mut() {
			if (layer.tags & tags) == tags {
				//println!(" ##### SETTING LAYER DEPTH FOR LAYER: '{}' TO: {} #####", layer_name, depth);
				layer.depth = depth;
			}
		}
	}

	pub fn set_layer_depths(&mut self, pamap: &ProtoareaMap) {
		// FEEDFORWARD INPUT COMES FROM EFF AREAS, FEEDBACK INPUT COMES FROM AFF AREAS
		self.set_layer_depth(map::FF_IN, pamap.eff_areas.len() as u8);
		self.set_layer_depth(map::FB_IN, pamap.aff_areas.len() as u8);
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
			self.set_layer_depths(pamap);
			self.frozen = true;
		}		

		self.gen_slc_lists();

		// (0) START COUNTER FOR ABSOLUTE BASE ROWS 
		let mut next_base_slc = 0u8;

		// (1) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL SPATIAL LAYER KINDS 	
		for (axon_kind, list) in &self.axn_layer_kind_slc_lists {
			match axon_kind {
				&ProtoaxonKind::Spatial => {
					self.axn_layer_kind_base_slc_ids.insert(axon_kind.clone(), next_base_slc);
					println!("    Adding Axon Kind: '{:?}', len: {}, kind_base_slc: {}", axon_kind, list.len(), next_base_slc);
					assert!(list.len() == self.depth_axon_kind(&axon_kind) as usize);
					next_base_slc += list.len() as u8;
				},
				_ => ()
			}
		}

		// (2) ADD ABSOLUTE BASE_ROW_IDS FOR ALL CELLULAR LAYER KINDS 
		for (cell_kind, list) in &self.cel_layer_kind_slc_lists {
			self.cel_layer_kind_base_slc_ids.insert(cell_kind.clone(), next_base_slc);
			println!("    Adding Cell Kind: '{:?}', len: {}, kind_base_slc: {}", cell_kind, list.len(), next_base_slc);
			assert!(list.len() == self.depth_cell_kind(&cell_kind) as usize);
			next_base_slc += list.len() as u8;
			//next_base_slc += std::num::cast::<usize, u8>(list.len()).expect("cortical_region::ProtolayerMap::freeze()");
		}

		// (2b) SAVE DEMARCATION BETWEEN VERTICAL (SPATIAL) AND HORIZONTAL ROWS 
		self.hrz_demarc = next_base_slc;

		// (3) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL HORIZONTAL LAYER KINDS 	
		for (axon_kind, list) in &self.axn_layer_kind_slc_lists {
			match axon_kind {
				&ProtoaxonKind::Horizontal => {
					self.axn_layer_kind_base_slc_ids.insert(axon_kind.clone(), next_base_slc);
					println!("    Adding Axon Kind: '{:?}', len: {}, kind_base_slc: {}", axon_kind, list.len(), next_base_slc);
					assert!(list.len() == self.depth_axon_kind(&axon_kind) as usize);
					next_base_slc += list.len() as u8;
				},
				_ => ()
			}
		}

		print!("\n");

		// (4) SET BASE ROW POSITION ON INDIVIDUAL NON-HORIZONTAL LAYERS 
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {

				&ProtolayerKind::Cellular(ref protocell) => {
					layer.base_slc_id = self.cel_layer_kind_base_slc_ids[&protocell.cell_kind] + layer.kind_base_slc_id;
					print!("    <{}>: ProtocellKind::{:?} ", layer_name, &protocell.cell_kind);
				},

				&ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Horizontal => continue,

						_ => {
							layer.base_slc_id = self.axn_layer_kind_base_slc_ids[axon_kind] + layer.kind_base_slc_id;
							print!("    <{}>: ProtoaxonKind::{:?} ", layer_name, axon_kind);
						},
					}
				},
			}

			for i in layer.base_slc_id..(layer.base_slc_id + layer.depth()) {
				self.slc_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
			print!("\n");
		}

		// (5) SET BASE ROW POSITION ON INDIVIDUAL HORIZONTAL LAYERS 
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {
				&ProtolayerKind::Cellular(ref protocell) => continue,

				&ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Horizontal => {
							layer.base_slc_id = self.axn_layer_kind_base_slc_ids[axon_kind] + layer.kind_base_slc_id;
							print!("    <{}>: ProtoaxonKind::{:?} ", layer_name, axon_kind);
						},

						_ => continue,
					}
				},
			}

			for i in layer.base_slc_id..(layer.base_slc_id + layer.depth()) {
				self.slc_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
			print!("\n");
		}

		// (6) MARVEL AT THE MOST CONVOLUTED FUNCTION EVER 
		print!("\n");
	}


	// pub fn base_slc(&self, layer_name: &'static str) -> u8 {
	// 	let ref layer = self.layers[layer_name];
	// 	layer.base_slc_id
	// }

	// pub fn base_slc_cell_kind(&self, cell_kind: &ProtocellKind) -> u8 {
	// 	match self.cel_layer_kind_base_slc_ids.get(cell_kind) {
	// 		Some(base_slc) 	=> base_slc.clone(),
	// 		None 			=> panic!("ProtolayerMap::base_slc_cell_king(): Base slc for type not found"),
	// 	}
	// }
	

	// ##### DEPTHS #####

	pub fn depth_total(&self) -> u8 {
		self.depth_axonal_spatial() + self.depth_cellular() + self.depth_axonal_horizontal()
	}

	pub fn depth_axonal_spatial(&self) -> u8 {
		let mut axon_slcs = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Spatial => axon_slcs += layer.depth,
						_	=> (),
					}
				},
				ProtolayerKind::Cellular(_) => (),
			}
		}

		axon_slcs
	}

	pub fn depth_axonal_horizontal(&self) -> u8 {
		let mut axon_slcs = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Horizontal => axon_slcs += layer.depth,
						_	=> (),
					}
				},
				ProtolayerKind::Cellular(_) => (),
			}
		}

		axon_slcs
	} 

	pub fn depth_cellular(&self) -> u8 {
		let mut cell_slcs = 0u8;

		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Axonal(_) => (),
				ProtolayerKind::Cellular(_) => cell_slcs += layer.depth,
			}
		}

		cell_slcs
	}

	pub fn depth_cell_kind(&self, cell_kind: &ProtocellKind) -> u8 {
		let mut count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Cellular(ref protocell) => {
					if &protocell.cell_kind == cell_kind {
						count += layer.depth;
					} else {
						//println!("{:?} didn't match {:?}", protocell.cell_kind, cell_kind);
					}
				},
				ProtolayerKind::Axonal(_) => (),
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

	pub fn depth_axon_kind(&self, axon_kind: &ProtoaxonKind) -> u8 {
		let mut axonal_layer_count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Axonal(ref ak) => {
					if ak == axon_kind {
						axonal_layer_count += layer.depth;
					}
				},

				ProtolayerKind::Cellular(_) => {}
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


	// ##### LAYERS #####


	pub fn layers(&self) -> &HashMap<&'static str, Protolayer> {
		&self.layers
	}	

	pub fn slc_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		if !self.frozen { // REPLACE WITH ASSERT (evaluate release build implications first)
			panic!("ProtolayerMap must be frozen with .freeze() before slc_ids can be called.");
		}

		let mut slc_ids = Vec::new();

		for layer_name in layer_names.iter() {
			let l = &self.layers[layer_name];
				for i in l.base_slc_id..(l.base_slc_id + l.depth) {
					slc_ids.push(i);
				}
		}

		slc_ids
	}

	// SRC_SLC_IDS(): Get a merged list of source slice ids for all source layers.
	// [FIXME] TODO: Merge this with dst_* below.
	pub fn src_slc_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_lyr_names = self.layers[&layer_name].src_lyr_names(den_type);
		
		self.slc_ids(src_lyr_names)
 	}


 	// DST_SRC_SLC_IDS(): Get a grouped list of source slice ids for each distal dendritic tuft in a layer.
 	pub fn dst_src_slc_ids(&self, layer_name: &'static str) -> Vec<Vec<u8>> {
 		let src_tufts = self.dst_src_lyrs_by_tuft(layer_name);

 		let mut dst_src_slc_ids = Vec::with_capacity(src_tufts.len());

 		for tuft in src_tufts {
 			dst_src_slc_ids.push(self.slc_ids(tuft));
		}

		dst_src_slc_ids
	}

	// DST_SRC_LYRS_BY_TUFT(): Get a grouped list of source layer names for each distal dendritic tuft in a layer.
	pub fn dst_src_lyrs_by_tuft(&self, layer_name: &'static str) -> Vec<Vec<&'static str>> {
		// [FIXME][DONE?] TODO: RETURN ONLY VALID TUFTS!
		let mut potential_tufts = self.layers[layer_name].dst_src_lyrs_by_tuft();
		let mut valid_tufts: Vec<Vec<&'static str>> = Vec::with_capacity(potential_tufts.len());

		for mut potential_tuft_src_lyrs in potential_tufts.drain(..) {
			let mut valid_src_lyrs: Vec<&'static str> = Vec::with_capacity(potential_tuft_src_lyrs.len());

			for lyr_name in potential_tuft_src_lyrs.drain(..) {
				if self.layers[lyr_name].depth > 0 {
					valid_src_lyrs.push(lyr_name);
				}
			}

			if valid_src_lyrs.len() > 0 {
				valid_tufts.push(valid_src_lyrs);
			}
		}

		valid_tufts		
	}

 	pub fn spt_asc_layer(&self) -> Option<Protolayer> {
 		let mut input_layer: Option<Protolayer> = None;
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.tags & map::SPATIAL_ASSOCIATIVE) == map::SPATIAL_ASSOCIATIVE {
 				input_layer = Some(layer.clone());
 			}
 		}

		input_layer
 	}

 	pub fn aff_out_slcs(&self) -> Vec<u8> {
 		let mut output_slcs: Vec<u8> = Vec::with_capacity(4);
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.tags & map::FF_OUT) == map::FF_OUT {
 				let v = self.slc_ids(vec![layer.name]);
 				output_slcs.push_all(&v);
 			}
 		}

		output_slcs		
 	}

 	// TODO: DEPRICATE IN FAVOR OF LAYERS_WITH_FLAG()
 	pub fn layer_with_flag(&self, flag: LayerTags) -> Option<&Protolayer> {
 		//let mut input_layers: Vec<&Protolayer>
 		 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.tags & flag) == flag {
 				return Some(&layer);
 			}
 		}
 		return None;
 	}


 	/// Returns all layers containing 'tags'.
 	pub fn layers_with_tags(&self, tags: LayerTags) -> Vec<&Protolayer> {
 		let mut layers: Vec<&Protolayer> = Vec::with_capacity(16);
 		 		
 		for (_, layer) in self.layers.iter() {
 			// if (layer.tags & tags) == tags {
 			if layer.tags.contains(tags) {
 				layers.push(&layer);
 			}
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
