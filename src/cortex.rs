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
use proto::{ Protoregion, Protoregions, Protoareas, ProtoareasTrait, Protoarea, Cellular, Axonal, Spatial, Horizontal, Sensory, layer, Protocell };

pub struct Cortex {
	areas: HashMap<&'static str, Box<CorticalArea>>,
	thal: Thalamus,
}

impl Cortex {
	pub fn new(mut protoregions: Protoregions, protoareas: Protoareas) -> Cortex {
		print!("\nInitializing Cortex... ");
		let time_start = time::get_time();

		protoregions.freeze();

		let mut areas = HashMap::new();

		let hrz_demarc = protoregions[&Sensory].hrz_demarc();
		let hrz_demarc_opt = ocl::BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", hrz_demarc as i32);
		let mut build_options = cmn::build_options().add(hrz_demarc_opt);
		build_options.kern("filters.cl".to_string());
		build_options.kern("bismit.cl".to_string());

		let ocl: ocl::Ocl = ocl::Ocl::new(build_options);

		for (_, protoarea) in &protoareas {
			let mut protoarea_clone = protoarea.clone();
			protoarea_clone.dims.set_physical_increment(ocl.get_max_work_group_size());

			areas.insert(protoarea_clone.name, Box::new(CorticalArea::new(protoarea_clone.name, protoregions[&protoarea.region_kind].clone(), protoarea_clone, &ocl)));
		}

		let thal = Thalamus::new(protoareas, protoregions, ocl);

		// <<<<< MOVE THIS TO CMN AND MAKE A FUNCTION FOR IT >>>>>
		let time_complete = time::get_time() - time_start;
		let t_sec = time_complete.num_seconds();
		let t_ms = time_complete.num_milliseconds() - (t_sec * 1000);
		println!("\n\n... Cortex initialized in: {}.{} seconds.", t_sec, t_ms);

		Cortex {
			//cortical_area: cortical_area,
			//protoregions: protoregions,
			//protoareas: protoareas,
			areas: areas,
			thal: thal,
			//ocl: ocl, // GIVE TO THALAMUS
		}
	}


	
	pub fn area_mut(&mut self, area_name: &str) -> &mut Box<CorticalArea> {
		let emsg = format!("cortex::Cortex::area_mut(): Area: '{}' not found. ", area_name);
		self.areas.get_mut(area_name).expect(&emsg)
	}

	pub fn area(&self, area_name: &str) -> &Box<CorticalArea> {
		let emsg = format!("cortex::Cortex::area_mut(): Area: '{}' not found. ", area_name);
		self.areas.get(area_name).expect(&emsg)
	}


	pub fn write_input(&mut self, area_name: &str, sdr: &[ocl::cl_uchar]) {
		let emsg = format!("cortex::Cortex::write_input(): Area: '{}' not found. ", area_name);
		let area = self.areas.get_mut(area_name).expect(&emsg);
		self.thal.write_input(sdr, area);
		//self.thal.write_input(area_name, sdr, &mut self.areas)
	}


	/* WRITE(): TESTING PURPOSES */
	pub fn write(&mut self, area_name: &str, layer_target: &'static str, sdr: &[ocl::cl_uchar]) {
		self.thal.write(area_name, layer_target, sdr, &mut self.areas)
	}

	/*pub fn sense_vec(&mut self, area_name: &'static str, layer_target: &'static str, sdr: &[ocl::cl_uchar]) {
		//self.thal.write(area_name, layer_target, sdr, &self.areas);
		self.write(area_name, layer_target, sdr);
		self.cycle();
	}*/

	pub fn cycle(&mut self, area_name: &str) {
		let emsg = format!("cortex::Cortex::cycle(): Area: '{}' not found. ", area_name);

		let afferent_areas = {
			//println!("\nCycling '{}'", area_name);
			self.areas.get_mut(area_name).expect(&emsg).cycle()
		};

		match afferent_areas {
			Some(aff_area_names) => {
				for area_name_aff in aff_area_names {
					//println!("\nForwarding from '{}' to '{}'", area_name, area_name_aff);
					self.thal.forward_afferent_output(area_name, area_name_aff, &mut self.areas);

					self.cycle(area_name_aff);

					self.thal.backward_efferent_output(area_name_aff, area_name, &mut self.areas);					
				}
			},

			None => (),
		};
	}

