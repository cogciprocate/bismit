// use num;
// use rand;
// use std::mem;
// use std::iter; // TEMPORARY
//use rand::distributions::{ Normal, IndependentSample, Range };
// use rand::{ ThreadRng };
// use num::{ Integer };
// use std::default::{ Default };
// use std::fmt::{ Display };
use std::collections::{ /*BTreeMap,*/ HashMap };
use std::ops::{ Range };
use rand;

use cmn::{ self, CorticalDimensions, Renderer, Sdr, DataCellLayer };
use map::{ AreaMap };
use ocl::{ self, OclProgQueue, OclContext, /*WorkSize,*/ Envoy, /*BuildOptions,*/ /*BuildOption*/ };
use proto::{ /*ProtoLayerMap, ProtoLayerMaps, ProtoAreaMaps, ProtoAreaMap,*/ Cellular, /*Axonal, Spatial, Horizontal, Sensory,*/ Pyramidal, SpinyStellate, Inhibitory, layer, /*Protocell,*/ DendriteKind };

// use synapses::{ Synapses };
// use dendrites::{ Dendrites };
use axon_space::{ AxonSpace };
use minicolumns::{ Minicolumns };
use iinn::{ InhibitoryInterneuronNetwork };
use pyramidals::{ PyramidalLayer };
use spiny_stellates::{ SpinyStellateLayer };
use sensory_filter::{ SensoryFilter };

#[cfg(test)]
pub use self::tests::{ CorticalAreaTest };


pub type CorticalAreas = HashMap<&'static str, Box<CorticalArea>>;


pub struct CorticalArea {
	pub name: &'static str,
	pub dims: CorticalDimensions,
	area_map: AreaMap,
	pub axns: AxonSpace,
	pub mcols: Box<Minicolumns>,
	pub pyrs_map: HashMap<&'static str, Box<PyramidalLayer>>,		// MAKE ME PRIVATE -- FIX tests::hybrid
	pub ssts_map: HashMap<&'static str, Box<SpinyStellateLayer>>,	// MAKE ME PRIVATE -- FIX tests::hybrid
	pub iinns: HashMap<&'static str, Box<InhibitoryInterneuronNetwork>>,	// MAKE ME PRIVATE -- FIX tests::hybrid
	pub filters: Option<Vec<Box<SensoryFilter>>>,
	ptal_name: &'static str,	// PRIMARY TEMPORAL ASSOCIATIVE LAYER NAME
	psal_name: &'static str,	// PRIMARY SPATIAL ASSOCIATIVE LAYER NAME
	pub aux: Aux,
	ocl: OclProgQueue,
	ocl_context: OclContext,
	renderer: Renderer,
	counter: usize,
	rng: rand::XorShiftRng,
	pub bypass_inhib: bool,
	pub bypass_filters: bool,
	pub disable_pyrs: bool,
	pub disable_ssts: bool,
	pub disable_mcols: bool,
	pub disable_regrowth: bool,
	pub disable_learning: bool,
}

