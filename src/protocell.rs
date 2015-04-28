//use bitflags;


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protocell {
	pub cell_kind: CellKind,
	pub den_dst_srcs: Option<Vec<&'static str>>,
	pub den_prx_srcs: Option<Vec<&'static str>>,
	//pub flags: CellFlags,
}

impl Protocell {
	pub fn new(
					cell_kind: CellKind,
					den_dst_srcs: Option<Vec<&'static str>>,
					den_prx_srcs: Option<Vec<&'static str>>, 
					//flags: CellFlags,
	) -> Protocell {
			// DO SOME CHECKS ON PARAMETERS (certain cell types must/mustn't have certain dendritic segments)
			// REMOVE FLAGS
		Protocell {
			cell_kind: cell_kind,
			den_dst_srcs: den_dst_srcs,
			den_prx_srcs: den_prx_srcs,
			//flags: flags,
		}
	}

	/* NEW_PYRAMIDAL(): 
		- get rid of proximal source (maybe)
	*/
	pub fn new_pyramidal(
					dst_srcs: Vec<&'static str>,
					//prx_src: &'static str,
	) -> Option<Protocell> {
		Some(Protocell {
			cell_kind: CellKind::Pyramidal,
			den_dst_srcs: Some(dst_srcs),
			den_prx_srcs: None,
			//den_prx_srcs: Some(vec![prx_src]),
			//flags: flags,
		})
	}

	pub fn new_spiny_stellate(prx_srcs: Vec<&'static str>) -> Option<Protocell> {
		Some(Protocell {
			cell_kind: CellKind::SpinyStellate,
			den_dst_srcs: None,
			den_prx_srcs: Some(prx_srcs),
			//flags: flags,
		})
	}
}


#[derive(Copy, PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellKind {
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

	PeakColumn {
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

	PeakColumn {
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
