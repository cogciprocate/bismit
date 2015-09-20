use std;
use std::collections::{ self, HashMap, BTreeMap };
use std::collections::hash_state::{ HashState };
use num;
use std::ops::{ Index, IndexMut, Range };
use std::hash::{ self, Hash, SipHasher, Hasher };

//use proto::cell::{  };
//use super::layer as layer;
use super::{ Protoarea };
use super::layer::{ self, Protolayer, ProtolayerFlags, ProtoaxonKind, ProtolayerKind };
	//use super::layer::ProtolayerKind::{ self, Cellular, Axonal };
use super::cell::{ ProtocellKind, Protocell, DendriteKind };


/* PROTOREGION {}
	- [incomplete] THIS NEEDS TO BE STORED IN A DATABASE OR SOMETHING - GETTING TOO UNRULY
		- Or... redesign using a trait that can handle ProtocellKind and ProtoaxonKind both
			- Also could merge the two and have one or the other dominant
	- [incomplete] (cel, axn)_layer_kind_slc_lists needs to be redone asap
		- should be instances of some new type which manages their lists
*/
#[derive(Clone)]
pub struct Protoregion {
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

impl Protoregion {
	pub fn new (region_name: &'static str, kind: RegionKind)  -> Protoregion {	
		Protoregion { 
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

	pub fn layer(
					mut self, 
					layer_name: &'static str,
					layer_depth: u8,
					flags: ProtolayerFlags,
					kind: ProtolayerKind,
	) -> Protoregion {

		let next_kind_base_slc_pos = match kind {
			ProtolayerKind::Cellular(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			ProtolayerKind::Axonal(ref axon_kind) => self.depth_axon_kind(&axon_kind),
		};
		
		let cl = Protolayer {
			name : layer_name,
			kind: kind,
			base_slc_pos: 0, 
			kind_base_slc_pos: next_kind_base_slc_pos,
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
			panic!("Protoregion::add(): Cannot add new layers after region is frozen.");
		}		

		self.layers.insert(layer.name, layer);

		//println!("Looking for cell_kind:{:?}", &ck_tmp);

		/*match self.cel_layer_kind_slc_lists.get(&ck_tmp) {
			Some(vec) 	=> println!("Found Vector with len: {}",vec.len()),
			None 		=> println!("Vector NOT FOUND"),
		};*/
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
							
							layer.kind_base_slc_pos = vec.len() as u8;
							//layer.kind_base_slc_pos = std::num::cast(vec.len()).expect("Protoregion::add()");
							//println!("{:?} base_slc_pos: {}", cell_kind, layer.kind_base_slc_pos);

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
							
							layer.kind_base_slc_pos = vec.len() as u8;

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


	// SET_LAYER_DEPTH(): ASSUMES PROPER FLAG UNIQUENESS CONSTRAINS ALREADY APPLIED
	pub fn set_layer_depth(&mut self, flags: ProtolayerFlags, depth: u8) {
		if self.frozen { 
			panic!("region::Protoregion::set_layer_depth(): \
				Cannot set layer depth after region has been frozen."); 
		} 

		for (layer_name, mut layer) in self.layers.iter_mut() {
			if (layer.flags & flags) == flags {
				//println!(" ##### SETTING LAYER DEPTH FOR LAYER: '{}' TO: {} #####", layer_name, depth);
				layer.depth = depth;
			}
		}
	}
 

 	// 	PROTOREGION::FREEZE():
 	// 		- What a mess...
	// 		- Need to revamp how axon_types and cell_types are stored before we can do much with it
	// 		- cel_layer_kind_slc_lists being a vector needs to change asap
	//
	// 	<<<<< TODO: VERIFY FLAG UNIQUENESS, APPROPRIATENESS 	
	pub fn freeze(&mut self, protoarea: &Protoarea) {
		if self.frozen {
			return;
		} else {
			// AFFERENT INPUT COMES FROM EFFERENT AREAS, EFFERENT INPUT COMES FROM AFFERENT AREAS
			self.set_layer_depth(layer::AFFERENT_INPUT, protoarea.efferent_areas.len() as u8);
			self.set_layer_depth(layer::EFFERENT_INPUT, protoarea.afferent_areas.len() as u8);
			self.frozen = true;
		}		

		self.gen_slc_lists();

		/* (0) START COUNTER FOR ABSOLUTE BASE ROWS */
		let mut next_base_slc = 0u8;

		/* (1) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL SPATIAL LAYER KINDS */	
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

		/* (2) ADD ABSOLUTE BASE_ROW_IDS FOR ALL CELLULAR LAYER KINDS */
		for (cell_kind, list) in &self.cel_layer_kind_slc_lists {
			self.cel_layer_kind_base_slc_ids.insert(cell_kind.clone(), next_base_slc);
			println!("    Adding Cell Kind: '{:?}', len: {}, kind_base_slc: {}", cell_kind, list.len(), next_base_slc);
			assert!(list.len() == self.depth_cell_kind(&cell_kind) as usize);
			next_base_slc += list.len() as u8;
			//next_base_slc += std::num::cast::<usize, u8>(list.len()).expect("cortical_region::Protoregion::freeze()");
		}

		/* (2b) SAVE DEMARCATION BETWEEN VERTICAL (SPATIAL) AND HORIZONTAL ROWS */
		self.hrz_demarc = next_base_slc;

		/* (3) ADD ABSOLUTE BASE_ROW_IDS FOR AXONAL HORIZONTAL LAYER KINDS */	
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

		/* (4) SET BASE ROW POSITION ON INDIVIDUAL NON-HORIZONTAL LAYERS */
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {

				&ProtolayerKind::Cellular(ref protocell) => {
					layer.base_slc_pos = self.cel_layer_kind_base_slc_ids[&protocell.cell_kind] + layer.kind_base_slc_pos;
					print!("    <{}>: ProtocellKind::{:?} ", layer_name, &protocell.cell_kind);
				},

				&ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Horizontal => continue,

						_ => {
							layer.base_slc_pos = self.axn_layer_kind_base_slc_ids[axon_kind] + layer.kind_base_slc_pos;
							print!("    <{}>: ProtoaxonKind::{:?} ", layer_name, axon_kind);
						},
					}
				},
			}

			for i in layer.base_slc_pos..(layer.base_slc_pos + layer.depth()) {
				self.slc_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
			print!("\n");
		}

		/* (5) SET BASE ROW POSITION ON INDIVIDUAL HORIZONTAL LAYERS */
		for (layer_name, layer) in self.layers.iter_mut() {
			match &layer.kind {
				&ProtolayerKind::Cellular(ref protocell) => continue,

				&ProtolayerKind::Axonal(ref axon_kind) => {
					match axon_kind {
						&ProtoaxonKind::Horizontal => {
							layer.base_slc_pos = self.axn_layer_kind_base_slc_ids[axon_kind] + layer.kind_base_slc_pos;
							print!("    <{}>: ProtoaxonKind::{:?} ", layer_name, axon_kind);
						},

						_ => continue,
					}
				},
			}

			for i in layer.base_slc_pos..(layer.base_slc_pos + layer.depth()) {
				self.slc_map.insert(i, layer_name);
				print!("[{}] ", i);
			}
			print!("\n");
		}

		/* (6) MARVEL AT THE MOST CONVOLUTED FUNCTION EVER */
		print!("\n");
	}


	pub fn base_slc(&self, layer_name: &'static str) -> u8 {
		let ref layer = self.layers[layer_name];
		layer.base_slc_pos
	}

	pub fn base_slc_cell_kind(&self, cell_kind: &ProtocellKind) -> u8 {
		match self.cel_layer_kind_base_slc_ids.get(cell_kind) {
			Some(base_slc) 	=> base_slc.clone(),
			None 			=> panic!("Protoregion::base_slc_cell_king(): Base slc for type not found"),
		}
	}
	

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

		let mut count2 = match self.cel_layer_kind_slc_lists.get(cell_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		//println!("CKRC: kind: {:?} -> count = {}, count2 = {}", &cell_kind, count, count2);

		assert!(count as usize == count2, "Protoregion::depth_cell_kind(): mismatch");

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

		let mut count2 = match self.axn_layer_kind_slc_lists.get(axon_kind) {
			Some(vec) 	=> vec.len(),
			None 		=> 0,
		};

		assert!(count as usize == count2, "Protoregion::depth_axon_kind(): mismatch");

		count
	}	


	// ##### LAYERS #####


	pub fn layers(&self) -> &HashMap<&'static str, Protolayer> {
		&self.layers
	}	

	pub fn slcs_by_layer_name(&self, cell_kind: &ProtocellKind) -> Option<&Vec<&'static str>> {
		self.cel_layer_kind_slc_lists.get(cell_kind)
	}

