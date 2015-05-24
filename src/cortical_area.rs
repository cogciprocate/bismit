use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ ProtoAreas, ProtoArea };
use proto::regions::{ ProtoRegion, ProtoRegionKind };
use proto::cell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use minicolumns::{ MiniColumns };
use peak_column::{ PeakColumns };
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



pub struct CorticalArea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	//pub depth_axonal: u8,
	//pub depth_cellular: u8,
	pub slice_map: BTreeMap<u8, &'static str>,
	//pub region: ProtoRegion,
	pub axns: Axons,
	pub mcols: MiniColumns,
	pub pyrs: Pyramidal,
	//pub soma: Somata,
	pub aux: Aux,
	ocl: ocl::Ocl,
	counter: usize,
}

impl CorticalArea {
	pub fn new(name: &'static str, region: &ProtoRegion, protoarea: &ProtoArea, ocl: &Ocl) -> CorticalArea {
		//let (depth_axonal, depth_cellular) = region.depth();
		let dims = protoarea.dims.clone();
		//let dims.width = areas.width(&region.kind);
		//let height = areas.height(&region.kind);

		//print!("\nCorticalArea::new(): depth_axonal: {}, depth_cellular: {}, width: {}", depth_axonal, depth_cellular, width);

		//assert!(depth_cellular > 0, "cortical_area::CorticalArea::new(): Region has no cellular layers.");

		let axns = Axons::new(dims, region, ocl);

		let aux_dims = CorticalDimensions::new(dims.width() * 8, dims.height() * 8, dims.depth(), 0);
		let aux = Aux::new(aux_dims, ocl);

		let pyrs_dims = dims.clone_with_depth(region.depth_cell_kind(&CellKind::Pyramidal));
		let pyrs = Pyramidal::new(pyrs_dims, region, &axns, &aux, ocl);

		let mcols_layer = region.col_input_layer().expect("CorticalArea::new()");
		let mcols_dims = dims.clone_with_depth(mcols_layer.depth());
		let mcols = MiniColumns::new(mcols_dims, region, &axns, &pyrs, &aux, ocl);
		

		let mut cortical_area = CorticalArea {
			name: name,
			dims: dims,
			//depth_axonal: depth_axonal,
			//depth_cellular: depth_cellular,
			slice_map: region.slice_map(),
			//region: region,
			axns: axns,
			mcols: mcols,
			pyrs: pyrs,
			//soma: Somata::new(width, depth_cellular, region, ocl),
			aux: aux,
			ocl: ocl.clone(),
			counter: 0,
		};

		cortical_area.init_kernels();

		cortical_area
	}

	pub fn init_kernels(&mut self) {
		//self.axns.init_kernels(&self.mcols.asps, &self.mcols, &self.aux)
		//self.mcols.dens.syns.init_kernels(&self.axns, ocl);
		self.pyrs.init_kernels(&self.mcols, &self.axns, &self.aux);
	}

	pub fn cycle(&mut self, region: &ProtoRegion) {
		//self.soma.dst_dens.cycle(&self.axns, &self.ocl);
		//self.soma.cycle(&self.ocl);
		//self.soma.inhib(&self.ocl);
		//self.axns.cycle(&self.soma, &self.ocl);
		//self.soma.ltp(&self.ocl);
		//self.soma.dst_dens.syns.decay(&mut self.soma.rand_ofs, &self.ocl);

		let ltp: bool = cmn::LEARNING_ACTIVE;
		
		self.mcols.cycle(ltp);
		
		self.pyrs.activate(ltp);
		
		self.pyrs.cycle();	

		self.mcols.output();

		self.regrow(region);

	}

	pub fn regrow(&mut self, region: &ProtoRegion) {
		if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
			//print!("$");
			self.mcols.regrow(region);
			self.pyrs.regrow(region);
			self.counter = 0;
		} else {
			self.counter += 1;
		}
	}
}


pub struct Aux {
	dims: CorticalDimensions,
	pub ints_0: Envoy<ocl::cl_int>,
	pub ints_1: Envoy<ocl::cl_int>,
	pub chars_0: Envoy<ocl::cl_char>,
	pub chars_1: Envoy<ocl::cl_char>,
}

impl Aux {
	pub fn new(mut dims: CorticalDimensions, ocl: &Ocl) -> Aux {

		//let dims_multiplier: u32 = 512;

		//dims.columns() *= 512;

		Aux { 
			ints_0: Envoy::<ocl::cl_int>::new(dims, 0, ocl),
			ints_1: Envoy::<ocl::cl_int>::new(dims, 0, ocl),
			chars_0: Envoy::<ocl::cl_char>::new(dims, 0, ocl),
			chars_1: Envoy::<ocl::cl_char>::new(dims, 0, ocl),
			dims: dims,
		}
	}
}



/*pub struct Somata {
	depth: u8,
	dims: CorticalDimensions, height: u32, 
	pub dst_dens: Dendrites,
	pub states: Envoy<ocl::cl_uchar>,
	pub hcol_max_vals: Envoy<ocl::cl_uchar>,
	pub hcol_max_ids: Envoy<ocl::cl_uchar>,
	pub rand_ofs: Envoy<ocl::cl_char>,
}

impl Somata {
	pub fn new(dims: CorticalDimensions, height: u32,  depth: u8, region: &ProtoRegion, ocl: &Ocl) -> Somata {
		Somata { 
			depth: depth,
			width: width, height: height, 
			states: Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl),
			hcol_max_vals: Envoy::<ocl::cl_uchar>::new(dims.width / cmn::COLUMNS_PER_HYPERCOLUMN, depth, cmn::STATE_ZERO, ocl),
			hcol_max_ids: Envoy::<ocl::cl_uchar>::new(dims.width / cmn::COLUMNS_PER_HYPERCOLUMN, depth, 0u8, ocl),
			rand_ofs: Envoy::<ocl::cl_char>::shuffled(256, 1, -128, 127, ocl),
			dst_dens: Dendrites::new(width, depth, DendriteKind::Distal, cmn::DENDRITES_PER_CELL_DISTAL, region, ocl),

		}
	}

	fn cycle_pre(&self, dst_dens: &Dendrites, prx_dens: &Dendrites, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_cycle_pre");
		ocl::set_kernel_arg(1, prx_dens.states.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);

		let gws = (self.depth as usize, self.dims.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	}

	fn cycle(&self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_cycle_post");
		ocl::set_kernel_arg(0, self.dst_dens.states.buf, kern);
		//ocl::set_kernel_arg(1, self.bsl_prx_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.states.buf, kern);
		ocl::set_kernel_arg(2, self.depth as u32, kern);

		let gws = (self.depth as usize, self.dims.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	}

	pub fn inhib(&self, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "soma_inhib");
		ocl::set_kernel_arg(0, self.states.buf, kern);
		ocl::set_kernel_arg(1, self.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(2, self.hcol_max_vals.buf, kern);
		let mut kern_dims.width = self.dims.width as usize / cmn::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.depth as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

		ocl::set_kernel_arg(0, self.aux.chars_0.buf, kern);
		ocl::set_kernel_arg(1, self.aux.chars_1.buf, kern);
		kern_dims.width = kern_dims.width / (1 << grp_size_log2);
		let gws = (self.depth_cellular as usize, self.dims.width as usize / 64);
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

		let mut kern_dims.width = self.dims.width as usize / cmn::COLUMNS_PER_HYPERCOLUMN as usize;
		let gws = (self.depth as usize, kern_width);
		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);
	}
}*/