	pub fn print_area_output(&mut self, ao_name: &str) {
		self.area_mut(ao_name).axns.states.read();

		let (out_start_ao, out_end_ao) = self.area(ao_name).mcols.axn_output_range();
		let out_slc_ao = &self.area(ao_name).axns.states.vec[out_start_ao..out_end_ao];

		let cols = self.area(ao_name).dims.columns(); // DEBUG PURPOSES
		print!("\nArea: '{}' - out_start_ao: {}, out_end_ao: {}, cols: {}", ao_name, out_start_ao, out_end_ao, cols);

		//cmn::render_sdr(out_slc_ao, None, None, None, &self.area(ao_name).protoregion().slc_map(), true, cols);

	}

	pub fn valid_area(&self, area_name: &str) -> bool {
		self.areas.contains_key(area_name)
	}
}




	/*	WRITE_VEC(): 
			TODO: 
				- VALIDATE "layer_target, OTHERWISE: 
					- thread '<main>' panicked at '[protoregions::Protoregion::index(): 
					invalid layer name: "XXXXX"]', src/protoregions.rs:339
						- Just have slc_ids return an option<u8>
				- Handle multi-slc input vectors (for input compression, etc.)
					- Update assert statement to support this
	*/
	/*pub fn write_vec(&mut self, area_name: &'static str, layer_target: &'static str, vec: &[ocl::cl_uchar]) {
		let emsg = "cortex::Cortex::write_vec()";
		let ref region = self.protoregions[&Sensory];
		let axn_slcs: Vec<u8> = region.slc_ids(vec!(layer_target));

		for slc in axn_slcs { 
			let buffer_offset = cmn::axn_idx_2d(slc, self.areas.get(area_name).expect(emsg).dims.columns(), region.hrz_demarc()) as usize;
			//let buffer_offset = cmn::SYNAPSE_REACH_LIN + (axn_slc as usize * self.cortical_area.axns.dims.width as usize);

			//println!("##### write_vec(): {} offset: axn_idx_2d(axn_slc: {}, dims.columns(): {}, region.hrz_demarc(): {}): {}, vec.len(): {}", layer_target, slc, self.cortical_area.dims.columns(), region.hrz_demarc(), buffer_offset, vec.len());

			//assert!(vec.len() <= self.cortical_area.dims.columns() as usize); // <<<<< NEEDS CHANGING (for multi-slc inputs)

			ocl::enqueue_write_buffer(vec, self.areas.get(area_name).expect(emsg).axns.states.buf, self.ocl.command_queue, buffer_offset);
		}
	}*/



	/* Eventually move define_*() to a config file or some such */
/*pub fn define_protoregions() -> Protoregions {
	let mut cort_regs: Protoregions = Protoregions::new();

	let mut sen = Protoregion::new(Sensory)
		//.layer("test_noise", 1, layer::DEFAULT, Axonal(Spatial))

		.layer("eff_in", 1, layer::EFFERENT_INPUT, Axonal(Spatial))

		.layer("aff_in", 1, layer::AFFERENT_INPUT, Axonal(Spatial))

		.layer("aff_out", 1, layer::AFFERENT_OUTPUT | layer::EFFERENT_OUTPUT, Axonal(Spatial))

		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, Protocell::new_spiny_stellate(5, vec!["aff_in"], 256)) 
		//.layer("vi", 5, layer::DEFAULT, Protocell::new_spiny_stellate(3, vec!["thal"], 256)) 

		.layer("iv_inhib", 0, layer::DEFAULT, Protocell::new_inhibitory(4, "iv"))

		//.layer("iii", 1, layer::DEFAULT, Protocell::new_pyramidal(vec!["iii", "iii", "iii", "iii", "motor"]))
		.layer("iii", 4, layer::TEMPORAL_ASSOCIATIVE, Protocell::new_pyramidal(2, 4, vec!["iii"], 512).apical(vec!["eff_in"]))

		/*	<<<<< ADDING ADDITIONAL PYRS (AND PRESUMABLY SSTS) NEEDS FIX >>>>>
			Creating cells is still based on the idea (enforced by protoregion) that all cells of a certain type (ex. Pyramidal) are to be created in the same envoy. Need to change the way synapses build their indexes etc.
		*/
		//.layer("ii", 3, layer::DEFAULT, Protocell::new_pyramidal(2, 5, vec!["out"], 512)) // <<<<< FIX ME (FIX SYNS)

		//.layer("temp_padding", 2, layer::DEFAULT, Axonal(Horizontal))
		.layer("motor", 1, layer::DEFAULT, Axonal(Horizontal))

		.freeze()
	;

	cort_regs.add(sen);
	cort_regs
}