impl CorticalArea {
	pub fn new(area_map: AreaMap, device_idx: usize) -> CorticalArea {
		let emsg = "cortical_area::CorticalArea::new()";

		println!("\n\nCORTICALAREA::NEW(): Creating Cortical Area: '{}'...", area_map.proto_area_map().name);		

		let ocl_context: ocl::OclContext = OclContext::new(None);
		let mut ocl: ocl::OclProgQueue = ocl::OclProgQueue::new(&ocl_context, Some(device_idx));

		ocl.build(area_map.gen_build_options());

		// let dims = area_map.proto_area_map().dims.clone_with_depth(area_map.proto_layer_map().depth_total())
		// 	.with_physical_increment(ocl.get_max_work_group_size());

		let dims = area_map.dims().clone_with_physical_increment(ocl.get_max_work_group_size());

		println!("{}CORTICALAREA::NEW(): Area '{}' details: \
			(u_size: {}, v_size: {}, depth: {}), eff_areas: {:?}, aff_areas: {:?}", 
			cmn::MT, area_map.proto_area_map().name, dims.u_size(), dims.v_size(), dims.depth(), 
			area_map.proto_area_map().eff_areas, area_map.proto_area_map().aff_areas);

		let emsg_psal = format!("{}: Primary Spatial Associative Layer not defined.", emsg);
		let psal_name = area_map.proto_layer_map().layer_with_flag(layer::SPATIAL_ASSOCIATIVE).expect(&emsg_psal).name();

		let emsg_ptal = format!("{}: Primary Temporal Associative Layer not defined.", emsg);
		let ptal_name = area_map.proto_layer_map().layer_with_flag(layer::TEMPORAL_ASSOCIATIVE).expect(&emsg_ptal).name();
		

			/* <<<<< BRING BACK UPDATED VERSIONS OF BELOW >>>>> */
		//assert!(SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2 >= 2);
		//assert!(SYNAPSES_PER_DENDRITE_DISTAL_LOG2 >= 2);
		//assert!(DENDRITES_PER_CELL_DISTAL_LOG2 <= 8);
		//assert!(DENDRITES_PER_CELL_DISTAL <= 256);
		//assert!(DENDRITES_PER_CELL_PROXIMAL_LOG2 == 0);
		//assert!(depth_cellular > 0, "cortical_area::CorticalArea::new(): Region has no cellular layers.");

		let axns = AxonSpace::new(&area_map, &ocl);

		let mut pyrs_map = HashMap::new();
		let mut ssts_map = HashMap::new();
		let mut iinns = HashMap::new();


		/*=============================================================================
		================================== DATA CELLS =================================
		=============================================================================*/
		// BREAK OFF THIS CODE INTO NEW STRUCT DEF


		for (&layer_name, layer) in area_map.proto_layer_map().layers().iter() {
			match layer.kind {
				Cellular(ref pcell) => {
					println!("   CORTICALAREA::NEW(): making a(n) {:?} layer: '{}' (depth: {})", 
						pcell.cell_kind, layer_name, layer.depth);

					match pcell.cell_kind {
						Pyramidal => {
							let pyrs_dims = dims.clone_with_depth(layer.depth);

							let pyr_lyr = PyramidalLayer::new(
								layer_name, pyrs_dims, pcell.clone(), &area_map, &axns, /*&aux,*/ &ocl);

							pyrs_map.insert(layer_name, Box::new(pyr_lyr));
						},

						SpinyStellate => {							
							let ssts_map_dims = dims.clone_with_depth(layer.depth);
							let sst_lyr = SpinyStellateLayer::new(
								layer_name, ssts_map_dims, pcell.clone(), &area_map, &axns, /*&aux,*/ &ocl);
							ssts_map.insert(layer_name, Box::new(sst_lyr));
						},

						_ => (),
					}
				},

				_ => println!("   CORTICALAREA::NEW(): Axon layer: '{}' (depth: {})", layer_name, layer.depth),
			}
		}


		/*=============================================================================
		================================ CONTROL CELLS ================================
		=============================================================================*/
		// BREAK OFF THIS CODE INTO NEW STRUCT DEF


		for (&layer_name, layer) in area_map.proto_layer_map().layers().iter() {
			match layer.kind {
				Cellular(ref pcell) => {
					match pcell.cell_kind {
						Inhibitory => {
							let src_lyr_names = layer.src_lyr_names(DendriteKind::Distal);							
							assert!(src_lyr_names.len() == 1);

							let src_lyr_name = src_lyr_names[0];
							let src_slc_ids = area_map.proto_layer_map().slc_ids(vec![src_lyr_name]);
							let src_layer_depth = src_slc_ids.len() as u8;
							let src_base_axn_slc = src_slc_ids[0];

							println!("   CORTICALAREA::NEW(): Inhibitory cells: src_lyr_names: \
								{:?}, src_base_axn_slc: {:?}", src_lyr_names, src_base_axn_slc);

							let em1 = format!("{}: '{}' is not a valid layer", emsg, src_lyr_name);
							let src_soma_env = &ssts_map.get_mut(src_lyr_name).expect(&em1).soma();
						
							let iinns_dims = dims.clone_with_depth(src_layer_depth);
							let iinn_lyr = InhibitoryInterneuronNetwork::new(layer_name, iinns_dims, 
								pcell.clone(), &area_map, src_soma_env, 
								src_base_axn_slc, &axns, /*&aux,*/ &ocl);

							iinns.insert(layer_name, Box::new(iinn_lyr));

						},

						_ => (),
					}
				},

				_ => (),
			}
		}


		let mcols_dims = dims.clone_with_depth(1);
		
		// <<<<< EVENTUALLY ADD TO CONTROL CELLS (+PROTOCONTROLCELLS) >>>>>
		let mcols = Box::new({
			//let em_ssts = emsg.to_string() + ": ssts - em2".to_string();
			let em_ssts = format!("{}: '{}' is not a valid layer", emsg, psal_name);
			let ssts = ssts_map.get(psal_name).expect(&em_ssts);

			let em_pyrs = format!("{}: '{}' is not a valid layer", emsg, ptal_name);
			let pyrs = pyrs_map.get(ptal_name).expect(&em_pyrs);
			Minicolumns::new(mcols_dims, &area_map, &axns, ssts, pyrs, /*&aux,*/ &ocl)
		});


		/*=============================================================================
		=================================== FILTERS ===================================
		=============================================================================*/
		// BREAK OFF THIS CODE INTO NEW STRUCT DEF

		// <<<<< CHANGE TO LAYER**S**_WITH_FLAG() >>>>>
		let filters = {
			//let aff_in_layer = area_map.proto_layer_map().layer_with_flag(layer::AFFERENT_INPUT).expect(&emsg);
			//let base_axn_slc = aff_in_layer.base_slc();
			let mut filters_vec = Vec::with_capacity(5);

			match area_map.proto_area_map().filters {
				Some(ref protofilters) => {
					for pf in protofilters.iter() {
						filters_vec.push(Box::new(SensoryFilter::new(
							pf.filter_name(), 
							pf.cl_file_name(), 
							&area_map,
							//area_map.proto_area_map().name,
							//dims.clone_with_depth(aff_in_layer.depth()), 
							&axns, 
							//aff_in_layer.base_slc(), 
							&ocl
						)));
					}
					Some(filters_vec)
				},
				None => None,
			}
		};

		let renderer = Renderer::new(&dims);

		let aux = Aux::new(pyrs_map[ptal_name].dens().syns().dims(), &ocl);

		pyrs_map.get_mut(ptal_name).unwrap().dens_mut().syns_mut()
			.set_arg_env_named("aux_ints_0", &aux.ints_0);

		// TODO: give aux to whoever

		let cortical_area = CorticalArea {
			name: area_map.proto_area_map().name,
			dims: dims,
			area_map: area_map,
			ptal_name: ptal_name,
			psal_name: psal_name,
			axns: axns,
			mcols: mcols,
			pyrs_map: pyrs_map,
			ssts_map: ssts_map,
			iinns: iinns,
			filters: filters,
			aux: aux,
			ocl: ocl,
			ocl_context: ocl_context,
			renderer: renderer,
			counter: 0,
			rng: rand::weak_rng(),
			bypass_inhib: false,
			bypass_filters: false,
			disable_pyrs: false,
			disable_ssts: false,
			disable_mcols: false,
			disable_regrowth: false,
			disable_learning: false,
		};

		cortical_area
	}


