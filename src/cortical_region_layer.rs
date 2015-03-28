use protocell::{ CellKind, Protocell, DendriteKind };
use ocl;


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct CorticalRegionLayer {
	pub name: &'static str,
	pub cell: Option<Protocell>,
	pub base_row_pos: ocl::cl_uchar,
	pub kind_base_row_pos: ocl::cl_uchar,
	pub depth: ocl::cl_uchar,
	pub flags: LayerFlags,
}

impl CorticalRegionLayer {
	/*pub fn new(
				name: &'static str,
				cell: Option<Protocell>,
				base_row_pos: u8,
				kind_base_row_pos: u8,
				depth: u8,
				flags: LayerFlags,
	) -> CorticalRegionLayer {
		CorticalRegionLayer {
			name: name,
			cell: cell,
			base_row_pos: base_row_pos,
			kind_base_row_pos: kind_base_row_pos,
			depth: depth,
			flags: flags,
		}
	}*/

	pub fn base_row_pos(&self) -> ocl::cl_uchar {
		self.base_row_pos
	}

	pub fn depth(&self) -> ocl::cl_uchar {
		self.depth
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn src_layer_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
		let layer_names = match self.cell {
			Some(ref protocell) => match den_type {
				DendriteKind::Distal =>	protocell.den_dst_srcs.clone(),
				DendriteKind::Proximal => protocell.den_prx_srcs.clone(),
				//DendriteKind::Apical => protocell.den_apc_srcs.clone(),
			},
			_ => panic!("Layer must have a cell defined to determine source layers"),
		};

		match layer_names {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}
}


bitflags! {
	#[derive(Debug)]
	flags LayerFlags: u32 {
		const COLUMN_INPUT 	= 0b00000001,
		const COLUMN_OUTPUT	= 0b10000000,
		const DEFAULT		= 0b00000000,
	}
}

/*pub enum LayerFlags {
	ColumnInput 	= 0x0001,
	ColumnOuput 	= 0x0002,
	None			= 0x0000,
}*/

