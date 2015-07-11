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
use proto::{ Protoregion, Protoregions, Protoareas, ProtoareasTrait, Protoarea, Cellular, Axonal, Spatial, Horizontal, Sensory, Pyramidal, SpinyStellate, Inhibitory, layer, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use axons::{ Axons };
use minicolumns::{ Minicolumns };
use iinn::{ InhibitoryInterneuronNetwork };
use pyramidals::{ PyramidalCellularLayer };
use spiny_stellates::{ SpinyStellateCellularLayer };




pub struct CorticalArea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	ptal_name: &'static str,			// PRIMARY TEMPORAL ASSOCIATIVE LAYER NAME
	psal_name: &'static str,			// PRIMARY SPATIAL ASSOCIATIVE LAYER NAME
	protoregion: Protoregion,
	protoarea: Protoarea,
	//pub depth_axonal: u8,
	//pub depth_cellular: u8,
	//pub slc_map: BTreeMap<u8, &'static str>,
	//pub protoregion: Protoregion,
	pub axns: Axons,
	pub mcols: Box<Minicolumns>,
	//pub pyrs: PyramidalCellularLayer,
	pub pyrs_map: HashMap<&'static str, Box<PyramidalCellularLayer>>,		// MAKE ME PRIVATE -- FIX tests::hybrid
	pub ssts_map: HashMap<&'static str, Box<SpinyStellateCellularLayer>>,	// MAKE ME PRIVATE -- FIX tests::hybrid
	pub iinns: HashMap<&'static str, Box<InhibitoryInterneuronNetwork>>,	// MAKE ME PRIVATE -- FIX tests::hybrid
	//pub soma: Somata,
	pub aux: Aux,
	ocl: ocl::Ocl,
	counter: usize,
}

impl CorticalArea {
	pub fn new(name: &'static str, protoregion: Protoregion, protoarea: Protoarea, ocl: &Ocl) -> CorticalArea {
		let emsg = "cortical_area::CorticalArea::new()";

		let dims = protoarea.dims.clone_with_depth(protoregion.depth_total());


		print!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: '{}' (width: {}, height: {}, depth: {})", 
			name, dims.width(), dims.height(), dims.depth());
		/*print!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: '{}' (width: {}, height: {}, depth: {})", name, 1 << dims.width_l2(), 1 << dims.height_l2(), dims.depth());*/


		let emsg_psal = format!("{}: Primary Spatial Associative Layer not defined.", emsg);
		let psal_name = protoregion.layer_with_flag(layer::SPATIAL_ASSOCIATIVE).expect(&emsg_psal).name();

		let emsg_ptal = format!("{}: Primary Temporal Associative Layer not defined.", emsg);
		let ptal_name = protoregion.layer_with_flag(layer::TEMPORAL_ASSOCIATIVE).expect(&emsg_ptal).name();
		

			/* <<<<< BRING BACK >>>>> */
		//assert!(SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 >= 2);
		//assert!(SYNAPSES_PER_DENDRITE_DISTAL_LOG2 >= 2);
		//assert!(DENDRITES_PER_CELL_DISTAL_LOG2 <= 8);
		//assert!(DENDRITES_PER_CELL_DISTAL <= 256);
		//assert!(DENDRITES_PER_CELL_PROXIMAL_LOG2 == 0);
		//assert!(depth_cellular > 0, "cortical_area::CorticalArea::new(): Region has no cellular layers.");
		//print!("\nCorticalArea::new(): depth_axonal: {}, depth_cellular: {}, width: {}", depth_axonal, depth_cellular, width);

		let axns = Axons::new(dims, &protoregion, ocl);

		let aux_dims = CorticalDimensions::new(dims.width(), dims.height(), dims.depth(), 10, Some(dims.physical_increment()));
		//let aux_dims = CorticalDimensions::new(dims.width_l2(), dims.height_l2(), dims.depth(), 7);
		let aux = Aux::new(aux_dims, ocl);

		let mut pyrs_map = HashMap::new();
		let mut ssts_map = HashMap::new();
		let mut iinns = HashMap::new();

		for (&layer_name, layer) in protoregion.layers().iter() {
			match layer.kind {
				Cellular(ref pcell) => {
					print!("\n   CORTICALAREA::NEW(): making a(n) {:?} layer: '{}' (depth: {})", pcell.cell_kind, layer_name, layer.depth);

					match pcell.cell_kind {
						Pyramidal => {
							let pyrs_dims = dims.clone_with_depth(layer.depth);

							let pyr_lyr = PyramidalCellularLayer::new(
								layer_name, pyrs_dims, pcell.clone(), &protoregion, &axns, &aux, ocl);

							pyrs_map.insert(layer_name, Box::new(pyr_lyr));
						},

						SpinyStellate => {							
							let ssts_map_dims = dims.clone_with_depth(layer.depth);
							let sst_lyr = SpinyStellateCellularLayer::new(
								layer_name, ssts_map_dims, pcell.clone(), &protoregion, &axns, &aux, ocl);
							ssts_map.insert(layer_name, Box::new(sst_lyr));
						},

						_ => (),
					}
				},

				_ => print!("\n   CORTICALAREA::NEW(): Axon layer: '{}' (depth: {})", layer_name, layer.depth),
			}
		}