	// CYCLE(): <<<<< TODO: ISOLATE LEARNING INTO SEPARATE THREAD >>>>>
	pub fn cycle(&mut self) /*-> (&Vec<&'static str>, &Vec<&'static str>)*/ {
		let emsg = format!("cortical_area::CorticalArea::cycle(): Invalid layer.");

		if !self.disable_ssts {	self.psal_mut().cycle(); }

		self.iinns.get_mut("iv_inhib").expect(&emsg).cycle(self.bypass_inhib);

		if !self.disable_ssts {	if !self.disable_learning { self.psal_mut().learn(); } }

		if !self.disable_mcols { self.mcols.activate(); }
		
		if !self.disable_pyrs {
			if !self.disable_learning { self.ptal_mut().learn(); }
			self.ptal_mut().cycle();
		}

		if !self.disable_mcols { self.mcols.output(); }

		//if !self.disable_regrowth { self.regrow(); } // BEING CALLED DIRECTLY FROM CORTEXs

		/*(self.afferent_target_names(), self.efferent_target_names())*/
	}

	pub fn regrow(&mut self) {
		if !self.disable_regrowth { 
			if self.counter >= cmn::SYNAPSE_REGROWTH_INTERVAL {
				//print!("$");
				self.ssts_map.get_mut(self.psal_name).expect("cortical_area.rs").regrow();
				self.ptal_mut().regrow();
				self.counter = 0;
			} else {
				self.counter += 1;
			}
		}
	}


