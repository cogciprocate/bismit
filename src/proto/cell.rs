//use bitflags;
use proto::layer::ProtolayerKind::{ self, Cellular };


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protocell {
	pub dens_per_cel_l2: u8,
	pub syns_per_den_l2: u8,
	pub cell_kind: ProtocellKind,
	pub den_dst_srcs: Option<Vec<&'static str>>,
	pub den_prx_srcs: Option<Vec<&'static str>>,
	//pub flags: CellFlags,
}

impl Protocell {
	pub fn new(
					cell_kind: ProtocellKind,
					den_dst_srcs: Option<Vec<&'static str>>,
					den_prx_srcs: Option<Vec<&'static str>>, 
					dens_per_cel_l2: u8,
					syns_per_den_l2: u8,
					//flags: CellFlags,
	) -> Protocell {
			// DO SOME CHECKS ON PARAMETERS (certain cell types must/mustn't have certain dendritic segments)
			// REMOVE FLAGS
		Protocell {
			cell_kind: cell_kind,
			den_dst_srcs: den_dst_srcs,
			den_prx_srcs: den_prx_srcs,
			dens_per_cel_l2: dens_per_cel_l2,
			syns_per_den_l2: syns_per_den_l2,
			//flags: flags,
		}
	}

	/* NEW_PYRAMIDAL(): 
		- get rid of proximal source (maybe)
	*/
	pub fn new_pyramidal(dens_per_cel_l2: u8, syns_per_den_l2: u8, dst_srcs: Vec<&'static str>) -> ProtolayerKind {
		Cellular(Protocell {
			dens_per_cel_l2: dens_per_cel_l2,
			syns_per_den_l2: syns_per_den_l2,
			cell_kind: ProtocellKind::Pyramidal,
			den_dst_srcs: Some(dst_srcs),
			den_prx_srcs: None,
			//den_prx_srcs: Some(vec![prx_src]),
			//flags: flags,
		})
	}

	pub fn new_spiny_stellate(dens_per_cel_l2: u8, syns_per_den_l2: u8, prx_srcs: Vec<&'static str>) -> ProtolayerKind {
		Cellular(Protocell {
			dens_per_cel_l2: dens_per_cel_l2,
			syns_per_den_l2: syns_per_den_l2,
			cell_kind: ProtocellKind::SpinyStellate,
			den_dst_srcs: None,
			den_prx_srcs: Some(prx_srcs),
			//flags: flags,
		})
	}
}


#[derive(Copy, PartialEq, Debug, Clone, Eq, Hash)]
pub enum ProtocellKind {
	Pyramidal,
	SpinyStellate,
	AspinyStellate,
	Nada,
}


#[derive(Copy, PartialEq, Debug, Clone)]
pub enum DendriteKind {
	Proximal,
	Distal,
}

pub enum DendriteClass {
	Apical,
	Distal,
}

//#[derive(PartialEq, Debug, Clone, Eq, Hash)]
bitflags! {
	#[derive(Debug)]
	flags CellFlags: u32 {
		const HAPPY 		= 0b00000001,
		const SAD			= 0b00000010,
		const NONE			= 0b00000000,
	}
}

 





//pub struct ProtocellPyramidal


/*#[derive(PartialEq, Debug, Clone, Hash)]
pub enum CellPrototype {
	Pyramidal { 
		prx_src: &'static str,
		dst_srcs: Vec<&'static str>,
	},

	SpinyStellate {
		prx_srcs: Vec<&'static str>,
	},

	PeakColumns {
		prx_srcs: Vec<&'static str>,
	},

	None,
}


#[derive(PartialEq, Debug, Clone)]
pub enum CellBlueprint {
	Pyramidal {
		dens: u8,
		syns_per_den: u8,
		flags: CellFlags,
	},

	SpinyStellate {
		dens: u8,
		syns_per_den: u8,
		flags: CellFlags,
	},

	PeakColumns {
		dens: u8,
		syns_per_den: u8,
		flags: CellFlags,
	},
}*/


/*#[derive(PartialEq, Debug, Clone)]
pub enum AxonScope {
	Integererregional,
	Integererlaminar,
}*/
