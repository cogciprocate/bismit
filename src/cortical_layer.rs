//use protocell::{ CellKind, Protocell, AxonScope };
use ocl;



pub struct CorticalLayer {
	pub cell: Option<Protocell>,
	pub base_row_id: u8,
	pub kind_base_row_pos: u8,
	pub height: u8,
}

impl CorticalLayer {
	pub fn height(&self) -> ocl::cl_uchar {
		self.height
	}

	pub fn src_layer_names(&self, den_type: DendriteType) -> Vec<&'static str> {
		let layer_names = match self.cell {
			Some(ref protocell) => match den_type {
				DendriteType::Distal =>	protocell.den_dist_srcs.clone(),
				DendriteType::Proximal => protocell.den_prox_srcs.clone(),
			},
			_ => panic!("Layer must have a cell defined to determine source layers"),
		};

		match layer_names {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}
}


pub struct Protocell {
	pub cell_kind: CellKind,
	pub den_dist_srcs: Option<Vec<&'static str>>,
	pub den_prox_srcs: Option<Vec<&'static str>>,
}

impl Protocell {
	pub fn new(
					cell_kind: CellKind,
					den_dist_srcs: Option<Vec<&'static str>>,
					den_prox_srcs: Option<Vec<&'static str>>, 
	) -> Protocell {
		Protocell {
			cell_kind: cell_kind,
			den_dist_srcs: den_dist_srcs,
			den_prox_srcs: den_prox_srcs,
		}
	}
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellKind {
	Pyramidal,
	SpinyStellate,
	AspinyStellate,
}
// excitatory spiny stellate
// inhibitory aspiny stellate 


#[derive(Copy, PartialEq, Debug, Clone)]
pub enum DendriteType {
	Proximal,
	Distal,
}

/*#[derive(PartialEq, Debug, Clone)]
pub enum AxonScope {
	Interregional,
	Interlaminar,
}*/
