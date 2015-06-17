use std::ptr;
use std::ops::{ Drop };
use std::collections::{ HashMap };
use num;
use time;
use rand::distributions::{ IndependentSample, Range };


use ocl::{ self, Ocl, CorticalDimensions };
use cmn;
use chord::{ Chord };
use cortical_area:: { self, CorticalArea };
use thalamus::{ Thalamus };
use proto::regions::{ self, Protoregion, Protoregions, ProtoregionKind };
use proto::areas::{ self, Protoareas, ProtoareasTrait, Protoarea };
use proto::layer::{ self, Protolayer, ProtolayerKind, ProtoaxonKind };
	//use proto::layer::ProtolayerKind::{ Cellular, Axonal };
	//use proto::layer::ProtoaxonKind::{ Spatial, Horizontal };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind, CellFlags };


	/* Eventually move define_*() to a config file or some such */
pub fn define_protoregions() -> Protoregions {
	let mut cort_regs: Protoregions = Protoregions::new();

	let mut sen = Protoregion::new(ProtoregionKind::Sensory)
		//.layer("test_noise", 1, layer::DEFAULT, Axonal(Spatial))

		.layer("thal", 1, layer::DEFAULT, ProtolayerKind::Axonal(ProtoaxonKind::Spatial))

		.layer("out", 1, layer::AFFERENT_OUTPUT, ProtolayerKind::Axonal(ProtoaxonKind::Spatial))

		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, Protocell::new_spiny_stellate(5, vec!["thal"], 256)) 
		//.layer("vi", 5, layer::DEFAULT, Protocell::new_spiny_stellate(3, vec!["thal"], 256)) 

		.layer("iv_inhib", 0, layer::DEFAULT, Protocell::new_inhibitory(4, "iv"))

		//.layer("iii", 1, layer::DEFAULT, Protocell::new_pyramidal(vec!["iii", "iii", "iii", "iii", "motor"]))
		.layer("iii", 4, layer::TEMPORAL_ASSOCIATIVE, Protocell::new_pyramidal(2, 5, vec!["iii"], 512))

		/*	<<<<< ADDING ADDITIONAL PYRS (AND PRESUMABLY SSTS) NEEDS FIX >>>>>
			Creating cells is still based on the idea (enforced by protoregion) that all cells of a certain type (ex. Pyramidal) are to be created in the same envoy. Need to change the way synapses build their indexes etc.
		*/
		//.layer("ii", 3, layer::DEFAULT, Protocell::new_pyramidal(2, 5, vec!["out"], 512)) // <<<<< FIX ME (FIX SYNS)

		//.layer("temp_padding", 2, layer::DEFAULT, Axonal(Horizontal))
		.layer("motor", 1, layer::DEFAULT, ProtolayerKind::Axonal(ProtoaxonKind::Horizontal))

		.freeze()
	;

	cort_regs.add(sen);
	cort_regs
}

pub fn define_protoareas() -> Protoareas {
	let mut protoareas = Protoareas::new()
		.area("v1", 5, 5, ProtoregionKind::Sensory, Some(vec!["a1"]))
		.area("a1", 4, 4, ProtoregionKind::Sensory, None)
	;

	protoareas
}


pub struct Cortex {
	//pub cortical_area: CorticalArea, // <<<<< SLATED FOR REMOVAL
	protoregions: Protoregions,
	protoareas: Protoareas,
	cortical_areas: HashMap<&'static str, Box<CorticalArea>>,
	thalamus: Thalamus,
	ocl: ocl::Ocl, // GIVE TO THALAMUS
}

impl Cortex {
	pub fn new(protoregions: Protoregions, protoareas: Protoareas) -> Cortex {
		print!("\nInitializing Cortex... ");
		let time_start = time::get_time();		

		//let protoregions = define_protoregions();
		//let protoareas = define_protoareas();

		let mut cortical_areas = HashMap::new();

		let hrz_demarc = protoregions[&ProtoregionKind::Sensory].hrz_demarc();
		let hrz_demarc_opt = ocl::BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", hrz_demarc as i32);
		let build_options = cmn::build_options().add(hrz_demarc_opt);

		let ocl: ocl::Ocl = ocl::Ocl::new(build_options);

		// FOR EACH AREA...

		for (_, protoarea) in &protoareas {
			cortical_areas.insert(protoarea.name, Box::new(CorticalArea::new(protoarea.name, protoregions[&protoarea.region_kind].clone(), protoarea.clone(), &ocl)));
		}

		let thalamus = Thalamus::new(&protoareas);

		let time_complete = time::get_time() - time_start;
		println!("\n\n... Cortex initialized in: {}.{} sec.", time_complete.num_seconds(), time_complete.num_milliseconds());

		Cortex {
			//cortical_area: cortical_area,
			protoregions: protoregions,
			protoareas: protoareas,
			cortical_areas: cortical_areas,
			thalamus: thalamus,
			ocl: ocl, // GIVE TO THALAMUS
		}
	}


	
	pub fn area_mut(&mut self, area_name: &'static str) -> &mut Box<CorticalArea> {
		let e_string = format!("cortex::Cortex::area_mut(): Area: '{}' not found", area_name);
		self.cortical_areas.get_mut(area_name).expect(&e_string)
	}

	pub fn area(&self, area_name: &'static str) -> &Box<CorticalArea> {
		let e_string = format!("cortex::Cortex::area_mut(): Area: '{}' not found", area_name);
		self.cortical_areas.get(area_name).expect(&e_string)
	}