pub fn define_protoareas() -> Protoareas {
	let mut protoareas = Protoareas::new()
		//.area("v1", 32, 32, Sensory, Some(vec!["v2"]))
		.area("v1", 48, 48, Sensory, Some(vec!["b1"]))
		.area("b1", 48, 48, Sensory, Some(vec!["a1"]))
		.area("a1", 48, 48, Sensory, None)
	;

	protoareas
}*/


	/*pub fn release_components(&mut self) {
		print!("Cortex::release_components() called! (depricated)... ");
	}*/

	/*pub fn sense(&mut self, area_name: &'static str, layer_target: &'static str, chord: &Chord) {
		let mut vec: Vec<ocl::cl_uchar> = chord.unfold();
		self.sense_vec(area_name, layer_target, &vec);
		panic!("SLATED FOR REMOVAL");
	}*/




/*impl Drop for Cortex {
    fn drop(&mut self) {
        print!("Releasing OCL Components...");
		self.ocl.release_components();
    }
}*/


/*	fn cycle_syns(&self) {

		let width: u32 = self.areas.width(Sensory);
		let depth_total: u8 = self.protoregions.depth_total(Sensory);
		let (_, depth_cellular) = self.protoregions.depth(Sensory);
		let len: u32 = dims.width * depth_total as u32;

		let test_envoy = Envoy::<ocl::cl_int>::new(width, depth_total, 0, &self.ocl);

		//println!("cycle_cel_syns running with dims.width = {}, depth = {}", width, depth_total);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_syns");
		ocl::set_kernel_arg(0, self.cortical_area.axns.states.buf, kern);
		ocl::set_kernel_arg(1, self.cortical_area.dst_dens.syns.axn_slc_ids.buf, kern);
		ocl::set_kernel_arg(2, self.cortical_area.dst_dens.syns.axn_col_offs.buf, kern);
		ocl::set_kernel_arg(3, self.cortical_area.dst_dens.syns.strengths.buf, kern);
		ocl::set_kernel_arg(4, self.cortical_area.dst_dens.syns.states.buf, kern);

		//println!("depth_total: {}, depth_cellular: {}, width_syn_slc: {}", depth_total, depth_cellular, width_syn_slc);

		let gws = (depth_cellular as usize, dims.width as usize, cmn::SYNAPSES_PER_CELL);

		//println!("gws: {:?}", gws);

		ocl::enqueue_3d_kernel(kern, self.ocl.command_queue, &gws);

	}*/

/*	fn cycle_dens(&self) {

		let width: u32 = self.areas.width(Sensory);
		let (_, depth_cellular) = self.protoregions.depth(Sensory);

		let width_dens: usize = dims.width as usize * cmn::DENDRITES_PER_CELL * depth_cellular as usize;

		let kern = ocl::new_kernel(self.ocl.program, "cycle_dens");

		ocl::set_kernel_arg(0, self.cortical_area.dst_dens.syns.states.buf, kern);
		ocl::set_kernel_arg(1, self.cortical_area.dst_dens.thresholds.buf, kern);
		ocl::set_kernel_arg(2, self.cortical_area.dst_dens.states.buf, kern);

		ocl::enqueue_kernel(kern, self.ocl.command_queue, width_dens);

	}*/

	/*	fn cycle_axns(&self) {
		let width: u32 = self.areas.width(Sensory);
		let (depth_noncellular, depth_cellular) = self.protoregions.depth(Sensory);

		let kern = ocl::new_kernel(self.ocl.program, "cycle_axns");
		ocl::set_kernel_arg(0, self.cortical_area.dst_dens.states.buf, kern);
		ocl::set_kernel_arg(1, self.cortical_area.axns.states.buf, kern);
		ocl::set_kernel_arg(2, depth_noncellular as u32, kern);

		let gws = (depth_cellular as usize, dims.width as usize);

		ocl::enqueue_2d_kernel(kern, self.ocl.command_queue, &gws);

	}*/
