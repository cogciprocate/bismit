use ocl;
use envoy::{ Envoy };
use cells::{ Synapses };
//use axon_space::{ AxonRegion };

use std::ops::{ Range };
use std::collections::{ HashMap };



pub struct SynSegs {
	pub segs: HashMap<&'static str, SynSeg>,
	prox_syn_bufs: SynapseBufs,
	dist_syn_bufs: SynapseBufs,
}

impl SynSegs {
	pub fn new(dist_syns: &Synapses) -> SynSegs {
		let prox_syn_bufs = SynapseBufs {
			vals: 0 as ocl::cl_mem,		// prox_syns.values.buf,
			strs: 0 as ocl::cl_mem,		// prox_syns.strengths.buf,
			adrs: 0 as ocl::cl_mem, 		// prox_syns.src_idxs.buf,
		};

		let dist_syn_bufs = SynapseBufs {
			vals: dist_syns.values.buf,
			strs: dist_syns.strengths.buf,
			adrs: dist_syns.src_idxs.buf,
		};

		SynSegs { 
			segs: HashMap::new(),
			prox_syn_bufs: prox_syn_bufs,
			dist_syn_bufs: dist_syn_bufs,
		}
	}

	pub fn new_segment(&mut self, name: &'static str, start: usize, end: usize, seg_type: SegType) {
		let syn_seg = SynSeg::new(name, start, end, seg_type);
		self.insert(name, syn_seg);	
	}

	pub fn insert(&mut self, name: &'static str, syn_seg: SynSeg) {
		let opt = self.segs.insert(name, syn_seg);
		match opt {
			Some(x)		=> panic!("Cannot insert duplicate SynSegs into SynSegs"),
			None		=> (),
		}
	}

/*	pub fn segment(&mut self, name: &'static str) -> &SynSeg {

	}*/
}


pub struct SynSeg {
	pub range: Range<usize>,
	pub name: &'static str,
	//sub_segs: Vec<SynSeg>,
	pub seg_type: SegType,

}

impl SynSeg {
	pub fn new(name: &'static str, start: usize, end: usize, seg_type: SegType) -> SynSeg {
		SynSeg {
			range: Range { start: start, end: end },
			name: name,
			//sub_segs: Vec::new(),
			seg_type: seg_type,
		} 
	}

	/*pub fn gen_sub_segs(&mut self, ss_len: usize) {
		assert!(self.range.len() % ss_len != 0, "synaptic_segments::SynSeg::gen_sub_segs(): synapse segment length must be wholly divisible by sub-segment length.");
		}

		sss = self.range.len() / ss_len;
		self.sub_segs.clear();
		for i in range(0, sss) {
			
		}
	}*/
}


pub struct SynapseBufs {
	vals: ocl::cl_mem,
	strs: ocl::cl_mem,
	adrs: ocl::cl_mem,
}


pub enum SegType {
	Proximal,
	Distal,
}