	/* AXN_OUTPUT(): NEEDS UPDATING (DEPRICATION?) */
	pub fn axn_output_range(&self) -> Range<u32> {
		self.area_map.axn_range_by_flag(layer::AFFERENT_OUTPUT)
	}

	/* LAYER_INPUT_RANGES(): NEEDS UPDATE / REMOVAL */
	pub fn layer_input_ranges(&self, layer_name: &'static str, den_kind: &DendriteKind) -> Vec<Range<u32>> {
		let mut axn_irs: Vec<Range<u32>> = Vec::with_capacity(10);
		let src_slc_ids = self.area_map.proto_layer_map().src_slc_ids(layer_name, *den_kind);

		for ssid in src_slc_ids {
			//let idz = cmn::axn_idz_2d(ssid, self.dims.columns(), self.area_map.hrz_demarc());
			let idz = self.area_map.axn_idz(ssid);
		 	let idn = idz + self.dims.columns();
			axn_irs.push(idz..idn);
		}

		axn_irs
	}

	pub fn write_input(&self, sdr: &Sdr, layer_flags: layer::ProtolayerFlags) {
		//let tar_slice_size = area_map.

		if layer_flags.contains(layer::AFFERENT_INPUT) && !self.bypass_filters {
			match self.filters {
				Some(ref filters_vec) => {
					filters_vec[0].write(sdr);

					for fltr in filters_vec.iter() { // ***** UN-MUT ME
						fltr.cycle();
					}

					return
				},
				None => (),
			}
		}

		let axn_range = self.area_map.axn_range_by_flag(layer_flags);
		//println!("\nCORTICALAREA::WRITE_INPUT(): axn_range: {:?}", axn_range);
		assert!(sdr.len() == axn_range.len() as usize, format!("\n\
			cortical_area::CorticalArea::write_input()<area: '{}'>: \
			sdr.len(): {} != axn_range.len(): {}", self.name, sdr.len(), axn_range.len()));
		
		self.write_to_axons(axn_range, sdr);

		// // 	TESTING:	
		// let axn_range = self.area_map.axn_range_by_flag(layer_flags);	
		// let mut test_sdr: Vec<u8> = iter::repeat(0).take(axn_range.len()).collect();
		// self.read_from_axons(axn_range, &mut test_sdr[..]);
		// println!("##### AxonSpace: {:?}", test_sdr);
	}

	pub fn read_output(&self, sdr: &mut Sdr, layer_flags: layer::ProtolayerFlags) {
		let axn_range = self.area_map.axn_range_by_flag(layer_flags);

		assert!(sdr.len() == axn_range.len() as usize, format!("\n\
			cortical_area::CorticalArea::read_output()<area: '{}', flags: '{:?}'>: \
			sdr.len(): {} != axn_range.len(): {}", self.name, layer_flags, sdr.len(), axn_range.len()));

		self.read_from_axons(axn_range, sdr);
	}

	// READ_FROM_AXONS(): PUBLIC FOR TESTING/DEBUGGING PURPOSES
	// <<<<< TODO: DEPRICATE IN FAVOR OF ENVOY::WRITE_DIRECT() >>>>>
	fn read_from_axons(&self, axn_range: Range<u32>, sdr: &mut Sdr) {
		assert!((axn_range.end - axn_range.start) as usize == sdr.len());
		ocl::enqueue_read_buffer(sdr, self.axns.states.buf(), self.ocl.queue(), axn_range.start as usize);
	}

	// WRITE_TO_AXONS(): PUBLIC FOR TESTING/DEBUGGING PURPOSES
	// <<<<< TODO: DEPRICATE IN FAVOR OF ENVOY::WRITE_DIRECT() >>>>>
	fn write_to_axons(&self, axn_range: Range<u32>, sdr: &Sdr) {
		assert!((axn_range.end - axn_range.start) as usize == sdr.len());
		ocl::enqueue_write_buffer(sdr, self.axns.states.buf(), self.ocl.queue(), axn_range.start as usize);
	}

	pub fn write_to_axon_slice(&self, slc_id: u8, sdr: &Sdr) {
		// check str.len() == slice cols (areamap should have this info)
		// 

	}


	pub fn mcols(&self) -> &Box<Minicolumns> {
		&self.mcols
	}


