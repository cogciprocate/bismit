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
use minicolumns::{ Minicolumns };
use peak_column::{ PeakColumns };
use pyramidals::{ PyramidalCellularLayer };
use spiny_stellates::{ SpinyStellateCellularLayer };




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
	pub mcols: Minicolumns,
	//pub pyrs: PyramidalCellularLayer,
	pub pyrs: HashMap<&'static str, Box<PyramidalCellularLayer>>,
	pub ssts: HashMap<&'static str, Box<SpinyStellateCellularLayer>>,
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

			/* <<<<< BRING BACK >>>>> */
		//assert!(SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 >= 2);
		//assert!(SYNAPSES_PER_DENDRITE_DISTAL_LOG2 >= 2);
		//assert!(DENDRITES_PER_CELL_DISTAL_LOG2 <= 8);
		//assert!(DENDRITES_PER_CELL_DISTAL <= 256);
		//assert!(DENDRITES_PER_CELL_PROXIMAL_LOG2 == 0);

		//print!("\nCorticalArea::new(): depth_axonal: {}, depth_cellular: {}, width: {}", depth_axonal, depth_cellular, width);

		//assert!(depth_cellular > 0, "cortical_area::CorticalArea::new(): Region has no cellular layers.");

		let axns = Axons::new(dims, protoregion, ocl);

		let aux_dims = CorticalDimensions::new(dims.width_l2() + 1, dims.height_l2() + 1, dims.depth(), 0);
		let aux = Aux::new(aux_dims, ocl);

		let mut pyrs = HashMap::new();
		let mut ssts = HashMap::new();

		for (&layer_name, layer) in protoregion.layers().iter() {
			match layer.kind {
				ProtolayerKind::Cellular(ref pcell) => {
					print!("\nCORTICALAREA::NEW(): making a {:?} for layer: {} (depth: {})", pcell.cell_kind, layer_name, layer.depth);

					match pcell.cell_kind {
						ProtocellKind::Pyramidal => {
							let pyrs_dims = dims.clone_with_depth(layer.depth);
							let pyr_lyr = PyramidalCellularLayer::new(layer_name, pyrs_dims, pcell.clone(), protoregion, &axns, &aux, ocl);
							pyrs.insert(layer_name, Box::new(pyr_lyr));
						},

						ProtocellKind::SpinyStellate => {							
							let ssts_dims = dims.clone_with_depth(layer.depth);
							let sst_lyr = SpinyStellateCellularLayer::new(layer_name, ssts_dims, pcell.clone(), protoregion, &axns, &aux, ocl);
							ssts.insert(layer_name, Box::new(sst_lyr));
						},

						_ => (),
					}
				},

				_ => print!("\nCORTICALAREA::NEW(): skipping over axon layer: {}", layer_name),
			}
		}

		let mcols_dims = dims.clone_with_depth(1);
		let mcols = Minicolumns::new(mcols_dims, protoregion, &axns, &ssts, &pyrs, &aux, ocl);
		
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
			//pyrs: pyrs,
			pyrs: pyrs,
			ssts: ssts,
			//layer_cells: layer_cells,
			//soma: Somata::new(width, depth_cellular, protoregion, ocl),
			aux: aux,
			ocl: ocl.clone(),
			counter: 0,
		};

		cortical_area.init_kernels();
		cortical_area
	}


	pub fn cycle(&mut self, protoregion: &Protoregion) {

		let learning = true;
			
		self.ssts.get_mut("iv").expect("cortical_area.rs").cycle(learning);
		
		self.pyrs.get_mut("iii").expect("cortical_area.rs").activate(learning); // *****
		
		self.pyrs.get_mut("iii").expect("cortical_area.rs").cycle();	// *****

		self.mcols.output();

		self.regrow(protoregion);

	}


	pub fn init_kernels(&mut self) {
		//self.axns.init_kernels(&self.mcols.asps, &self.mcols, &self.aux)
		//self.mcols.dens.syns.init_kernels(&self.axns, ocl);
		self.pyrs.get_mut("iii").expect("cortical_area.rs").init_kernels(&self.mcols, &self.ssts.get_mut("iv").expect("cortical_area.rs"), &self.axns, &self.aux);
	}

	pub fn regrow(&mut self, protoregion: &Protoregion) {
		if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
			//print!("$");
			self.ssts.get_mut("iv").expect("cortical_area.rs").regrow(protoregion);
			self.pyrs.get_mut("iii").expect("cortical_area.rs").regrow(protoregion);
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