	pub fn slc_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		if !self.frozen { // REPLACE WITH ASSERT (evaluate release build implications first)
			panic!("Protoregion must be frozen with .freeze() before slc_ids can be called.");
		}

		let mut slc_ids = Vec::new();

		for layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in l.base_slc_pos..(l.base_slc_pos + l.depth) {
					slc_ids.push(i);
				}
		}

		slc_ids
	}

	pub fn src_slc_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_layer_names = self[&layer_name].src_layer_names(den_type);
		
		self.slc_ids(src_layer_names)
 	}

 	pub fn dst_tuft_src_slc_ids(&self, layer_name: &'static str) -> Vec<Vec<u8>> {
 		let src_tufts = self[&layer_name].dst_src_tufts();

 		let mut src_tuft_slc_ids = Vec::with_capacity(src_tufts.len());

 		for tuft in src_tufts {
 			src_tuft_slc_ids.push(self.slc_ids(tuft));
		}

		src_tuft_slc_ids
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

 	pub fn aff_out_slcs(&self) -> Vec<u8> {
 		let mut output_slcs: Vec<u8> = Vec::with_capacity(4);
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.flags & layer::AFFERENT_OUTPUT) == layer::AFFERENT_OUTPUT {
 				let v = self.slc_ids(vec![layer.name]);
 				output_slcs.push_all(&v);
 			}
 		}

		output_slcs		
 	}

 	// TODO: VERIFY FLAG UNIQUENESS, APPROPRIATENESS
 	// DEPRICATE IN FAVOR OF LAYERS_WITH_FLAG(), RETURNING A VEC OF PROTOLAYERS
 	// REIMPLEMENT AS AN OVERLOAD OF Index & IndexMut WHICH RETURNS AN UNWRAPPED VEC OF LAYERS
 	pub fn layer_with_flag(&self, flag: ProtolayerFlags) -> Option<Protolayer> {
 		let mut input_layer: Option<Protolayer> = None;
 		
 		for (layer_name, layer) in self.layers.iter() {
 			if (layer.flags & flag) == flag {
 				input_layer = Some(layer.clone());
 			}
 		}

		input_layer		
 	}

 	pub fn slc_map(&self) -> BTreeMap<u8, &'static str> {
 		self.slc_map.clone()
	}

 	pub fn layer_name(&self, slc_id: u8) -> &'static str {
 		match self.slc_map.get(&slc_id) {
 			Some(ln) 	=> ln,
 			None 		=> "[INVALID LAYER]",
		}

	}

	pub fn hrz_demarc(&self) -> u8 {
		self.hrz_demarc
	}

	// fn layer(&self, layer_name: &'static str) -> Option<&Protolayer> {
	// 	self.layers.get(layer_name)
	// }
}

