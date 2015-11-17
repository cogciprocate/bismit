// use bitflags;

use map::{ LayerFlags };
use proto::{ /*ProtocellKind,*/ Protocell, DendriteKind };
use proto::DendriteKind::{ Distal, Proximal };
use self::ProtolayerKind::{ Cellular, Axonal };
//use ocl;


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protolayer {
	pub name: &'static str,
	//pub kind: Option<Protocell>,
	pub kind: ProtolayerKind,
	pub base_slc_id: u8, // <<<<< REMOVE THE '_pos'
	pub kind_base_slc_id: u8, // <<<<< ''
	pub depth: u8,
	pub flags: LayerFlags,
}

impl Protolayer {
	/*pub fn new(
				name: &'static str,
				cell: Option<Protocell>,
				base_slc_id: u8,
				kind_base_slc_id: u8,
				depth: u8,
				flags: LayerFlags,
	) -> Protolayer {
		Protolayer {
			name: name,
			cell: cell,
			base_slc_id: base_slc_id,
			kind_base_slc_id: kind_base_slc_id,
			depth: depth,
			flags: flags,
		}
	}*/

	pub fn base_slc(&self) -> u8 {
		self.base_slc_id
	}

	pub fn depth(&self) -> u8 {
		self.depth
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	/* SRC_LAYER_NAMES(): TODO: DEPRICATE OR RENAME */
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

/*	pub fn dst_src_lyr_names(&self) -> Vec<Vec<&'static str>> {
		let layer_names = match self.kind {
			ProtolayerKind::Cellular(ref protocell) => protocell.den_dst_src_lyrs.clone(),
			_ => panic!(format!("Protolayer '{}' is not Cellular.", self.name)),
		};
	}*/

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

	// pub fn dst_src_lyrs_len(&self) -> u32 {
	// 	match self.kind {
	// 		ProtolayerKind::Cellular(ref protocell) => protocell.dst_src_lyrs_len(),
	// 		_ => 0,
	// 	}
	// }
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


/*pub enum LayerFlags {
	ColumnInput 	= 0x0001,
	ColumnOuput 	= 0x0002,
	None			= 0x0000,
}*/

