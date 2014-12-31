use common;
use ocl;
use cortical_component::{ CorticalComponent };

pub struct Axons {
	pub target_cells: CorticalComponent<ocl::cl_ushort>,
	pub target_synapses: CorticalComponent<ocl::cl_uchar>,
}
impl Axons {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Axons {
		Axons {
			target_cells: CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl),
			target_synapses: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
		}
	}
}


pub struct Dendrites {
	pub thresholds: CorticalComponent<ocl::cl_uchar>,
	pub synapse_states: CorticalComponent<ocl::cl_ushort>,
}
impl Dendrites {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			thresholds: CorticalComponent::<ocl::cl_uchar>::new(size, 16u8, ocl),
			synapse_states: CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl),
		}
	}
}


pub struct Synapses {
	pub strengths: CorticalComponent<ocl::cl_uchar>,
}
impl Synapses {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Synapses {

		Synapses {
			strengths: CorticalComponent::<ocl::cl_uchar>::new(size, 16u8, ocl),
		}
	}
}
