use map::{ LayerTags };
use proto::{ Protocell, DendriteKind };
use proto::DendriteKind::{ Distal, Proximal };
use self::ProtolayerKind::{ Cellular, Axonal };


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protolayer {
	name: &'static str,
	kind: ProtolayerKind,
	depth: Option<u8>,
	base_slc_id: u8, 
	kind_base_slc_id: u8,
	tags: LayerTags,
}

impl Protolayer {
	pub fn new(name: &'static str, kind: ProtolayerKind, depth: Option<u8>, base_slc_id: u8, 
				kind_base_slc_id: u8, tags: LayerTags) -> Protolayer
	{
		if cfg!(debug) { tags.debug_validate(); }
		
		Protolayer {name : name, kind: kind, depth: depth, base_slc_id: base_slc_id, 
			kind_base_slc_id: kind_base_slc_id, tags: tags}
	}	

	// SRC_LAYER_NAMES(): TODO: DEPRICATE OR RENAME 
	pub fn src_lyr_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
		let layer_names = match self.kind {
			ProtolayerKind::Cellular(ref protocell) => match den_type {
				Distal => Some(protocell.den_dst_src_lyrs.clone().unwrap()[0].clone()),
				Proximal => protocell.den_prx_src_lyrs.clone(),
			},
			_ => panic!(format!("Protolayer '{}' is not 'Cellular'.", self.name)),
		};

		match layer_names {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}

	pub fn dst_src_lyrs_by_tuft(&self) -> Vec<Vec<&'static str>> {
		let layers_by_tuft = match self.kind {
			ProtolayerKind::Cellular(ref protocell) => protocell.den_dst_src_lyrs.clone(),
			_ => panic!(format!("Protolayer '{}' is not 'Cellular'.", self.name)),
		};

		match layers_by_tuft {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}

	pub fn base_slc(&self) -> u8 {
		self.base_slc_id
	}

	pub fn depth(&self) -> u8 {
		match self.depth {
			Some(d) => d,
			// None => panic!("Cannot get layer depth for an axonal protolayer"),
			None => 0,
		}
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn kind(&self) -> ProtolayerKind {
		self.kind.clone()
	}

	pub fn base_slc_id(&self) -> u8 {
		self.base_slc_id
	}

	pub fn kind_base_slc_id(&self) -> u8 {
		self.kind_base_slc_id
	}

	pub fn tags(&self) -> LayerTags {
		self.tags
	}

	pub fn set_depth(&mut self, depth: u8) {
		self.depth = Some(depth);
	}

	pub fn set_base_slc_id(&mut self, id: u8) {
		self.base_slc_id = id;
	}

	pub fn set_kind_base_slc_id(&mut self, id: u8) {
		self.kind_base_slc_id = id;
	}
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum ProtolayerKind {
	Cellular(Protocell),
	Axonal(ProtoaxonKind),
}

impl ProtolayerKind {
	pub fn apical(mut self, dst_srcs: Vec<&'static str>) -> ProtolayerKind {
		match &mut self {
			&mut ProtolayerKind::Cellular(ref mut pc) => {
				match pc.den_dst_src_lyrs {
					Some(ref mut vec) => vec.push(dst_srcs),
					None => (),
				}
			},

			&mut ProtolayerKind::Axonal(_) => (),
		};
		
		self
	}
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Copy)]
pub enum ProtoaxonKind {
	Spatial,
	Horizontal,
}
