use protocell::{ CellKind, Protocell, DendriteKind };
use ocl;



pub struct CorticalRegionLayer {
	pub name: &'static str,
	pub cell: Option<Protocell>,
	pub base_row_id: u8,
	pub kind_base_row_pos: u8,
	pub height: u8,
	pub flags: LayerFlags,
}

impl CorticalRegionLayer {
	/*pub fn new(
				name: &'static str,
				cell: Option<Protocell>,
				base_row_id: u8,
				kind_base_row_pos: u8,
				height: u8,
				flags: LayerFlags,
	) -> CorticalRegionLayer {
		CorticalRegionLayer {
			name: name,
			cell: cell,
			base_row_id: base_row_id,
			kind_base_row_pos: kind_base_row_pos,
			height: height,
			flags: flags,
		}
	}*/
	pub fn height(&self) -> ocl::cl_uchar {
		self.height
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

