use cmn::{ CmnError };
use map::{ self, LayerTags };
use proto::{ Protocell, DendriteKind };
use proto::DendriteKind::{ Distal, Proximal };
use self::LayerKind::{ Cellular, Axonal };


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protolayer {
	name: &'static str,
	kind: LayerKind,
	depth: Option<u8>,
	// base_slc_id: u8, 
	// kind_base_slc_id: u8,
	tags: LayerTags,
}

impl Protolayer {
	pub fn new(name: &'static str, kind: LayerKind, depth: Option<u8>, /*base_slc_id: u8, 
				kind_base_slc_id: u8,*/ tags: LayerTags) -> Protolayer
	{
		if cfg!(debug) { tags.debug_validate(); }
		
		Protolayer {name : name, kind: kind, depth: depth, /*base_slc_id: base_slc_id, 
			kind_base_slc_id: kind_base_slc_id,*/ tags: tags}
	}

	// pub fn set_depth(&mut self, depth: u8) {
	// 	self.depth = Some(depth);
	// }

	// [FIXME]: DEPRICATE:
	// pub fn depth_old_tmp(&self) -> u8 {
	// 	match self.depth {
	// 		Some(d) => d,
	// 		// None => panic!("Cannot get layer depth for an axonal protolayer"),
	// 		None => 0,
	// 	}
	// }

	// SRC_LAYER_NAMES(): TODO: DEPRICATE OR RENAME 
	pub fn src_lyr_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
		let layer_names = match self.kind {
			LayerKind::Cellular(ref protocell) => match den_type {
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
			LayerKind::Cellular(ref protocell) => protocell.den_dst_src_lyrs.clone(),
			_ => panic!(format!("Protolayer '{}' is not 'Cellular'.", self.name)),
		};

		match layers_by_tuft {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}	

	pub fn depth(&self) -> Option<u8> {
		self.depth
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn kind(&self) -> &LayerKind {
		&self.kind
	}

	pub fn axn_kind(&self) -> Result<AxonKind, CmnError> {
		match self.kind {
			Axonal(ak) => Ok(ak.clone()),
			Cellular(_) => Ok(try!(AxonKind::from_tags(self.tags))),
		}
	}

	pub fn tags(&self) -> LayerTags {
		self.tags
	}
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum LayerKind {
	Cellular(Protocell),
	Axonal(AxonKind),
}

impl LayerKind {
	pub fn axn_kind(&self) -> Option<AxonKind> {
		match self {
			&Axonal(ak) => Some(ak.clone()),
			_ => None,
		}
	}
}

impl LayerKind {
	pub fn apical(mut self, dst_srcs: Vec<&'static str>) -> LayerKind {
		match &mut self {
			&mut LayerKind::Cellular(ref mut pc) => {
				match pc.den_dst_src_lyrs {
					Some(ref mut vec) => vec.push(dst_srcs),
					None => (),
				}
			},

			&mut LayerKind::Axonal(_) => (),
		};
		
		self
	}
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Copy)]
pub enum AxonKind {
	Spatial,
	Horizontal,
	None,
}

impl AxonKind {
	// [FIXME]: Make this a Result with CmnError describing mismatch:
	pub fn from_tags<'a>(tags: LayerTags) -> Result<AxonKind, CmnError> {
		if tags.contains(map::SPATIAL) && tags.contains(map::HORIZONTAL) {
			Err(CmnError::new(format!("Error converting tags to AxonKind, tags must contain \
				only one of either 'map::SPATIAL' or 'map::HORIZONTAL'. (tags: '{:?}')", tags)))
		} else if tags.contains(map::SPATIAL) {
			Ok(AxonKind::Spatial)
		} else if tags.contains(map::HORIZONTAL) {
			Ok(AxonKind::Horizontal)
		} else {
			// Err(CmnError::new(format!("Unable to determine axon kind from tags: '{:?}'", tags)))
			Ok(AxonKind::None)
		}
	}

	pub fn matches_tags(&self, tags: LayerTags) -> bool {
		match self {
			&AxonKind::Spatial => tags.contains(map::SPATIAL),
			&AxonKind::Horizontal => tags.contains(map::HORIZONTAL),
			&AxonKind::None => (!tags.contains(map::SPATIAL)) && (!tags.contains(map::HORIZONTAL)),
		}
	}
}
