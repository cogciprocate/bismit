
use cmn::{ self, CorticalDimensions };
use proto::{ ProtoLayerMap, ProtoLayerMaps, ProtoAreaMaps, ProtoAreaMap, Cellular, Axonal, Spatial, Horizontal, Sensory, Thalamic, layer, Protocell, Protofilter, Protoinput };
use cortex::{ self, Cortex };
use ocl::{ Envoy, WorkSize, OclProgQueue, EnvoyDimensions };
use super::input_czar::{ self, InputCzar, InputKind };
use super::hybrid;


// 	IDEAS FOR TESTS:
// 		- set synapse src_ids, src_ofs, strs to 0
// 		- test some specific inputs and make sure that synapses are responding exactly


/* Eventually move defines to a config file or some such */
pub fn define_protolayer_maps() -> ProtoLayerMaps {
	let mut cort_regs: ProtoLayerMaps = ProtoLayerMaps::new();

	cort_regs.add(ProtoLayerMap::new("visual", Sensory)
		//.layer("test_noise", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("motor_in", 1, layer::DEFAULT, Axonal(Horizontal))
		//.layer("olfac", 1, layer::DEFAULT, Axonal(Horizontal))
		.layer("eff_in", 0, layer::EFFERENT_INPUT, Axonal(Spatial))
		//.layer("nothing", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("aff_in", 0, layer::AFFERENT_INPUT, Axonal(Spatial))
		.layer("out", 1, layer::AFFERENT_OUTPUT | layer::EFFERENT_OUTPUT, Axonal(Spatial))
		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["aff_in"], 600)) 
		.layer("iv_inhib", 0, layer::DEFAULT, 
			Protocell::new_inhibitory(4, "iv"))
		.layer("iii", 1, layer::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(0, 5, vec!["iii"], 1200).apical(vec!["eff_in"]))
	);

	cort_regs.add(ProtoLayerMap::new("external", Thalamic)
		.layer("ganglion", 1, layer::AFFERENT_OUTPUT | layer::AFFERENT_INPUT, Axonal(Spatial))
	);

	cort_regs
}

pub fn define_protoareas() -> ProtoAreaMaps {
	let area_side = 32 as u32;

	let mut protoareas = ProtoAreaMaps::new()

		.area_ext("v0", "external", area_side, area_side, 
			Protoinput::IdxReader { 
				file_name: "data/train-images-idx3-ubyte", 
				repeats: 1,
			},

			None, 
			Some(vec!["v1"]),
		)

		.area("v1", "visual", area_side, area_side, 
			Some(vec![Protofilter::new("retina", Some("filters.cl"))]),			
			Some(vec!["b1"]),
			//None,
		)

		.area("b1", "visual", area_side, area_side,
		 	None,
		 	//Some(vec!["a1"]),
		 	None,
		)
	;

	protoareas
}


#[test]
fn test_cortex() {
	let mut cortex = Cortex::new(define_protolayer_maps(), define_protoareas());
	let area_name = "v1";

	hybrid::test_cycles(&mut cortex, area_name);
}



#[test]
fn test_learning() {
	let mut cortex = Cortex::new(define_protolayer_maps(), define_protoareas());
	let area_name = "v1";
	let si_layer_name = "iv_inhib";

	hybrid::test_learning(&mut cortex, si_layer_name, area_name);
}


#[test]
// TEST_KERNELS(): TODO: NEED TO UPDATE TO NEW OCL INSTANTIATION SYSTEM
fn test_kernels() {
	// let hrz_demarc_opt = ocl::BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", 128 as i32);
	// let build_options = cmn::build_options().add(hrz_demarc_opt);
	// let ocl = ocl::OclProgQueue::new(build_options);
	// let dims = ocl::CorticalDimensions::new(16, 16, 1, 0, Some(ocl.get_max_work_group_size()));
	//let (ocl, dims) = init_ocl();

	//test_safe_dim_ofs(&ocl, dims.clone());

	//ocl.release_components();
}

fn test_safe_dim_ofs(ocl: &OclProgQueue, dims: CorticalDimensions) {
	let mut dim_ids = Envoy::<u32>::shuffled(dims, 0, 15, &ocl);
	let mut dim_offs = Envoy::<i8>::shuffled(dims, -16, 15, &ocl);
	let mut safe_dim_offs = Envoy::<i8>::new(dims, 0, &ocl);

	let kern_test_safe_dim_ofs = ocl.new_kernel("test_safe_dim_ofs".to_string(), 
		WorkSize::OneDim(dims.physical_len() as usize))
		.arg_env(&dim_ids)
		.arg_env(&dim_offs)
		.arg_scl(dims.u_size())
		.arg_env(&safe_dim_offs) 
	;

	kern_test_safe_dim_ofs.enqueue();

	println!("dim_ids:");
	dim_ids.print_simple();
	println!("dim_offs:");
	dim_offs.print_simple();
	println!("safe_dim_offs:");
	safe_dim_offs.print_simple();
	//safe_dim_offs.read();

	for i in 0..safe_dim_offs.len() {
		let safe_dim_id: isize = dim_ids[i] as isize + safe_dim_offs[i] as isize;
		assert!(safe_dim_id >= 0);
		assert!(safe_dim_id < dims.u_size() as isize);
	}
}



// pub fn init_ocl() -> (ocl::OclProgQueue, ocl::CorticalDimensions) {
// 	// let hrz_demarc_opt = ocl::BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", 128 as i32);
// 	// let build_options = cmn::build_options().add(hrz_demarc_opt);
// 	// let ocl = ocl::OclProgQueue::new(build_options);
// 	//let dims = ocl::CorticalDimensions::new(16, 16, 1, 0, Some(ocl.get_max_work_group_size()));
// 	(ocl, dims)
// }
