//use bitflags;
use proto::layer::ProtolayerKind::{ self, Cellular };
//use std::option::{ Option };

/* PROTOCELL:
 		Merge srcs to a Vec<Box<Vec<..>>>, A Vec of src vec lists
			- use composable functions to define
			- maybe redefine Vec<&'static str> to it's own type with an enum property
			defining what it's source type is
*/
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protocell {
	pub dens_per_grp_l2: u8,
	pub syns_per_den_l2: u8,
	pub cols_per_cel_l2: u8,
	pub cell_kind: ProtocellKind,
	pub den_dst_srcs: Option<Vec<Vec<&'static str>>>,		
	pub den_prx_srcs: Option<Vec<&'static str>>,
	pub den_thresh_init: Option<u32>,
	//pub flags: CellFlags,							
}

impl Protocell {
	pub fn new(					
					dens_per_grp_l2: u8,
					syns_per_den_l2: u8,
					cols_per_cel_l2: u8,
					cell_kind: ProtocellKind,
					den_dst_srcs: Option<Vec<Vec<&'static str>>>,
					den_prx_srcs: Option<Vec<&'static str>>,
					thresh: Option<u32>,
					//flags: CellFlags,
	) -> Protocell {
			// DO SOME CHECKS ON PARAMETERS (certain cell types must/mustn't have certain dendritic segments)
			// REMOVE FLAGS

		Protocell {
			cell_kind: cell_kind,
			dens_per_grp_l2: dens_per_grp_l2,
			syns_per_den_l2: syns_per_den_l2,
			cols_per_cel_l2: 0,
			den_dst_srcs: den_dst_srcs,
			den_prx_srcs: den_prx_srcs,
			den_thresh_init: thresh,
			//flags: flags,
		}
	}

	/* NEW_PYRAMIDAL(): 
		- get rid of proximal source (maybe)
	*/
	pub fn new_pyramidal(dens_per_grp_l2: u8, syns_per_den_l2: u8, dst_srcs: Vec<&'static str>, thresh: u32) -> ProtolayerKind {
		Cellular(Protocell {
			dens_per_grp_l2: dens_per_grp_l2,
			syns_per_den_l2: syns_per_den_l2,
			cols_per_cel_l2: 0,
			cell_kind: ProtocellKind::Pyramidal,
			den_dst_srcs: Some(vec![dst_srcs]),
			den_prx_srcs: None,
			den_thresh_init: Some(thresh),
			//den_prx_srcs: Some(vec![prx_src]),
			//flags: flags,
		})
	}

	// SWITCH TO DISTAL
	pub fn new_spiny_stellate(syns_per_den_l2: u8, dst_srcs: Vec<&'static str>, thresh: u32) -> ProtolayerKind {
		Cellular(Protocell {
			dens_per_grp_l2: 0,
			syns_per_den_l2: syns_per_den_l2,
			cols_per_cel_l2: 0,
			cell_kind: ProtocellKind::SpinyStellate,
			den_dst_srcs: Some(vec![dst_srcs]),
			den_prx_srcs: None,
			den_thresh_init: Some(thresh),
			//flags: flags,
		})
	}

	pub fn new_inhibitory(cols_per_cel_l2: u8, dst_src: &'static str) -> ProtolayerKind {
		Cellular(Protocell {
			dens_per_grp_l2: 0,
			syns_per_den_l2: 0,
			cols_per_cel_l2: cols_per_cel_l2,
			cell_kind: ProtocellKind::Inhibitory,
			den_dst_srcs: Some(vec![vec![dst_src]]),
			den_prx_srcs: None,
			den_thresh_init: None,
		})
	}

	pub fn dst_src_grps_len(&self) -> u32 {
		match self.den_dst_srcs {
			Some(ref src_grps) => src_grps.len() as u32,
			None => 0u32,
		}
	}
}


#[derive(Copy, PartialEq, Debug, Clone, Eq, Hash)]
pub enum ProtocellKind {
	Pyramidal,
	SpinyStellate,
	//AspinyStellate,
	Inhibitory,
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

	InhibitoryInterneuronNetwork {
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

	InhibitoryInterneuronNetwork {
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
