use cmn;
use ocl::{ self, Ocl, WorkSize };
use ocl::{ Envoy };
use proto::areas::{ ProtoAreas, Width };
use proto::regions::{ ProtoRegion, ProtoRegionKind };
use proto::cell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use minicolumns::{ MiniColumns };
use peak_column::{ PeakColumn };
use pyramidals::{ Pyramidal };


use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };
use std::collections::{ BTreeMap };



pub struct RegionCells {
	pub width: u32,
	//pub depth_axonal: u8,
	//pub depth_cellular: u8,
	pub row_map: BTreeMap<u8, &'static str>,
	//pub region: ProtoRegion,
	pub axns: Axons,
	pub cols: MiniColumns,
	pub pyrs: Pyramidal,
	//pub soma: Somata,
	pub aux: Aux,
	ocl: ocl::Ocl,
	counter: usize,
}

impl RegionCells {
	pub fn new(region: &ProtoRegion, areas: &ProtoAreas, ocl: &Ocl) -> RegionCells {
		//let (depth_axonal, depth_cellular) = region.depth();
		let width = areas.width(&region.kind);

		//print!("\nRegionCells::new(): depth_axonal: {}, depth_cellular: {}, width: {}", depth_axonal, depth_cellular, width);

		//assert!(depth_cellular > 0, "region_cells::RegionCells::new(): Region has no cellular layers.");

		let aux = Aux::new(width, 1, ocl);
		let axns = Axons::new(width, region, ocl);
		let pyrs = Pyramidal::new(width, region, &axns, &aux, ocl);
		let cols = MiniColumns::new(width, region, &axns, &pyrs, &aux, ocl);
		

		let mut region_cells = RegionCells {
			width: width,
			//depth_axonal: depth_axonal,
			//depth_cellular: depth_cellular,
			row_map: region.row_map(),
			//region: region,
			axns: axns,
			cols: cols,
			pyrs: pyrs,
			//soma: Somata::new(width, depth_cellular, region, ocl),
			aux: aux,
			ocl: ocl.clone(),
			counter: 0,
		};

		region_cells.init_kernels();

		region_cells
	}

	pub fn init_kernels(&mut self) {
		//self.axns.init_kernels(&self.cols.asps, &self.cols, &self.aux)
		//self.cols.syns.init_kernels(&self.axns, ocl);
		self.pyrs.init_kernels(&self.cols, &self.axns, &self.aux);
	}

	pub fn cycle(&mut self, region: &ProtoRegion) {
		//self.soma.dst_dens.cycle(&self.axns, &self.ocl);
		//self.soma.cycle(&self.ocl);
		//self.soma.inhib(&self.ocl);
		//self.axns.cycle(&self.soma, &self.ocl);
		//self.soma.ltp(&self.ocl);
		//self.soma.dst_dens.syns.decay(&mut self.soma.rand_ofs, &self.ocl);

		let ltp: bool = cmn::LEARNING_ACTIVE;
		
		self.cols.cycle(ltp);
		
		self.pyrs.activate(ltp);	
		
		self.pyrs.cycle();	

		self.cols.output();

		self.regrow(region);

	}

	pub fn regrow(&mut self, region: &ProtoRegion) {
		if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
			//print!("$");
			self.cols.regrow(region);
			self.pyrs.regrow(region);
			self.counter = 0;
		} else {
			self.counter += 1;
		}
	}
}


pub struct Aux {
	depth: u8,
	width: u32,
	pub ints_0: Envoy<ocl::cl_int>,
	pub ints_1: Envoy<ocl::cl_int>,
	pub chars_0: Envoy<ocl::cl_char>,
	pub chars_1: Envoy<ocl::cl_char>,
}

impl Aux {
	pub fn new(width: u32, depth: u8, ocl: &Ocl) -> Aux {

		let width_multiplier: u32 = 512;

		Aux { 
			ints_0: Envoy::<ocl::cl_int>::new(width * width_multiplier, depth, 0, ocl),
			ints_1: Envoy::<ocl::cl_int>::new(width * width_multiplier, depth, 0, ocl),
			chars_0: Envoy::<ocl::cl_char>::new(width, depth, 0, ocl),
			chars_1: Envoy::<ocl::cl_char>::new(width, depth, 0, ocl),
			depth: depth,
			width: width,
		}
	}
}



/*pub struct Somata {
	depth: u8,
	width: u32,
	pub dst_dens: Dendrites,
	pub states: Envoy<ocl::cl_uchar>,
	pub hcol_max_vals: Envoy<ocl::cl_uchar>,
	pub hcol_max_ids: Envoy<ocl::cl_uchar>,
	pub rand_ofs: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(width: u32, depth: u8, region: &ProtoRegion, ocl: &Ocl) -> Somata {
		Somata { 
			depth: depth,
			width: width,
			states: Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl),
			hcol_max_vals: Envoy::<ocl::cl_uchar>::new(width / cmn::COLUMNS_PER_HYPERCOLUMN, depth, cmn::STATE_ZERO, ocl),
			hcol_max_ids: Envoy::<ocl::cl_uchar>::new(width / cmn::COLUMNS_PER_HYPERCOLUMN, depth, 0u8, ocl),
			rand_ofs: Envoy::<ocl::cl_char>::shuffled(256, 1, -128, 127, ocl),
			dst_dens: Dendrites::new(width, depth, DendriteKind::Distal, cmn::DENDRITES_PER_CELL_DISTAL, region, ocl),

		}
	}

	fn cycle_pre(&self, dst_dens: &Dendrites, prx_dens: &Dendrites, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_cycle_pre");
		ocl::set_kernel_arg(1, prx_dens.states.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);

		let gws = (self.depth as usize, self.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	}

	fn cycle(&self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_cycle_post");
		ocl::set_kernel_arg(0, self.dst_dens.states.buf, kern);
		//ocl::set_kernel_arg(1, self.bsl_prx_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.states.buf, kern);
		ocl::set_kernel_arg(2, self.depth as u32, kern);

		let gws = (self.depth as usize, self.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	}

	pub fn inhib(&self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_inhib");
		ocl::set_kernel_arg(0, self.states.buf, kern);
		ocl::set_kernel_arg(1, self.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(2, self.hcol_max_vals.buf, kern);
		let mut kern_width = self.width as usize / cmn::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.depth as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

		ocl::set_kernel_arg(0, self.aux.chars_0.buf, kern);
		ocl::set_kernel_arg(1, self.aux.chars_1.buf, kern);
		kern_width = kern_width / (1 << grp_size_log2);
		let gws = (self.depth_cellular as usize, self.width as usize / 64);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);
	}

	pub fn ltp(&mut self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "syns_ltp");
		ocl::set_kernel_arg(0, self.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(1, self.dst_dens.syns.states.buf, kern);
		ocl::set_kernel_arg(2, self.dst_dens.thresholds.buf, kern);
		ocl::set_kernel_arg(3, self.dst_dens.states.buf, kern);
		ocl::set_kernel_arg(4, self.dst_dens.syns.strengths.buf, kern);
		ocl::set_kernel_arg(5, self.rand_ofs.buf, kern);

		let mut kern_width = self.width as usize / cmn::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.depth as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);
	}
}*/




