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
		//self.hash_map.insert(cr.kind.clone(), cr);
		self.add(pr);
		self
	}

	/*pub fn depth(&self, cr_type: ProtoregionKind) -> (u8, u8) {
		let mut depth_axonal_slices = 0u8;		//	Integererregional
		let mut depth_cellular_slices = 0u8;			//	Integererlaminar

		for (region_type, region) in self.hash_map.iter() {					// CHANGE TO FILTER
			if *region_type == cr_type {							//
				let (noncell, cell) = region.depth_axonal_and_cellular();
				depth_axonal_slices += noncell;
				depth_cellular_slices += cell;
				//println!("*** noncell: {}, cell: {} ***", noncell, cell);
			}
		}

		(depth_axonal_slices, depth_cellular_slices)
	}*/

	/*pub fn depth_total(&self, cr_type: ProtoregionKind) -> u8 {
		let (hacr, hcr) = self.depth(cr_type);
		hacr + hcr
	}*/

	pub fn add(&mut self, pr: Protoregion) {
		self.hash_map.insert(pr.kind.clone(), pr);
	}
}

impl<'b> Index<&'b ProtoregionKind> for Protoregions
{
    type Output = Protoregion;

    fn index<'a>(&'a self, index: &'b ProtoregionKind) -> &'a Protoregion {
        self.hash_map.get(index).expect("Invalid region name.")
    }
}

impl<'b> IndexMut<&'b ProtoregionKind> for Protoregions
{
    fn index_mut<'a>(&'a mut self, index: &'b ProtoregionKind) -> &'a mut Protoregion {
        self.hash_map.get_mut(index).expect("Invalid region name.")
    }
}



/* CORTICALREGION {}
	- [incomplete] THIS NEEDS TO BE STORED IN A DATABASE OR SOMETHING - GETTING TOO UNRULY
		- Or... redesign using a trait that can handle ProtocellKind and ProtoaxonKind both
			- Also could merge the two and have one or the other dominant
	- [incomplete] (cel, axn)_layer_kind_slice_lists needs to be redone asap
*/
#[derive(Clone)]
pub struct Protoregion {
	layers: HashMap<&'static str, Protolayer>,
	cel_layer_kind_slice_lists: HashMap<ProtocellKind, Vec<&'static str>>,
	cel_layer_kind_base_slice_ids: HashMap<ProtocellKind, u8>,
	axn_layer_kind_slice_lists: HashMap<ProtoaxonKind, Vec<&'static str>>,
	axn_layer_kind_base_slice_ids: HashMap<ProtoaxonKind, u8>,
	slice_map: BTreeMap<u8, &'static str>,
	pub kind: ProtoregionKind,
	frozen: bool,
	hrz_demarc: u8,
}

impl Protoregion {
	pub fn new (kind: ProtoregionKind)  -> Protoregion {
		/*let mut next_slice_id = HashMap::new();
		next_slice_id.insert(ProtocellKind::Pyramidal, 0);
		next_slice_id.insert(ProtocellKind::InhibitoryInterneuronNetwork, 0);
		next_slice_id.insert(ProtocellKind::SpinyStellate, 0);*/
	
		Protoregion { 
			layers: HashMap::new(),
			cel_layer_kind_slice_lists: HashMap::new(),
			cel_layer_kind_base_slice_ids: HashMap::new(),
			axn_layer_kind_slice_lists: HashMap::new(),
			axn_layer_kind_base_slice_ids: HashMap::new(),
			kind: kind,
			frozen: false,
			slice_map: BTreeMap::new(),
			hrz_demarc: 0,
		}
	}

