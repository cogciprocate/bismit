use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use columns::{ Columns };
use aspiny::{ AspinyStellate };
use pyramidal:: { Pyramidal };


use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };



pub struct Cells {
	pub width: u32,
	pub height_noncellular: u8,
	pub height_cellular: u8,
	ocl: ocl::Ocl,
	pub cols: Columns,
	pub asps: AspinyStellate,
	pub axns: Axons,
	pub soma: Somata,
	pub aux: Aux,

}

impl Cells {
	pub fn new(region: &CorticalRegion, areas: &CorticalAreas, ocl: &Ocl) -> Cells {
		let (height_noncellular, height_cellular) = region.height();
		print!("\nCells::new(): height_noncellular: {}, height_cellular: {}", height_noncellular, height_cellular);
		assert!(height_cellular > 0, "cells::Cells::new(): Region has no cellular layers.");
		let height_total = height_noncellular + height_cellular;
		let width = areas.width(&region.kind);

		let axns = Axons::new(width, height_noncellular, height_cellular, region, ocl);

		let cols = Columns::new(width, region, &axns, ocl);

		let asps = AspinyStellate::new(width, common::ASPINY_HEIGHT, region, &cols, ocl);

		let pyrs = Pyramidal::new(width, height_cellular, region, ocl);



		let mut cells = Cells {
			width: width,
			height_noncellular: height_noncellular,
			height_cellular: height_cellular,
			cols: cols,
			asps: asps,
			axns: axns,
			soma: Somata::new(width, height_cellular, region, ocl),
			aux: Aux::new(width, height_cellular, ocl),
			ocl: ocl.clone(),
			
		};


		cells.init_kernels(ocl);

		cells
	}

	pub fn init_kernels(&mut self, ocl: &Ocl) {
		self.axns.init_kernels(&self.asps, &self.cols, &self.aux)
		//self.cols.syns.init_kernels(&self.axns, ocl);
	}

	pub fn cycle(&mut self) {
		
		//self.soma.dst_dens.cycle(&self.axns, &self.ocl);
		//self.soma.cycle(&self.ocl);
		//self.soma.inhib(&self.ocl);
		//self.axns.cycle(&self.soma, &self.ocl);
		//self.soma.learn(&self.ocl);
		//self.soma.dst_dens.syns.decay(&mut self.soma.rand_ofs, &self.ocl);

		self.cols.cycle();
		self.asps.cycle();
		self.axns.cycle();
	}
}



pub struct Somata {
	height: u8,
	width: u32,
	pub dst_dens: Dendrites,
	pub states: Envoy<ocl::cl_uchar>,
	pub hcol_max_vals: Envoy<ocl::cl_uchar>,
	pub hcol_max_ids: Envoy<ocl::cl_uchar>,
	pub rand_ofs: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(width: u32, height: u8, region: &CorticalRegion, ocl: &Ocl) -> Somata {
		Somata { 
			height: height,
			width: width,
			states: Envoy::<ocl::cl_uchar>::new(width, height, common::STATE_ZERO, ocl),
			hcol_max_vals: Envoy::<ocl::cl_uchar>::new(width / common::COLUMNS_PER_HYPERCOLUMN, height, common::STATE_ZERO, ocl),
			hcol_max_ids: Envoy::<ocl::cl_uchar>::new(width / common::COLUMNS_PER_HYPERCOLUMN, height, 0u8, ocl),
			rand_ofs: Envoy::<ocl::cl_char>::shuffled(256, 1, -128, 127, ocl),
			dst_dens: Dendrites::new(width, height, DendriteKind::Distal, common::DENDRITES_PER_CELL_DISTAL, region, ocl),

		}
	}

	fn cycle_pre(&self, dst_dens: &Dendrites, prx_dens: &Dendrites, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_cycle_pre");
		ocl::set_kernel_arg(1, prx_dens.states.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);

		let gws = (self.height as usize, self.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	}

	fn cycle(&self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_cycle_post");
		ocl::set_kernel_arg(0, self.dst_dens.states.buf, kern);
		//ocl::set_kernel_arg(1, self.bsl_prx_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.states.buf, kern);
		ocl::set_kernel_arg(2, self.height as u32, kern);

		let gws = (self.height as usize, self.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	}

	pub fn inhib(&self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_inhib");
		ocl::set_kernel_arg(0, self.states.buf, kern);
		ocl::set_kernel_arg(1, self.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(2, self.hcol_max_vals.buf, kern);
		let mut kern_width = self.width as usize / common::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.height as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

		/*ocl::set_kernel_arg(0, self.aux.chars_0.buf, kern);
		ocl::set_kernel_arg(1, self.aux.chars_1.buf, kern);
		kern_width = kern_width / (1 << grp_size_log2);
		let gws = (self.height_cellular as usize, self.width as usize / 64);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);*/
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
		let gws = (self.height as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);
	}
}



pub struct Aux {
	height: u8,
	width: u32,
	pub ints_0: Envoy<ocl::cl_int>,
	pub ints_1: Envoy<ocl::cl_int>,
	pub chars_0: Envoy<ocl::cl_uchar>,
	pub chars_1: Envoy<ocl::cl_uchar>,
}

impl Aux {
	pub fn new(width: u32, height: u8, ocl: &Ocl) -> Aux {

		let width_multiplier: u32 = 100;

		Aux { 
			ints_0: Envoy::<ocl::cl_int>::new(width * width_multiplier, height, 0, ocl),
			ints_1: Envoy::<ocl::cl_int>::new(width * width_multiplier, height, 0, ocl),
			chars_0: Envoy::<ocl::cl_uchar>::new(width, height, 0, ocl),
			chars_1: Envoy::<ocl::cl_uchar>::new(width, height, 0, ocl),
			height: height,
			width: width,
		}
	}
}
