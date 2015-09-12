use bitflags;

use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use proto::cell::DendriteKind::{ Distal, Proximal };
use self::ProtolayerKind::{ Cellular, Axonal };
//use ocl;


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protolayer {
	pub name: &'static str,
	//pub kind: Option<Protocell>,
	pub kind: ProtolayerKind,
	pub base_slc_pos: u8, // <<<<< REMOVE THE '_pos'
	pub kind_base_slc_pos: u8, // <<<<< ''
	pub depth: u8,
	pub flags: ProtolayerFlags,
}

impl Protolayer {
	/*pub fn new(
				name: &'static str,
				cell: Option<Protocell>,
				base_slc_pos: u8,
				kind_base_slc_pos: u8,
				depth: u8,
				flags: ProtolayerFlags,
	) -> Protolayer {
		Protolayer {
			name: name,
			cell: cell,
			base_slc_pos: base_slc_pos,
			kind_base_slc_pos: kind_base_slc_pos,
			depth: depth,
			flags: flags,
		}
	}*/

	pub fn base_slc(&self) -> u8 {
		self.base_slc_pos
	}

	pub fn depth(&self) -> u8 {
		self.depth
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	/* SRC_LAYER_NAMES(): TODO: DEPRICATE OR RENAME */
	pub fn src_layer_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
		let layer_names = match self.kind {
			ProtolayerKind::Cellular(ref protocell) => match den_type {
				Distal => Some(protocell.den_dst_srcs.clone().unwrap()[0].clone()),
				Proximal => protocell.den_prx_srcs.clone(),
			},
			_ => panic!(format!("Protolayer '{}' is not Cellular.", self.name)),
		};

		match layer_names {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}

/*	pub fn dst_src_layer_names(&self) -> Vec<Vec<&'static str>> {
		let layer_names = match self.kind {
			ProtolayerKind::Cellular(ref protocell) => protocell.den_dst_srcs.clone(),
			_ => panic!(format!("Protolayer '{}' is not Cellular.", self.name)),
		};
	}*/

	pub fn dst_src_tufts(&self) -> Vec<Vec<&'static str>> {
		let layer_tufts = match self.kind {
			ProtolayerKind::Cellular(ref protocell) => protocell.den_dst_srcs.clone(),
			_ => panic!(format!("Protolayer '{}' is not Cellular.", self.name)),
		};

		match layer_tufts {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}

	pub fn dst_src_tufts_len(&self) -> u32 {
		match self.kind {
			ProtolayerKind::Cellular(ref protocell) => protocell.dst_src_tufts_len(),
			_ => 0,
		}
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
				match pc.den_dst_srcs {
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


bitflags! {
	#[derive(Debug)]
	flags ProtolayerFlags: usize {
		const DEFAULT				= 0b00000000,
		const AFFERENT_INPUT		= 0b00000001,
		const AFFERENT_OUTPUT		= 0b00000010,
		const SPATIAL_ASSOCIATIVE 	= 0b00000100,
		const TEMPORAL_ASSOCIATIVE 	= 0b00001000,
		const EFFERENT_INPUT		= 0b00010000,
		const EFFERENT_OUTPUT		= 0b00100000,
	}
}

/*pub enum ProtolayerFlags {
	ColumnInput 	= 0x0001,
	ColumnOuput 	= 0x0002,
	None			= 0x0000,
}*/