		for (&layer_name, layer) in protoregion.layers().iter() {
			match layer.kind {
				Cellular(ref pcell) => {
					match pcell.cell_kind {
						Inhibitory => {
							let src_layer_names = layer.src_layer_names(DendriteKind::Distal);
							assert!(src_layer_names.len() == 1);

							let src_layer_name = src_layer_names[0];
							let src_slc_ids = protoregion.slc_ids(vec![src_layer_name]);
							let src_layer_depth = src_slc_ids.len() as u8;
							let src_axn_base_slc = src_slc_ids[0];

							let em1 = format!("{}: '{}' is not a valid layer", emsg, src_layer_name);

							let src_soma_env = &ssts_map.get_mut(src_layer_name).expect(&em1).soma();

						
							let iinns_dims = dims.clone_with_depth(src_layer_depth);
							let mut iinn_lyr = InhibitoryInterneuronNetwork::new(layer_name, iinns_dims, pcell.clone(), &protoregion, src_soma_env, src_axn_base_slc, &axns, &aux, ocl);

							iinns.insert(layer_name, Box::new(iinn_lyr));

						},

						_ => (),
					}
				},

				_ => (),
			}
		}


		let mcols_dims = dims.clone_with_depth(1);
		
		let mcols = Box::new({
			//let em_ssts = emsg.to_string() + ": ssts - em2".to_string();
			let em_ssts = format!("{}: '{}' is not a valid layer", emsg, psal_name);
			let ssts = ssts_map.get(psal_name).expect(&em_ssts);

			let em_pyrs = format!("{}: '{}' is not a valid layer", emsg, ptal_name);
			let pyrs = pyrs_map.get(ptal_name).expect(&em_pyrs);
			Minicolumns::new(mcols_dims, &protoregion, &axns, ssts, pyrs, &aux, ocl)
		});
		

		let mut cortical_area = CorticalArea {
			name: name,
			dims: dims,
			ptal_name: ptal_name,
			psal_name: psal_name,
			protoregion: protoregion,
			protoarea: protoarea,
			//depth_axonal: depth_axonal,
			//depth_cellular: depth_cellular,
			//slc_map: protoregion.slc_map(),
			//protoregion: protoregion,
			axns: axns,
			mcols: mcols,
			pyrs_map: pyrs_map,
			ssts_map: ssts_map,
			iinns: iinns,
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

		let emsg = "cortical_area::CorticalArea::init_kernels(): Invalid Primary Spatial Associative Layer Name.";

		self.pyrs_map.get_mut(self.ptal_name).expect(emsg).init_kernels(&self.mcols, &self.ssts_map.get_mut(self.psal_name).expect("emsg"), &self.axns, &self.aux);


		/*
		let layer_name_iv = self.psal_name;
		let slc_ids_iv = self.protoregion.slc_ids(vec![layer_name_iv]);
		let layer_depth_iv = slc_ids_iv.len() as u8;
		let base_slc_iv = slc_ids_iv[0];
		let soma_envoy_iv = &self.ssts_map.get_mut(layer_name_iv).expect("cortical_area.rs").soma();
		self.iinns.get_mut("iv_inhib").expect("cortical_area.rs").init_kernels(soma_envoy_iv, base_slc_iv, layer_depth_iv);
		*/
	}

	pub fn cycle(&mut self) -> Option<Vec<&'static str>> {
		let emsg = format!("cortical_area::CorticalArea::cycle(): Invalid layer.");


		self.psal_mut().cycle();
		self.iinns.get_mut("iv_inhib").expect(&emsg).cycle();
		self.psal_mut().learn();
		
		self.ptal_mut().activate();
		self.ptal_mut().learn();
		self.ptal_mut().cycle();

		self.mcols.output();

		self.regrow();