	/* PIL(): Get Primary Spatial Associative Layer (immutable) */
	pub fn psal(&self) -> &Box<SpinyStellateLayer> {
		let e_string = "cortical_area::CorticalArea::psal(): Primary Spatial Associative Layer: '{}' not found. ";
		self.ssts_map.get(self.psal_name).expect(e_string)
	}

	/* PIL_MUT(): Get Primary Spatial Associative Layer (mutable) */
	pub fn psal_mut(&mut self) -> &mut Box<SpinyStellateLayer> {
		let e_string = "cortical_area::CorticalArea::psal_mut(): Primary Spatial Associative Layer: '{}' not found. ";
		self.ssts_map.get_mut(self.psal_name).expect(e_string)
	}


	/* PAL(): Get Primary Temporal Associative Layer (immutable) */
	pub fn ptal(&self) -> &Box<PyramidalLayer> {
		let e_string = "cortical_area::CorticalArea::ptal(): Primary Temporal Associative Layer: '{}' not found. ";
		self.pyrs_map.get(self.ptal_name).expect(e_string)
	}

	/* PAL_MUT(): Get Primary Temporal Associative Layer (mutable) */
	pub fn ptal_mut(&mut self) -> &mut Box<PyramidalLayer> {
		let e_string = "cortical_area::CorticalArea::ptal_mut(): Primary Temporal Associative Layer: '{}' not found. ";
		self.pyrs_map.get_mut(self.ptal_name).expect(e_string)
	}


	// <<<<< TODO: DEPRICATE >>>>>
	// pub fn protolayer_map(&self) -> &ProtoLayerMap {
	// 	&self.area_map.proto_layer_map()
	// }

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

	pub fn psal_name(&self) -> &'static str {
		self.psal_name
	}

	pub fn ptal_name(&self) -> &'static str {
		self.ptal_name
	}

	pub fn afferent_target_names(&self) -> &Vec<&'static str> {
		//self.area_map.proto_area_map().aff_areas.clone
		&self.area_map.proto_area_map().aff_areas
	}

	pub fn efferent_target_names(&self) -> &Vec<&'static str> {
		//self.area_map.proto_area_map().eff_areas.clone()
		&self.area_map.proto_area_map().eff_areas
	}

	pub fn ocl(&self) -> &OclProgQueue {
		&self.ocl
	}

	pub fn render_aff_out(&mut self, input_status: &str, print_summary: bool) {
		let out_axns = &self.axns.states.vec()[self.mcols.aff_out_axn_range()];
		let sst_axns = &self.axns.states.vec()[self.psal().axn_range()];
		self.renderer.render(out_axns, Some(sst_axns), None, input_status, print_summary);
	}

	pub fn render_axon_space(&mut self) {
		let axn_states = &self.axns.states.vec()[..];
		let slc_map = &self.area_map.proto_layer_map().slc_map();
		let cols = self.dims.columns();
		let hrz_demarc = self.area_map.proto_layer_map().hrz_demarc();

		self.renderer.render_axon_space(axn_states, &self.area_map.slices())
	}

	pub fn area_map(&self) -> &AreaMap {
		&self.area_map
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
	// pub chars_0: Envoy<ocl::cl_char>,
	// pub chars_1: Envoy<ocl::cl_char>,
}

impl Aux {
	pub fn new(dims: &CorticalDimensions, ocl: &OclProgQueue) -> Aux {

		//let dims_multiplier: u32 = 512;

		//dims.columns() *= 512;

		let int_32_min = -2147483648;

		Aux { 
			ints_0: Envoy::<ocl::cl_int>::new(dims, int_32_min + 100, ocl),
			ints_1: Envoy::<ocl::cl_int>::new(dims, int_32_min + 100, ocl),
			// chars_0: Envoy::<ocl::cl_char>::new(dims, 0, ocl),
			// chars_1: Envoy::<ocl::cl_char>::new(dims, 0, ocl),
			dims: dims.clone(),
		}
	}

	pub unsafe fn resize(&mut self, new_dims: &CorticalDimensions) {
		let int_32_min = -2147483648;
		self.dims = new_dims.clone();
		self.ints_0.resize(&self.dims, int_32_min + 100);
		self.ints_1.resize(&self.dims, int_32_min + 100);
		// self.chars_0.resize(&self.dims, 0);
		// self.chars_1.resize(&self.dims, 0);
	}
}