impl<'b> Index<&'b&'static str> for Protoregion
{
    type Output = Protolayer;

    fn index<'a>(&'a self, index: &'b&'static str) -> &'a Protolayer {
        self.layers.get(index).unwrap_or_else(|| panic!("Protoregion::index(): invalid layer name: '{}'", index))
    }
}

impl<'b> IndexMut<&'b&'static str> for Protoregion
{
    fn index_mut<'a>(&'a mut self, index: &'b&'static str) -> &'a mut Protolayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[Protoregion::index(): invalid layer name: '{}'", index))
    }
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum RegionKind {
	Associational,
	Sensory,
	Motor,
	Thalamic,
	//Thalamic(Box<ProtoInputSource>),
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ProtoInputSource {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader { file_name: &'static str, repeats: usize },
}

//impl Copy for RegionKind {}


/*pub struct Protoregion {
	pub layers: HashMap<&'static str, Protolayer>,
	pub kind: RegionKind,
}

impl Protoregion {
	pub fn new (kind: RegionKind)  -> Protoregion {
		let mut next_slc_id = HashMap::new();
		next_slc_id.insert(ProtocellKind::Pyramidal, 0);
		next_slc_id.insert(ProtocellKind::InhibitoryInterneuronNetwork, 0);
		next_slc_id.insert(ProtocellKind::SpinyStellate, 0);
	
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
		let (noncell_slcs, cell_slcs) = self.depth();

		let next_base_slc_pos = self.total_depth();

		let next_kind_base_slc_pos = match cell {
			Some(ref protocell) => self.depth_cell_kind(&protocell.cell_kind),
			None => noncell_slcs,
		};

		println!("Protolayer: {}, layer_depth: {}, base_slc_pos: {}, kind_base_slc_pos: {}", layer_name, layer_depth, next_base_slc_pos, next_kind_base_slc_pos);
		
		let cl = Protolayer {
			name : layer_name,
			cell: cell,
			base_slc_pos: next_base_slc_pos, 
			kind_base_slc_pos: next_kind_base_slc_pos,
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
		let mut noncell_slcs = 0u8;
		let mut cell_slcs = 0u8;
		
		for (layer_name, layer) in self.layers.iter() {
			match layer.cell {
				None => noncell_slcs += layer.depth,
				Some(_) => cell_slcs += layer.depth,
			}
		}
		(noncell_slcs, cell_slcs)
	}

	pub fn slc_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		let mut slc_ids = Vec::new();
		for &layer_name in layer_names.iter() {
			let l = &self[layer_name];
				for i in range(l.base_slc_pos, l.base_slc_pos + l.depth) {
					slc_ids.push(i);
				}
		}
		slc_ids
	}

	pub fn src_slc_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_layer_names = self[layer_name].src_layer_names(den_type);
		
		self.slc_ids(src_layer_names)
 	}

 	pub fn col_input_slc(&self) -> u8 {
 		for (layer_name, layer) in self.layers.iter() {

 		}
 		5
 	}

}

impl Index<&'static str> for Protoregion
{
    type Output = Protolayer;

    fn index<'a>(&'a self, index: &&'static str) -> &'a Protolayer {
        self.layers.get(index).unwrap_or_else(|| panic!("[Protoregion::index(): invalid layer name: \"{}\"]", index))
    }
}

impl IndexMut<&'static str> for Protoregion
{
    type Output = Protolayer;

    fn index_mut<'a>(&'a mut self, index: &&'static str) -> &'a mut Protolayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[Protoregion::index(): invalid layer name: \"{}\"]", index))
    }
}*/



 	/*pub fn kind_slc_ids(&self, layer_name: &'static str) -> Vec<u8> {

		let l = &self[layer_name];
		let mut slc_ids = Vec::new();
			for i in range(l.base_slc_pos, l.base_slc_pos + l.depth) {
				slc_ids.push(i);
			}
		return slc_ids;
	}

	pub fn kind_src_slc_ids(&self, layer_name: &'static str) -> Vec<u8> {
		let src_layer_names = self[layer_name].src_layer_names();
		
		let mut src_slc_ids = Vec::new();

		for &src_slc_name in src_layer_names.iter() {
			src_slc_ids.push_all(self.kind_slc_ids(src_slc_name).as_slc());
		}

		//println!("Protoregion::layer_srcs_slc_ids(): (name:sources:idxs) [{}]:{:?}:{:?}", layer_name, src_layer_names, src_slc_ids);
		
		src_slc_ids
 	}*/


/* AxonScope 
	
	Integererlaminar(
		Distal Dendrite Input Protolayers,
		Proximal Dendrite Input Protolayers,
		Cell Type
	)

*/


/*fn increment_slc_index(mut cri: u8, by: u8) -> u8 {
	cri += by;
	cri
}*/