		self.afferent_target_names()
	}

	pub fn regrow(&mut self) {
		if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
			//print!("$");
			self.ssts_map.get_mut(self.psal_name).expect("cortical_area.rs").regrow();
			self.ptal_mut().regrow();
			self.counter = 0;
		} else {
			self.counter += 1;
		}
	}


	/* PIL(): Get Primary Sensory Associative Layer (immutable) */
	pub fn psal(&self) -> &Box<SpinyStellateCellularLayer> {
		let e_string = "cortical_area::CorticalArea::psal(): Primary Sensory Associative Layer: '{}' not found. ";
		self.ssts_map.get(self.psal_name).expect(e_string)
	}

	/* PIL_MUT(): Get Primary Sensory Associative Layer (mutable) */
	pub fn psal_mut(&mut self) -> &mut Box<SpinyStellateCellularLayer> {
		let e_string = "cortical_area::CorticalArea::psal_mut(): Primary Sensory Associative Layer: '{}' not found. ";
		self.ssts_map.get_mut(self.psal_name).expect(e_string)
	}


	/* PAL(): Get Primary Temporal Associative Layer (immutable) */
	pub fn ptal(&self) -> &Box<PyramidalCellularLayer> {
		let e_string = "cortical_area::CorticalArea::ptal(): Primary Temporal Associative Layer: '{}' not found. ";
		self.pyrs_map.get(self.ptal_name).expect(e_string)
	}

	/* PAL_MUT(): Get Primary Temporal Associative Layer (mutable) */
	pub fn ptal_mut(&mut self) -> &mut Box<PyramidalCellularLayer> {
		let e_string = "cortical_area::CorticalArea::ptal_mut(): Primary Temporal Associative Layer: '{}' not found. ";
		self.pyrs_map.get_mut(self.ptal_name).expect(e_string)
	}


	/* AXN_OUTPUT(): NEEDS UPDATING (DEPRICATION?) */
	pub fn axn_output_range(&self) -> (usize, usize) {
		//println!("self.axn_output_slc: {}, self.dims.columns(): {}, cmn::SYNAPSE_REACH_LIN: {}", self.axn_output_slc as usize, self.dims.columns() as usize, cmn::SYNAPSE_REACH_LIN);
		let output_slcs = self.protoregion.aff_out_slcs();
		assert!(output_slcs.len() == 1);
		let axn_output_slc = output_slcs[0];

		let start = (axn_output_slc as usize * self.dims.columns() as usize) + cmn::SYNAPSE_REACH_LIN as usize;
		(start, start + (self.dims.per_slc()) as usize)
	}

	/* LAYER_INPUT_RANGES(): NEEDS UPDATE / REMOVAL */
	pub fn layer_input_ranges(&self, layer_name: &'static str, den_kind: &DendriteKind) -> Vec<Range<u32>> {
		let mut axn_irs: Vec<Range<u32>> = Vec::with_capacity(10);
		let src_slc_ids = self.protoregion.src_slc_ids(layer_name, *den_kind);

		for ssid in src_slc_ids {
			let idz = cmn::axn_idx_2d(ssid, self.dims.columns(), self.protoregion.hrz_demarc());
		 	let idn = idz + self.dims.columns();
			axn_irs.push(idz..idn);
		}

		axn_irs
	}

	pub fn write_input(&mut self, sdr: &[ocl::cl_uchar], layer_flags: layer::ProtolayerFlags) {
		let axn_range = self.axn_range(layer_flags);

		assert!(sdr.len() == axn_range.len() as usize, format!("\nsdr.len(): {} != axn_range.len(): {}", sdr.len(), axn_range.len()));

		self.write_to_axons(axn_range, sdr);
	}

	pub fn read_output(&self, sdr: &mut [ocl::cl_uchar], layer_flags: layer::ProtolayerFlags) {
		let axn_range = self.axn_range(layer_flags);

		assert!(sdr.len() == axn_range.len() as usize, format!("\nsdr.len(): {} != axn_range.len(): {}", sdr.len(), axn_range.len()));

		self.read_from_axons(axn_range, sdr);
	}

	fn axn_range(&self, layer_flags: layer::ProtolayerFlags) -> Range<u32> {
		let emsg = format!("cortical_area::CorticalArea::axn_range(): \
			'{:?}' flag not set for any layer in area: '{}'.", layer_flags, self.name);

		let layer = self.protoregion.layer_with_flag(layer_flags).expect(&emsg);
		let len = self.dims.columns() * layer.depth as u32;
		let base_slc = layer.base_slc_pos;
		let buffer_offset = cmn::axn_idx_2d(base_slc, self.dims.columns(), self.protoregion.hrz_demarc());

		buffer_offset..(buffer_offset + len)
	}

	pub fn read_from_axons(&self, axn_range: Range<u32>, sdr: &mut [ocl::cl_uchar]) {
		assert!((axn_range.end - axn_range.start) as usize == sdr.len());
		ocl::enqueue_read_buffer(sdr, self.axns.states.buf, self.ocl.command_queue, axn_range.start as usize);
	}

	pub fn write_to_axons(&mut self, axn_range: Range<u32>, sdr: &[ocl::cl_uchar]) {
		assert!((axn_range.end - axn_range.start) as usize == sdr.len());
		ocl::enqueue_write_buffer(sdr, self.axns.states.buf, self.ocl.command_queue, axn_range.start as usize);
	}

	pub fn protoregion(&self) -> &Protoregion {
		&self.protoregion
	}

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

	pub fn psal_name(&self) -> &'static str {
		self.psal_name
	}

	pub fn ptal_name(&self) -> &'static str {
		self.ptal_name
	}

	pub fn afferent_target_names(&self) -> Option<Vec<&'static str>> {
		self.protoarea.afferent_areas.clone()
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

		let int_32_min = -2147483648;

		Aux { 
			ints_0: Envoy::<ocl::cl_int>::new(dims, int_32_min + 100, ocl),
			ints_1: Envoy::<ocl::cl_int>::new(dims, int_32_min + 100, ocl),
			chars_0: Envoy::<ocl::cl_char>::new(dims, 0, ocl),
			chars_1: Envoy::<ocl::cl_char>::new(dims, 0, ocl),
			dims: dims,
		}
	}
}