impl Drop for CorticalArea {
	fn drop(&mut self) {
    	print!("Releasing OpenCL components for '{}'... ", self.name);
    	self.ocl.release_components();
    	self.ocl_context.release_components();
    	print!(" ...complete. \n");
	}
}



#[cfg(test)]
mod tests {
	use std::ops::{ Range };
	use rand::distributions::{ IndependentSample, Range as RandRange };

	use super::*;
	use axon_space::{ AxonSpaceTest };
	use cmn::{ Sdr, CelCoords };
	// use synapses::{ SynCoords };
	use map::{ AreaMapTest };

	pub trait CorticalAreaTest {
		//fn axons
		fn read_from_axons(&self, sdr: &mut Sdr, axn_range: Range<u32>);
		fn write_to_axons(&self, sdr: &Sdr, axn_range: Range<u32>);
		fn axn_state(&self, idx: usize) -> u8;
		fn write_to_axon(&self, val: u8, idx: usize);
		fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axn_slc: u8
			) -> (i8, i8, u32);
		fn print_aux(&mut self);
		fn print_axns(&mut self);
	}

	impl CorticalAreaTest for CorticalArea {
		fn read_from_axons(&self, sdr: &mut Sdr, axn_range: Range<u32>) {
			self.read_from_axons(axn_range, sdr);
		}

		fn write_to_axons(&self, sdr: &Sdr, axn_range: Range<u32>) {
			self.write_to_axons(axn_range, sdr);
		}

		fn axn_state(&self, idx: usize) -> u8 {
			self.axns.axn_state(idx)
		}

		fn write_to_axon(&self, val: u8, idx: usize) {
			self.axns.write_to_axon(val, idx);
		}

		fn rand_safe_src_axn(&mut self, cel_coords: &CelCoords, src_axn_slc: u8) -> (i8, i8, u32) {
			let v_ofs_range = RandRange::new(-8i8, 9);
			let u_ofs_range = RandRange::new(-8i8, 9);

			for i in 0..50 {
				let v_ofs = v_ofs_range.ind_sample(&mut self.rng);
				let u_ofs = u_ofs_range.ind_sample(&mut self.rng);

				if v_ofs | u_ofs == 0 {
					continue;
				}

				let idx_rslt = self.area_map.axn_idx(src_axn_slc, cel_coords.v_id, 
					v_ofs, cel_coords.u_id, u_ofs);

				match idx_rslt {
					Ok(idx) => return (v_ofs, u_ofs, idx),
					Err(_) => (),
				}
			}

			panic!("SynCoords::rand_safe_src_axn_offs(): Error finding valid offset pair.");
		}

		fn print_aux(&mut self) {
			print!("aux.ints_0: ");
			let view_radius = 1 << 24;
			self.aux.ints_0.print((1 << 0) as usize, 
				Some((0 - view_radius, view_radius)), None, true);
			
			print!("aux.ints_1: ");
			self.aux.ints_1.print((1 << 0) as usize, 
				Some((0 - view_radius, view_radius)), None, true);
		}

		fn print_axns(&mut self) {
			print!("axns: ");
			//let view_radius = 1 << 24;
			self.axns.states.print(1 << 0, Some((0, 255)), None, false);
		}
	}
}






	// pub fn init_kernels(&mut self) {
	// 	//self.axns.init_kernels(&self.mcols.asps, &self.mcols, &self.aux)
	// 	//self.mcols.dens.syns().init_kernels(&self.axns, ocl);

	// 	let emsg = "cortical_area::CorticalArea::init_kernels(): Invalid Primary Spatial Associative Layer Name.";

	// 	self.pyrs_map.get_mut(self.ptal_name).expect(emsg).init_kernels(&self.mcols, &self.ssts_map.get_mut(self.psal_name).expect("emsg"), &self.axns, &self.aux);


	// 	/*
	// 	let layer_name_iv = self.psal_name;
	// 	let slc_ids_iv = self.protolayer_map.slc_ids(vec![layer_name_iv]);
	// 	let layer_depth_iv = slc_ids_iv.len() as u8;
	// 	let base_slc_iv = slc_ids_iv[0];
	// 	let soma_envoy_iv = &self.ssts_map.get_mut(layer_name_iv).expect("cortical_area.rs").soma();
	// 	self.iinns.get_mut("iv_inhib").expect("cortical_area.rs").init_kernels(soma_envoy_iv, base_slc_iv, layer_depth_iv);
	// 	*/
	// }


	// pub fn axn_output_range(&self) -> Range<u32> {
	// 	//println!("self.axn_aff_out_slc: {}, self.dims.columns(): {}, cmn::AXON_MAR__GIN_SIZE: {}", self.axn_aff_out_slc as usize, self.dims.columns() as usize, cmn::AXON_MAR__GIN_SIZE);
	// 	// let output_slcs = self.area_map.proto_layer_map().aff_out_slcs();
	// 	// assert!(output_slcs.len() == 1, "\nCorticalArea: ERROR: Too many afferent output layers defined.");
	// 	// let axn_aff_out_slc = output_slcs[0];

	// 	// //let start = (axn_aff_out_slc as usize * self.dims.columns() as usize) + cmn::AXON_MAR__GIN_SIZE as usize;
	// 	// //let start = cmn::axn_idz_2d(axn_aff_out_slc, self.dims.columns(), self.area_map.hrz_demarc());
	// 	// let idz = self.area_map.axn_idz(axn_aff_out_slc);
	// 	// (idz, idz + (self.dims.per_slc()))
	// 	self.area_map.axn_range_by_flag(layer::AFFERENT_OUTPUT)
	// }


	// pub fn write_to_layer(&self, layer_target: &'static str, sdr: &Sdr) {
	// 	let emsg = format!("cortex::Cortex::write_vec(): Invalid area name: {}", area_name);
	// 	//let area = areas.get(area_name).expect(&emsg);

	// 	//let ref region = self.protolayer_maps[&RegionKind::Sensory];
	// 	let region = this.area_map.proto_layer_map();
	// 	let axn_slcs: Vec<ocl::cl_uchar> = region.slc_ids(vec!(layer_target));
		
	// 	for slc in axn_slcs { 
	// 		//let buffer_offset = cmn::axn_idz_2d(slc, area.dims.columns(), region.hrz_demarc()) as usize;
	// 		let buffer_offset = self.area_map.axn_idz(slc);
	// 		ocl::enqueue_write_buffer(sdr, area.axns.states.buf, area.ocl().queue(), buffer_offset);
	// 	}
	// }


	// pub fn axn_range(&self, layer_flags: layer::ProtolayerFlags) -> Range<u32> {
	// 	let emsg = format!("\ncortical_area::CorticalArea::axn_range(): \
	// 		'{:?}' flag not set for any layer in area: '{}'.", layer_flags, self.name);
	// 	let layer = self.area_map.proto_layer_map().layer_with_flag(layer_flags).expect(&emsg); // CHANGE TO LAYERS_WITH_FLAG()
	// 	let len = self.dims.columns() * layer.depth as u32;
	// 	let base_slc = layer.base_slc_id;
	// 	//let buffer_offset = cmn::axn_idz_2d(base_slc, self.dims.columns(), self.area_map.proto_layer_map().hrz_demarc());
	// 	let idz = self.area_map.axn_idz(base_slc);

	// 	idz..(idz + len)
	// }

	// 	INPUT_SRC_AREAS():
	//  <<<<< TODO: DEPRICATE OR WRAP AREA_MAP::INPUT_SRC_AREA_NAMES() >>>>>
	// 		- REMINDER: AFFERENT INPUT COMES FROM EFFERENT AREAS, EFFERENT INPUT COMES FROM AFFERENT AREAS
	// pub fn input_src_area_names(&self, layer_flags: layer::ProtolayerFlags) -> Vec<&'static str> {
	// 	 // let emsg = format!("\ncortical_area::CorticalArea::axn_range(): \
	// 	 // 	'{:?}' flag not set for any layer in area: '{}'.", layer_flags, self.name);
	// 	// let layer = self.area_map.proto_layer_map().layer_with_flag(layer_flags);
	// 	// return layer.expect(&emsg).depth();

	// 	// 
	// 	if layer_flags == layer::EFFERENT_INPUT {
	// 		self.area_map.proto_area_map().aff_areas.clone()
	// 	} else if layer_flags == layer::AFFERENT_INPUT {
	// 		self.area_map.proto_area_map().eff_areas.clone()
	// 	} else {
	// 		panic!("\nCorticalArea::input_src_areas(): Can only be called with an \
	// 			input layer flag as argument");
	// 	}		
	// }
