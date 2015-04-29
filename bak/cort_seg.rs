//use synaptic_segments::{ };
use common;
use axon_space::{ AxonSpace };
use syn_segs::{ SynSegs };

pub struct CortSeg {
	syn_segs: Vec<&'static str>,
	name: &'static str,		// probably temporary
	size: usize, 	// in Columns

}

impl CortSeg {
	pub fn new(name: &'static str, size: usize) -> CortSeg {
		let syn_segs = Vec::with_capacity(common::LAYERS_PER_SEGMENT);

		CortSeg {
			syn_segs: syn_segs,
			name: name,
			size: size,
		}
	}

	pub fn gen_layers(&mut self, input_region: &'static str, mut axon_space: &mut AxonSpace, mut syn_segs: &mut SynSegs) {
		let test = syn_segs.segs["visual"].name;

		for i in range(1, common::LAYERS_PER_SEGMENT) {
			//syn_segs.add
		}
	}
}