	pub fn layer(
					mut self, 
					layer_name: &'static str,
					layer_depth: u8,
					flags: ProtolayerFlags,
					kind: ProtolayerKind,
	) -> Protoregion {

		let next_kind_base_slice_pos = match kind {
			ProtolayerKind::Cellular(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			ProtolayerKind::Axonal(ref axon_kind) => self.depth_axon_kind(&axon_kind),
		};
		
		let cl = Protolayer {
			name : layer_name,
			kind: kind,
			base_slice_pos: 0, 
			kind_base_slice_pos: next_kind_base_slice_pos,
			depth: layer_depth,
			flags: flags,
		};

		self.add(cl);
		self
	}

	/* PROTOREGION::ADD()
		- [incomplete] NEED TO CHECK FOR DUPLICATE LAYERS!
	*/
	pub fn add(&mut self, mut layer: Protolayer) {

		/*let ck_tmp = match layer.kind {
			Some(ref kind) 	=> kind.cell_kind.clone(),
			None 			=> ProtocellKind::Nada,
		};*/
		if self.frozen {
			panic!("protoregions::Protoregion::add(): Cannot add new layers after region is frozen.");
		}
		
		match layer.kind {

			ProtolayerKind::Cellular(ref cell) => {
				let cell_kind = cell.cell_kind.clone();

				let ck_vec_opt: Option<&mut Vec<&'static str>> = if self.cel_layer_kind_slice_lists.contains_key(&cell_kind) {
					self.cel_layer_kind_slice_lists.get_mut(&cell_kind)
				} else {
					self.cel_layer_kind_slice_lists.insert(cell_kind.clone(), Vec::new());
					self.cel_layer_kind_slice_lists.get_mut(&cell_kind)
				};

				match ck_vec_opt {

					Some(vec) => {
						
						layer.kind_base_slice_pos = vec.len() as u8;
						//layer.kind_base_slice_pos = std::num::cast(vec.len()).expect("protoregions::Protoregion::add()");
						//print!("\n{:?} base_slice_pos: {}", cell_kind, layer.kind_base_slice_pos);

						for i in 0..layer.depth {							 
							vec.push(layer.name);
							//print!("\nAdding {} to list of {:?}", layer.name, cell_kind);
						}

						//print!("\n{:?} list len: {}", cell_kind, vec.len());
					},
					None => (),
				}
			},

			ProtolayerKind::Axonal(ref axon_kind) => {
				let ck_vec_opt: Option<&mut Vec<&'static str>> = if self.axn_layer_kind_slice_lists.contains_key(&axon_kind) {
					self.axn_layer_kind_slice_lists.get_mut(&axon_kind)
				} else {
					self.axn_layer_kind_slice_lists.insert(axon_kind.clone(), Vec::new());
					self.axn_layer_kind_slice_lists.get_mut(&axon_kind)
				};

				match ck_vec_opt {

					Some(vec) => {
						
						layer.kind_base_slice_pos = vec.len() as u8;

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

		/*match self.cel_layer_kind_slice_lists.get(&ck_tmp) {
			Some(vec) 	=> print!("\nFound Vector with len: {}",vec.len()),
			None 		=> print!("\nVector NOT FOUND"),
		};*/
	}

	pub fn base_slice(&self, layer_name: &'static str) -> u8 {
		let ref layer = self.layers[layer_name];
		layer.base_slice_pos
	}

	pub fn base_slice_cell_kind(&self, cell_kind: &ProtocellKind) -> u8 {
		match self.cel_layer_kind_base_slice_ids.get(cell_kind) {
			Some(base_slice) 	=> base_slice.clone(),
			None 			=> panic!("Protoregion::base_slice_cell_king(): Base slice for type not found"),
		}
	}

	/*pub fn depth_axonal_and_cellular(&self) -> (u8, u8) {
		let mut axon_slices = 0u8;
		let mut cell_slices = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Axonal(_) => axon_slices += layer.depth,
				ProtolayerKind::Cellular(_) => cell_slices += layer.depth,
			}
		}

		(axon_slices, cell_slices)
	}*/

	pub fn depth_total(&self) -> u8 {
		self.depth_axonal_spatial() + self.depth_cellular() + self.depth_axonal_horizontal()
	}

	pub fn depth_axonal_spatial(&self) -> u8 {
		let mut axon_slices = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Spatial => axon_slices += layer.depth,
						_	=> (),
					}
				},
				ProtolayerKind::Cellular(_) => (),
			}
		}

