use num;
use rand;
use std::mem;
//use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };
use std::collections::{ BTreeMap, HashMap };
use std::ops::{ Range };

use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ Protoareas, Protoarea };
use proto::regions::{ Protoregion, ProtoregionKind };
use proto::layer::{ Protolayer, ProtolayerKind };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use minicolumns::{ MiniColumns };
use peak_column::{ PeakColumns };
use pyramidals::{ Pyramidals };
//use spiny_stellates::{ SpinyStellates };




pub struct CorticalArea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	protoregion: Protoregion,
	protoarea: Protoarea,
	//pub depth_axonal: u8,
	//pub depth_cellular: u8,
	pub slice_map: BTreeMap<u8, &'static str>,
	//pub protoregion: Protoregion,
	pub axns: Axons,
	pub mcols: MiniColumns,
	pub pyrs: Pyramidals,
	//pub layer_cells: HashMap<&'static str, LayerCells>,
	//pub soma: Somata,
	pub aux: Aux,
	ocl: ocl::Ocl,
	counter: usize,
}

impl CorticalArea {
	pub fn new(name: &'static str, protoregion: &Protoregion, protoarea: &Protoarea, ocl: &Ocl) -> CorticalArea {
		//let (depth_axonal, depth_cellular) = protoregion.depth();
		let dims = protoarea.dims.clone_with_depth(protoregion.depth_total());
		//let dims.width = areas.width(&protoregion.kind);
		//let height = areas.height(&protoregion.kind);

		//print!("\nCorticalArea::new(): depth_axonal: {}, depth_cellular: {}, width: {}", depth_axonal, depth_cellular, width);

		//assert!(depth_cellular > 0, "cortical_area::CorticalArea::new(): Region has no cellular layers.");

		let axns = Axons::new(dims, protoregion, ocl);

		let aux_dims = CorticalDimensions::new(dims.width_l2() + 3, dims.height_l2() + 3, dims.depth(), 0);
		let aux = Aux::new(aux_dims, ocl);

		//let layer_cells: HashMap<&'static str, LayerCells> = HashMap::new();


		for (&layer_name, layer) in protoregion.layers().iter() {
			match layer.kind {
				ProtolayerKind::Cellular(ref cell) => {
					match cell.cell_kind {
						ProtocellKind::Pyramidal => {
							print!("\n##### Making a Pyramidal ({:?}) for layer: {} (depth: {})", cell.cell_kind, layer_name, layer.depth);

							() // make a pyramidal
						},

						ProtocellKind::SpinyStellate => {
							print!("\n##### Making a SpinyStellate ({:?}) for layer: {} (depth: {})", cell.cell_kind, layer_name, layer.depth);
							() // " 
						},

						_ => (),
					}
				},

				_ => print!("\n##### Skipping over layer: {}", layer_name),
			}
		}

					/*if cell.cell_kind == self.cell_kind {
						region.src_slice_ids(layer_name, self.den_kind)
					} else {
						continue
					}*/


		let pyrs_dims = dims.clone_with_depth(protoregion.depth_cell_kind(&ProtocellKind::Pyramidal));
		let pyrs = Pyramidals::new(pyrs_dims, protoregion, &axns, &aux, ocl);

		let mcols_layer = protoregion.col_input_layer().expect("CorticalArea::new()");
		let mcols_dims = dims.clone_with_depth(mcols_layer.depth());
		let mcols = MiniColumns::new(mcols_dims, protoregion, &axns, &pyrs, &aux, ocl);
		

		let mut cortical_area = CorticalArea {
			name: name,
			dims: dims,
			protoregion: protoregion.clone(),
			protoarea: protoarea.clone(),
			//depth_axonal: depth_axonal,
			//depth_cellular: depth_cellular,
			slice_map: protoregion.slice_map(),
			//protoregion: protoregion,
			axns: axns,
			mcols: mcols,
			pyrs: pyrs,
			//layer_cells: layer_cells,
			//soma: Somata::new(width, depth_cellular, protoregion, ocl),
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

	pub fn cycle(&mut self, protoregion: &Protoregion) {
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

		self.regrow(protoregion);

	}

	pub fn regrow(&mut self, protoregion: &Protoregion) {
		if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
			//print!("$");
			self.mcols.regrow(protoregion);
			self.pyrs.regrow(protoregion);
			self.counter = 0;
		} else {
			self.counter += 1;
		}
	}

	pub fn axn_output_range(&self) -> (usize, usize) {
		//println!("self.axn_output_slice: {}, self.dims.columns(): {}, cmn::SYNAPSE_REACH_LIN: {}", self.axn_output_slice as usize, self.dims.columns() as usize, cmn::SYNAPSE_REACH_LIN);
		let output_slices = self.protoregion.col_output_slices();
		assert!(output_slices.len() == 1);
		let axn_output_slice = output_slices[0];

		let start = (axn_output_slice as usize * self.dims.columns() as usize) + cmn::SYNAPSE_REACH_LIN as usize;
		(start, start + (self.dims.per_slice()) as usize)
	}

	pub fn layer_input_ranges(&self, layer_name: &'static str, den_kind: &DendriteKind) -> Vec<Range<u32>> {
		let mut axn_irs: Vec<Range<u32>> = Vec::with_capacity(10);
		let src_slice_ids = self.protoregion.src_slice_ids(layer_name, *den_kind);

		for ssid in src_slice_ids {
			let idz = cmn::axn_idx_2d(ssid, self.dims.columns(), self.protoregion.hrz_demarc());
		 	let idn = idz + self.dims.columns();
			axn_irs.push(idz..idn);
		}

		axn_irs
	}

	pub fn write_to_axons(&mut self, axn_range: Range<u32>, vec: &Vec<ocl::cl_uchar>) {
		assert!((axn_range.end - axn_range.start) as usize == vec.len());
		ocl::enqueue_write_buffer(&vec, self.axns.states.buf, self.ocl.command_queue, axn_range.start as usize);
	}
}

/*pub enum LayerCells {
	Pyramidaly(Pyramidals),
	SpinyStellatey(SpinyStellates),
}*/


pub struct AreaParams {
	den_per_cel_distal_l2: u8,
	syn_per_den_distal_l2: u8,

	den_per_cel_proximal: u8,
	syn_per_den_proximal: u8,
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
	pub fn new(dims: CorticalDimensions, height: u32,  depth: u8, protoregion: &Protoregion, ocl: &Ocl) -> Somata {
		Somata { 
			depth: depth,
			width: width, height: height, 
			states: Envoy::<ocl::cl_uchar>::new(width, depth, cmn::STATE_ZERO, ocl),
			hcol_max_vals: Envoy::<ocl::cl_uchar>::new(dims.width / cmn::COLUMNS_PER_HYPERCOLUMN, depth, cmn::STATE_ZERO, ocl),
			hcol_max_ids: Envoy::<ocl::cl_uchar>::new(dims.width / cmn::COLUMNS_PER_HYPERCOLUMN, depth, 0u8, ocl),
			rand_ofs: Envoy::<ocl::cl_char>::shuffled(256, 1, -128, 127, ocl),
			dst_dens: Dendrites::new(width, depth, DendriteKind::Distal, cmn::DENDRITES_PER_CELL_DISTAL, protoregion, ocl),

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




