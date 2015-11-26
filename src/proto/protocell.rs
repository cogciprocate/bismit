//use bittags;
use proto::layer::LayerKind::{ self, Cellular };
//use std::option::{ Option };
use cmn;

/* PROTOCELL:
 		Merge srcs to a Vec<Box<Vec<..>>>, A Vec of src vec lists
			- use composable functions to define
			- maybe redefine Vec<&'static str> to it's own type with an enum property
			defining what it's source type is
*/
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Protocell {
	pub dens_per_tuft_l2: u8,
	pub syns_per_den_l2: u8,
	pub cols_per_cel_l2: u8,
	pub cell_kind: CellKind,
	pub cell_class: CellClass,
	pub den_prx_src_lyrs: Option<Vec<&'static str>>,	
	pub den_dst_src_lyrs: Option<Vec<Vec<&'static str>>>,
	pub den_prx_syn_reach: i8,
	pub den_dst_syn_reach: i8,	
	pub den_thresh_init: Option<u32>,
}

impl Protocell {
	pub fn new(					
					dens_per_tuft_l2: u8,
					syns_per_den_l2: u8,
					cols_per_cel_l2: u8,
					cell_kind: CellKind,
					cell_class: CellClass,
					den_dst_src_lyrs: Option<Vec<Vec<&'static str>>>,
					den_prx_src_lyrs: Option<Vec<&'static str>>,
					den_prx_syn_reach: i8,
					den_dst_syn_reach: i8,
					thresh: Option<u32>,
	) -> Protocell {
			// DO SOME CHECKS ON PARAMETERS (certain cell types must/mustn't have certain dendritic segments)

		Protocell {
			cell_kind: cell_kind,
			cell_class: cell_class,
			dens_per_tuft_l2: dens_per_tuft_l2,
			syns_per_den_l2: syns_per_den_l2,
			cols_per_cel_l2: 0,
			den_dst_src_lyrs: den_dst_src_lyrs,
			den_prx_src_lyrs: den_prx_src_lyrs,
			den_prx_syn_reach: den_prx_syn_reach,
			den_dst_syn_reach: den_dst_syn_reach,
			den_thresh_init: thresh,
		}.validate()
	}	

	pub fn pyramidal(dens_per_tuft_l2: u8, syns_per_den_l2: u8, dst_srcs: Vec<&'static str>, 
				thresh: u32, dst_reach: i8) -> LayerKind 
	{
		Cellular(Protocell {
			dens_per_tuft_l2: dens_per_tuft_l2,
			syns_per_den_l2: syns_per_den_l2,
			cols_per_cel_l2: 0,
			cell_kind: CellKind::Pyramidal,
			cell_class: CellClass::Material,
			den_dst_src_lyrs: Some(vec![dst_srcs]),
			den_prx_src_lyrs: None,
			den_prx_syn_reach: dst_reach,
			den_dst_syn_reach: dst_reach,
			den_thresh_init: Some(thresh),			
		}.validate())
	}

	// SWITCH TO DISTAL
	pub fn spiny_stellate(syns_per_den_l2: u8, prx_srcs: Vec<&'static str>, thresh: u32,
				prx_reach: i8) -> LayerKind 
	{
		Cellular(Protocell {
			dens_per_tuft_l2: 0,
			syns_per_den_l2: syns_per_den_l2,
			cols_per_cel_l2: 0,
			cell_kind: CellKind::SpinyStellate,
			cell_class: CellClass::Material,
			den_dst_src_lyrs: None, // Some(vec![dst_srcs]),
			den_prx_src_lyrs: Some(prx_srcs),
			den_prx_syn_reach: prx_reach,
			den_dst_syn_reach: prx_reach,
			den_thresh_init: Some(thresh),
		}.validate())
	}

	pub fn inhibitory(cols_per_cel_l2: u8, dst_src: &'static str) -> LayerKind 
	{
		Cellular(Protocell {
			dens_per_tuft_l2: 0,
			syns_per_den_l2: 0,
			cols_per_cel_l2: cols_per_cel_l2,
			cell_kind: CellKind::Inhibitory,
			cell_class: CellClass::Control,
			den_dst_src_lyrs: Some(vec![vec![dst_src]]),
			den_prx_src_lyrs: None,
			den_prx_syn_reach: 0,
			den_dst_syn_reach: 0,
			den_thresh_init: None,
		}.validate())
	}

	pub fn minicolumn(psal_lyr: &'static str, ptal_lyr: &'static str,) -> LayerKind 
	{
		Cellular(Protocell {
			dens_per_tuft_l2: 0,
			syns_per_den_l2: 0,
			cols_per_cel_l2: 0,
			cell_kind: CellKind::Complex,
			cell_class: CellClass::Control,
			den_dst_src_lyrs: Some(vec![vec![psal_lyr],vec![ptal_lyr]]),
			den_prx_src_lyrs: None,
			den_prx_syn_reach: 0,
			den_dst_syn_reach: 0,
			den_thresh_init: None,
		}.validate())
	}

	pub fn validate(self) -> Protocell {
		assert!(self.den_prx_syn_reach >= 0 && self.den_dst_syn_reach >= 0, 
			"Synapse reach must be between 0..127");

		self
	}

	pub fn validate_depth(&self, depth: Option<u8>) -> Option<u8> {
		match self.cell_kind {
			CellKind::Inhibitory => Some(0),
			CellKind::Complex => Some(cmn::DEFAULT_OUTPUT_LAYER_DEPTH),
			_ => depth,
		}
	}
}


#[derive(Copy, PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellKind {
	Pyramidal,
	SpinyStellate,
	//AspinyStellate,
	Inhibitory,
	Complex,
}

#[derive(Copy, PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellClass {
	Material,
	Control,
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

// [FIXME]: Depricate?
bitflags! {
	flags CellFlags: u32 {
		const HAPPY 		= 0b00000001,
		const SAD			= 0b00000010,
		const NONE			= 0b00000000,
	}
}
