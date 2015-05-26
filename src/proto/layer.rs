use bitflags;

use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
//use ocl;


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protolayer {
	pub name: &'static str,
	//pub kind: Option<Protocell>,
	pub kind: ProtolayerKind,
	pub base_slice_pos: u8,
	pub kind_base_slice_pos: u8,
	pub depth: u8,
	pub flags: ProtolayerFlags,
}

impl Protolayer {
	/*pub fn new(
				name: &'static str,
				cell: Option<Protocell>,
				base_slice_pos: u8,
				kind_base_slice_pos: u8,
				depth: u8,
				flags: ProtolayerFlags,
	) -> Protolayer {
		Protolayer {
			name: name,
			cell: cell,
			base_slice_pos: base_slice_pos,
			kind_base_slice_pos: kind_base_slice_pos,
			depth: depth,
			flags: flags,
		}
	}*/

	pub fn base_slice_pos(&self) -> u8 {
		self.base_slice_pos
	}

	pub fn depth(&self) -> u8 {
		self.depth
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn src_layer_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
		let layer_names = match self.kind {
			ProtolayerKind::Cellular(ref protocell) => match den_type {
				DendriteKind::Distal =>	protocell.den_dst_srcs.clone(),
				DendriteKind::Proximal => protocell.den_prx_srcs.clone(),
				//DendriteKind::Apical => protocell.den_apc_srcs.clone(),
			},
			_ => panic!("Protolayer must have a kind defined to determine source layers"),
		};

		match layer_names {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum ProtolayerKind {
	Cellular(Protocell),
	Axonal(ProtoaxonKind),
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum ProtoaxonKind {
	Spatial,
	Horizontal,
}


bitflags! {
	#[derive(Debug)]
	flags ProtolayerFlags: u32 {
		const DEFAULT		= 0b0000000000000000,
		const COLUMN_INPUT 	= 0b0000000000000001,
		const COLUMN_OUTPUT	= 0b0000000000000010,
		const HORIZONTAL	= 0b0000000000000100,
	}
}

/*pub enum ProtolayerFlags {
	ColumnInput 	= 0x0001,
	ColumnOuput 	= 0x0002,
	None			= 0x0000,
}*/