	/*	WRITE_VEC(): 
			TODO: 
				- VALIDATE "layer_target, OTHERWISE: 
					- thread '<main>' panicked at '[protoregions::Protoregion::index(): 
					invalid layer name: "XXXXX"]', src/protoregions.rs:339
						- Just have slice_ids return an option<u8>
				- Handle multi-slice input vectors (for input compression, etc.)
					- Update assert statement to support this
	*/
	pub fn write_vec(&mut self, area_name: &'static str, layer_target: &'static str, vec: &[ocl::cl_uchar]) {
		let emsg = "cortex::Cortex::write_vec()";
		let ref region = self.protoregions[&ProtoregionKind::Sensory];
		let axn_slices: Vec<u8> = region.slice_ids(vec!(layer_target));

		for slice in axn_slices { 
			let buffer_offset = cmn::axn_idx_2d(slice, self.cortical_areas.get(area_name).expect(emsg).dims.columns(), region.hrz_demarc()) as usize;
			//let buffer_offset = cmn::SYNAPSE_REACH_LIN + (axn_slice as usize * self.cortical_area.axns.dims.width as usize);

			//println!("##### write_vec(): {} offset: axn_idx_2d(axn_slice: {}, dims.columns(): {}, region.hrz_demarc(): {}): {}, vec.len(): {}", layer_target, slice, self.cortical_area.dims.columns(), region.hrz_demarc(), buffer_offset, vec.len());

			//assert!(vec.len() <= self.cortical_area.dims.columns() as usize); // <<<<< NEEDS CHANGING (for multi-slice inputs)

			ocl::enqueue_write_buffer(vec, self.cortical_areas.get(area_name).expect(emsg).axns.states.buf, self.ocl.command_queue, buffer_offset);
		}
	}


	pub fn sense_vec(&mut self, area_name: &'static str, layer_target: &'static str, vec: &[ocl::cl_uchar]) {
		self.write_vec(area_name, layer_target, vec);
		self.cycle();
	}

	pub fn cycle(&mut self) {
		//let ref region = &self.protoregions[&ProtoregionKind::Sensory];
		for (_, cortical_area) in self.cortical_areas.iter_mut() {
			cortical_area.cycle();
		}
	}

	/*pub fn release_components(&mut self) {
		print!("Cortex::release_components() called! (depricated)... ");
	}*/

	/*pub fn sense(&mut self, area_name: &'static str, layer_target: &'static str, chord: &Chord) {
		let mut vec: Vec<ocl::cl_uchar> = chord.unfold();
		self.sense_vec(area_name, layer_target, &vec);
		panic!("SLATED FOR REMOVAL");
	}*/
}

impl Drop for Cortex {
    fn drop(&mut self) {
        print!("Releasing OCL Components...");
		self.ocl.release_components();
    }
}










/*	fn cycle_syns(&self) {

		let width: u32 = self.areas.width(ProtoregionKind::Sensory);
		let depth_total: u8 = self.protoregions.depth_total(ProtoregionKind::Sensory);
		let (_, depth_cellular) = self.protoregions.depth(ProtoregionKind::Sensory);
		let len: u32 = dims.width * depth_total as u32;

		let test_envoy = Envoy::<ocl::cl_int>::new(width, depth_total, 0, &self.ocl);

		//println!("cycle_cel_syns running with dims.width = {}, depth = {}", width, depth_total);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_syns");
		ocl::set_kernel_arg(0, self.cortical_area.axns.states.buf, kern);
		ocl::set_kernel_arg(1, self.cortical_area.dst_dens.syns.axn_slice_ids.buf, kern);
		ocl::set_kernel_arg(2, self.cortical_area.dst_dens.syns.axn_col_offs.buf, kern);
		ocl::set_kernel_arg(3, self.cortical_area.dst_dens.syns.strengths.buf, kern);
		ocl::set_kernel_arg(4, self.cortical_area.dst_dens.syns.states.buf, kern);

		//println!("depth_total: {}, depth_cellular: {}, width_syn_slice: {}", depth_total, depth_cellular, width_syn_slice);

		let gws = (depth_cellular as usize, dims.width as usize, cmn::SYNAPSES_PER_CELL);

		//println!("gws: {:?}", gws);

		ocl::enqueue_3d_kernel(kern, self.ocl.command_queue, &gws);

	}*/

/*	fn cycle_dens(&self) {

		let width: u32 = self.areas.width(ProtoregionKind::Sensory);
		let (_, depth_cellular) = self.protoregions.depth(ProtoregionKind::Sensory);

		let width_dens: usize = dims.width as usize * cmn::DENDRITES_PER_CELL * depth_cellular as usize;

		let kern = ocl::new_kernel(self.ocl.program, "cycle_dens");

		ocl::set_kernel_arg(0, self.cortical_area.dst_dens.syns.states.buf, kern);
		ocl::set_kernel_arg(1, self.cortical_area.dst_dens.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.cortical_area.dst_dens.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, width_dens);

	}*/

/*	fn cycle_axns(&self) {
		let width: u32 = self.areas.width(ProtoregionKind::Sensory);
		let (depth_noncellular, depth_cellular) = self.protoregions.depth(ProtoregionKind::Sensory);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_axns");
		ocl::set_kernel_arg(0, self.cortical_area.dst_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.cortical_area.axns.states.buf, kern);
		ocl::set_kernel_arg(2, depth_noncellular as u32, kern);

		let gws = (depth_cellular as usize, dims.width as usize);

		ocl::enqueue_2d_kernel(kern, self.ocl.command_queue, &gws);

	}*/
