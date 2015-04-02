use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use columns::{ Columns };
use peak_column::{ PeakColumn };
use pyramidal::{ Pyramidal };


use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };



pub struct Cells {
	pub width: u32,
	pub depth_noncellular: u8,
	pub depth_cellular: u8,
	ocl: ocl::Ocl,
	pub axns: Axons,
	pub cols: Columns,
	pub pyrs: Pyramidal,
	//pub soma: Somata,
	pub aux: Aux,

}

impl Cells {
	pub fn new(region: &CorticalRegion, areas: &CorticalAreas, ocl: &Ocl) -> Cells {
		let (depth_noncellular, depth_cellular) = region.depth();
		let depth_total = depth_noncellular + depth_cellular;
		let width = areas.width(&region.kind);

		//print!("\nCells::new(): depth_noncellular: {}, depth_cellular: {}, width: {}", depth_noncellular, depth_cellular, width);

		assert!(depth_cellular > 0, "cells::Cells::new(): Region has no cellular layers.");

		let aux = Aux::new(width, depth_cellular, ocl);
		let axns = Axons::new(width, depth_noncellular, depth_cellular, region, ocl);
		let pyrs = Pyramidal::new(width, region, &axns, ocl);
		let cols = Columns::new(width, region, &axns, &pyrs, &aux, ocl);
		

		let mut cells = Cells {
			width: width,
			depth_noncellular: depth_noncellular,
			depth_cellular: depth_cellular,
			axns: axns,
			cols: cols,
			pyrs: pyrs,
			//soma: Somata::new(width, depth_cellular, region, ocl),
			aux: aux,
			ocl: ocl.clone(),
			
		};

		cells.init_kernels();

		cells
	}

	pub fn init_kernels(&mut self) {
		//self.axns.init_kernels(&self.cols.asps, &self.cols, &self.aux)
		//self.cols.syns.init_kernels(&self.axns, ocl);
		self.pyrs.init_kernels(&self.cols, &self.axns);
	}

	pub fn cycle(&mut self) {
		
		//self.soma.dst_dens.cycle(&self.axns, &self.ocl);
		//self.soma.cycle(&self.ocl);
		//self.soma.inhib(&self.ocl);
		//self.axns.cycle(&self.soma, &self.ocl);
		//self.soma.learn(&self.ocl);
		//self.soma.dst_dens.syns.decay(&mut self.soma.rand_ofs, &self.ocl);

		

		
		self.cols.cycle();

		self.pyrs.activate();
		
		self.pyrs.cycle();

		self.cols.output();
		
		//self.axns.cycle();
	}
}


pub struct Aux {
	depth: u8,
	width: u32,
	pub ints_0: Envoy<ocl::cl_int>,
	pub ints_1: Envoy<ocl::cl_int>,
	pub chars_0: Envoy<ocl::cl_uchar>,
	pub chars_1: Envoy<ocl::cl_uchar>,
}

impl Aux {
	pub fn new(width: u32, depth: u8, ocl: &Ocl) -> Aux {

		let width_multiplier: u32 = 100;

		Aux { 
			ints_0: Envoy::<ocl::cl_int>::new(width * width_multiplier, depth, 0, ocl),
			ints_1: Envoy::<ocl::cl_int>::new(width * width_multiplier, depth, 0, ocl),
			chars_0: Envoy::<ocl::cl_uchar>::new(width, depth, 0, ocl),
			chars_1: Envoy::<ocl::cl_uchar>::new(width, depth, 0, ocl),
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
	pub fn new(width: u32, depth: u8, region: &CorticalRegion, ocl: &Ocl) -> Somata {
		Somata { 
			depth: depth,
			width: width,
			states: Envoy::<ocl::cl_uchar>::new(width, depth, common::STATE_ZERO, ocl),
			hcol_max_vals: Envoy::<ocl::cl_uchar>::new(width / common::COLUMNS_PER_HYPERCOLUMN, depth, common::STATE_ZERO, ocl),
			hcol_max_ids: Envoy::<ocl::cl_uchar>::new(width / common::COLUMNS_PER_HYPERCOLUMN, depth, 0u8, ocl),
			rand_ofs: Envoy::<ocl::cl_char>::shuffled(256, 1, -128, 127, ocl),
			dst_dens: Dendrites::new(width, depth, DendriteKind::Distal, common::DENDRITES_PER_CELL_DISTAL, region, ocl),

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
		let mut kern_width = self.width as usize / common::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.depth as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

		ocl::set_kernel_arg(0, self.aux.chars_0.buf, kern);
		ocl::set_kernel_arg(1, self.aux.chars_1.buf, kern);
		kern_width = kern_width / (1 << grp_size_log2);
		let gws = (self.depth_cellular as usize, self.width as usize / 64);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);
	}

	pub fn learn(&mut self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "syns_learn");
		ocl::set_kernel_arg(0, self.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(1, self.dst_dens.syns.states.buf, kern);
		ocl::set_kernel_arg(2, self.dst_dens.thresholds.buf, kern);
		ocl::set_kernel_arg(3, self.dst_dens.states.buf, kern);
		ocl::set_kernel_arg(4, self.dst_dens.syns.strengths.buf, kern);
		ocl::set_kernel_arg(5, self.rand_ofs.buf, kern);

		let mut kern_width = self.width as usize / common::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.depth as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);
	}
}*/