		axon_slices
	}

	pub fn depth_axonal_horizontal(&self) -> u8 {
		let mut axon_slices = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Horizontal => axon_slices += layer.depth,
						_	=> (),
					}
				},
				ProtolayerKind::Cellular(_) => (),
			}
		}

		axon_slices
	} 

	pub fn depth_cellular(&self) -> u8 {
		let mut cell_slices = 0u8;

		for (layer_name, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Axonal(_) => (),
				ProtolayerKind::Cellular(_) => cell_slices += layer.depth,
			}
		}

		cell_slices
	}

	pub fn depth_cell_kind(&self, cell_kind: &ProtocellKind) -> u8 {
		let mut count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.kind {
				ProtolayerKind::Cellular(ref protocell) => {
					if &protocell.cell_kind == cell_kind {
						count += layer.depth;
					} else {
						//print!("\n{:?} didn't match {:?}", protocell.cell_kind, cell_kind);
					}
				},
				ProtolayerKind::Axonal(_) => (),
			}
		}

		let mut count2 = match self.cel_layer_kind_slice_lists.get(cell_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		//print!("\nCKRC: kind: {:?} -> count = {}, count2 = {}", &cell_kind, count, count2);

		assert!(count as usize == count2, "protoregions::Protoregion::depth_cell_kind(): mismatch");

		count
	}

	pub fn depth_axon_kind(&self, axon_kind: &ProtoaxonKind) -> u8 {
		let mut count = 0u8;

		for (_, layer) in self.layers.iter() {
			match layer.kind {

				ProtolayerKind::Axonal(ref ak) => {
					if ak == axon_kind {
						count += layer.depth;
					}
				},

				ProtolayerKind::Cellular(_) => {}
			}
		}

		let mut count2 = match self.axn_layer_kind_slice_lists.get(axon_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		assert!(count as usize == count2, "protoregions::Protoregion::depth_axon_kind(): mismatch");

		count
	}

	/*pub fn depth_slice_total(&self) -> u8 {
		let mut total_depth = 0u8;

		for (_, layer) in self.layers.iter() {
			total_depth += layer.depth;
		}

		total_depth
	}*/
 

 	/* PROTOREGION::FREEZE():
 		- What a mess...
		- Need to revamp how axon_types and cell_types are stored before we can do much with it
			- cel_layer_kind_slice_lists being a vector needs to change asap
 	*/
	pub fn freeze(mut self) -> Protoregion {
		if self.frozen {
			return self;
		} else {
			self.frozen = true;
		}


		/* (0) START COUNTER FOR ABSOLUTE BASE ROWS */
		let mut next_base_slice = 0u8;

		/* (1) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL SPATIAL LAYER KINDS */	
		for (axon_kind, list) in &self.axn_layer_kind_slice_lists {
			match axon_kind {
				&ProtoaxonKind::Spatial => {
					self.axn_layer_kind_base_slice_ids.insert(axon_kind.clone(), next_base_slice);
					print!("\n    Adding Axon Kind: '{:?}', len: {}, kind_base_slice: {}", axon_kind, list.len(), next_base_slice);
					assert!(list.len() == self.depth_axon_kind(&axon_kind) as usize);
					next_base_slice += list.len() as u8;
				},
				_ => ()
			}
		}

		/* (2) ADD ABSOLUTE BASE_ROW_IDS FOR ALL CELLULAR LAYER KINDS */
		for (cell_kind, list) in &self.cel_layer_kind_slice_lists {
			self.cel_layer_kind_base_slice_ids.insert(cell_kind.clone(), next_base_slice);
			print!("\n    Adding Cell Kind: '{:?}', len: {}, kind_base_slice: {}", cell_kind, list.len(), next_base_slice);
			assert!(list.len() == self.depth_cell_kind(&cell_kind) as usize);
			next_base_slice += list.len() as u8;
			//next_base_slice += std::num::cast::<usize, u8>(list.len()).expect("cortical_region::Protoregion::freeze()");
		}

		/* (2b) SAVE DEMARCATION BETWEEN VERTICAL (SPATIAL) AND HORIZONTAL ROWS */
		self.hrz_demarc = next_base_slice;

		/* (3) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL HORIZONTAL LAYER KINDS */	
		for (axon_kind, list) in &self.axn_layer_kind_slice_lists {
			match axon_kind {
				&ProtoaxonKind::Horizontal => {
					self.axn_layer_kind_base_slice_ids.insert(axon_kind.clone(), next_base_slice);
					print!("\n    Adding Axon Kind: '{:?}', len: {}, kind_base_slice: {}", axon_kind, list.len(), next_base_slice);
					assert!(list.len() == self.depth_axon_kind(&axon_kind) as usize);
					next_base_slice += list.len() as u8;
				},
				_ => ()
			}
		}

		print!("\n");

		/* (4) SET BASE ROW POSITION ON INDIVIDUAL NON-HORIZONTAL LAYERS */
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {

				&ProtolayerKind::Cellular(ref protocell) => {
					layer.base_slice_pos = self.cel_layer_kind_base_slice_ids[&protocell.cell_kind] + layer.kind_base_slice_pos;
					print!("\n    <{}>: ProtocellKind::{:?} ", layer_name, &protocell.cell_kind);
				},

				&ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Horizontal => continue,

						_ => {
							layer.base_slice_pos = self.axn_layer_kind_base_slice_ids[axon_kind] + layer.kind_base_slice_pos;
							print!("\n    <{}>: ProtoaxonKind::{:?} ", layer_name, axon_kind);
						},
					}
				},
			}

			for i in layer.base_slice_pos..(layer.base_slice_pos + layer.depth()) {
				self.slice_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
		}

		/* (5) SET BASE ROW POSITION ON INDIVIDUAL HORIZONTAL LAYERS */
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {
				&ProtolayerKind::Cellular(ref protocell) => continue,

				&ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Horizontal => {
							layer.base_slice_pos = self.axn_layer_kind_base_slice_ids[axon_kind] + layer.kind_base_slice_pos;
							print!("\n    <{}>: ProtoaxonKind::{:?} ", layer_name, axon_kind);
						},

						_ => continue,
					}
				},
			}

			for i in layer.base_slice_pos..(layer.base_slice_pos + layer.depth()) {
				self.slice_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
		}

		/* (6) MARVEL AT THE MOST CONVOLUTED FUNCTION EVER */
		print!("\n");
		self
	}


	pub fn layers(&self) -> &HashMap<&'static str, Protolayer> {
		&self.layers
	}

	pub fn get_layer(&self, layer_name: &'static str) -> Option<&Protolayer> {
		self.layers.get(layer_name)
	}

	pub fn slices_by_layer_name(&self, cell_kind: &ProtocellKind) -> Option<&Vec<&'static str>> {
		self.cel_layer_kind_slice_lists.get(cell_kind)
	}

	pub fn slice_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		if !self.frozen { // REPLACE WITH ASSERT (evaluate release build implications first)
			panic!("Protoregion must be frozen with .freeze() before slice_ids can be called.");
		}

		let mut slice_ids = Vec::new();

		for layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in l.base_slice_pos..(l.base_slice_pos + l.depth) {
					slice_ids.push(i);
				}
		}

		slice_ids
	}

	pub fn src_slice_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_layer_names = self[&layer_name].src_layer_names(den_type);
		
		self.slice_ids(src_layer_names)
 	}

 	pub fn spt_asc_layer(&self) -> Option<Protolayer> {
 		let mut input_layer: Option<Protolayer> = None;
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.flags & layer::SPATIAL_ASSOCIATIVE) == layer::SPATIAL_ASSOCIATIVE {
 				input_layer = Some(layer.clone());
 			}
 		}

		input_layer		
 	}

 	pub fn aff_out_slices(&self) -> Vec<u8> {
 		let mut output_slices: Vec<u8> = Vec::with_capacity(4);
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.flags & layer::AFFERENT_OUTPUT) == layer::AFFERENT_OUTPUT {
 				let v = self.slice_ids(vec![layer.name]);
 				output_slices.push_all(&v);
 			}
 		}

		output_slices		
 	}

 	pub fn layer_with_flag(&self, flag: ProtolayerFlags) -> Option<Protolayer> {
 		let mut input_layer: Option<Protolayer> = None;
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.flags & flag) == flag {
 				input_layer = Some(layer.clone());
 			}
 		}

		input_layer		
 	}

 	pub fn slice_map(&self) -> BTreeMap<u8, &'static str> {
 		self.slice_map.clone()
	}

 	pub fn layer_name(&self, slice_id: u8) -> &'static str {
 		match self.slice_map.get(&slice_id) {
 			Some(ln) 	=> ln,
 			None 		=> "[INVALID LAYER]",
		}

	}

	pub fn hrz_demarc(&self) -> u8 {
		self.hrz_demarc
	}
}

