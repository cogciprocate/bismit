use common;
use ocl;
use cortical_component::{ CorticalComponent };


pub struct Axons {
	pub target_columns: CorticalComponent<ocl::cl_ushort>,
	pub target_column_synapses: CorticalComponent<ocl::cl_uchar>,
}
impl Axons {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Axons {
		Axons {
			target_columns: CorticalComponent::<ocl::cl_ushort>::new(size, 0u16, ocl),
			target_column_synapses: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
		}
	}
}


pub struct Dendrites {
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub thresholds: CorticalComponent<ocl::cl_uchar>,
}
impl Dendrites {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Dendrites {
		Dendrites {
			values: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
			thresholds: CorticalComponent::<ocl::cl_uchar>::new(size, 16u8, ocl),
		}
	}
}


pub struct Synapses {
	pub values: CorticalComponent<ocl::cl_uchar>,
	pub strengths: CorticalComponent<ocl::cl_uchar>,
}
impl Synapses {
	pub fn new(size: uint, ocl: &ocl::Ocl) -> Synapses {
		Synapses {
			values: CorticalComponent::<ocl::cl_uchar>::new(size, 0u8, ocl),
			strengths: CorticalComponent::<ocl::cl_uchar>::new(size, 16u8, ocl),
		}
	}
}