impl<'b> Index<&'b&'static str> for Protoregion
{
    type Output = Protolayer;

    fn index<'a>(&'a self, index: &'b&'static str) -> &'a Protolayer {
        self.layers.get(index).unwrap_or_else(|| panic!("protoregions::Protoregion::index(): invalid layer name: '{}'", index))
    }
}

impl<'b> IndexMut<&'b&'static str> for Protoregion
{
    fn index_mut<'a>(&'a mut self, index: &'b&'static str) -> &'a mut Protolayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[protoregions::Protoregion::index(): invalid layer name: '{}'", index))
    }
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ProtoregionKind {
	Associational,
	Sensory,
	Motor,
}

impl Copy for ProtoregionKind {}


/*pub struct Protoregion {
	pub layers: HashMap<&'static str, Protolayer>,
	pub kind: ProtoregionKind,
}

impl Protoregion {
	pub fn new (kind: ProtoregionKind)  -> Protoregion {
		let mut next_slice_id = HashMap::new();
		next_slice_id.insert(ProtocellKind::Pyramidal, 0);
		next_slice_id.insert(ProtocellKind::InhibitoryInterneuronNetwork, 0);
		next_slice_id.insert(ProtocellKind::SpinyStellate, 0);
	
		Protoregion { 
			layers: HashMap::new(),
			kind: kind,
		}
	}

	pub fn new_layer(
					&mut self, 
					layer_name: &'static str,
					layer_depth: u8,
					flags: ProtolayerFlags,
					cell: Option<Protocell>,
	) {
		let (noncell_slices, cell_slices) = self.depth();

		let next_base_slice_pos = self.total_depth();

		let next_kind_base_slice_pos = match cell {
			Some(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			None => noncell_slices,
		};

		println!("Protolayer: {}, layer_depth: {}, base_slice_pos: {}, kind_base_slice_pos: {}", layer_name, layer_depth, next_base_slice_pos, next_kind_base_slice_pos);
		
		let cl = Protolayer {
			name : layer_name,
			cell: cell,
			base_slice_pos: next_base_slice_pos, 
			kind_base_slice_pos: next_kind_base_slice_pos,
			depth: layer_depth,
			flags: flags,
		};

		self.add(cl);
	}

	pub fn add(&mut self, layer: Protolayer) {
		self.layers.insert(layer.name, layer);
	}

	pub fn width() -> u8 {
		panic!("not implemented");
	}

	pub fn depth_cell_kind(&self, cell_kind: &ProtocellKind) -> u8 {
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
		let mut noncell_slices = 0u8;
		let mut cell_slices = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.cell {
				None => noncell_slices += layer.depth,
				Some(_) => cell_slices += layer.depth,
			}
		}
		(noncell_slices, cell_slices)
	}

	pub fn slice_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		let mut slice_ids = Vec::new();
		for &layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in range(l.base_slice_pos, l.base_slice_pos + l.depth) {
					slice_ids.push(i);
				}
		}
		slice_ids
	}

	pub fn src_slice_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_layer_names = self[layer_name].src_layer_names(den_type);
		
		self.slice_ids(src_layer_names)
 	}

 	pub fn col_input_slice(&self) -> u8 {
 		for (layer_name, layer) in self.layers.iter() {

 		}
 		5
 	}

}

impl Index<&'static str> for Protoregion
{
    type Output = Protolayer;

    fn index<'a>(&'a self, index: &&'static str) -> &'a Protolayer {
        self.layers.get(index).unwrap_or_else(|| panic!("[protoregions::Protoregion::index(): invalid layer name: \"{}\"]", index))
    }
}

impl IndexMut<&'static str> for Protoregion
{
    type Output = Protolayer;

    fn index_mut<'a>(&'a mut self, index: &&'static str) -> &'a mut Protolayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[protoregions::Protoregion::index(): invalid layer name: \"{}\"]", index))
    }
}*/



 	/*pub fn kind_slice_ids(&self, layer_name: &'static str) -> Vec<u8> {

		let l = &self[layer_name];
		let mut slice_ids = Vec::new();
			for i in range(l.base_slice_pos, l.base_slice_pos + l.depth) {
				slice_ids.push(i);
			}
		return slice_ids;
	}

	pub fn kind_src_slice_ids(&self, layer_name: &'static str) -> Vec<u8> {
		let src_layer_names = self[layer_name].src_layer_names();
		
		let mut src_slice_ids = Vec::new();

		for &src_slice_name in src_layer_names.iter() {
			src_slice_ids.push_all(self.kind_slice_ids(src_slice_name).as_slice());
		}

		//println!("Protoregion::layer_srcs_slice_ids(): (name:sources:idxs) [{}]:{:?}:{:?}", layer_name, src_layer_names, src_slice_ids);
		
		src_slice_ids
 	}*/


/* AxonScope 
	
	Integererlaminar(
		Distal Dendrite Input Protolayers,
		Proximal Dendrite Input Protolayers,
		Cell Type
	)

*/


/*fn increment_slice_index(mut cri: u8, by: u8) -> u8 {
	cri += by;
	cri
}*/
