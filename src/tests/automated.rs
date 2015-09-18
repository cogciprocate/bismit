
use cmn;
use proto::{ Protoregion, Protoregions, ProtoregionKind, Protoareas, ProtoareasTrait, Protoarea, ProtolayerKind, ProtoaxonKind, layer, Protocell };
use cortex::{ self, Cortex };
use ocl;
use super::input_czar::{ self, InputCzar, InputVecKind };
use super::hybrid;


pub fn define_prtrgns() -> Protoregions {
	Protoregions::new()
		.region(Protoregion::new(ProtoregionKind::Sensory)
			.l("thal_t", 1, layer::AFFERENT_INPUT, ProtolayerKind::Axonal(ProtoaxonKind::Spatial))
			.l("out_t", 1, layer::AFFERENT_OUTPUT, ProtolayerKind::Axonal(ProtoaxonKind::Spatial))
			.l("iv_t", 1, layer::SPATIAL_ASSOCIATIVE, Protocell::new_spiny_stellate(5, vec!["thal_t"], 256))  // , "motor"
			.l("iv_inhib_t", 0, layer::DEFAULT, Protocell::new_inhibitory(4, "iv_t"))
			.l("iii_t", 4, layer::TEMPORAL_ASSOCIATIVE, Protocell::new_pyramidal(2, 5, vec!["iii_t"], 256))
			.l("motor_t", 1, layer::DEFAULT, ProtolayerKind::Axonal(ProtoaxonKind::Horizontal))
			.freeze()
		)
}

pub fn define_prtareas() -> Protoareas {
	Protoareas::new().area("v1_t", 32, 32, ProtoregionKind::Sensory, None)
}

pub fn init_ocl() -> (ocl::OclProgQueue, ocl::CorticalDimensions) {
	let hrz_demarc_opt = ocl::BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", 128 as i32);
	let build_options = cmn::build_options().add(hrz_demarc_opt);
	let ocl = ocl::OclProgQueue::new(build_options);
	let dims = ocl::CorticalDimensions::new(16, 16, 1, 0, Some(ocl.get_max_work_group_size()));
	(ocl, dims)
}

/* IDEAS FOR TESTS:
	- set synapse src_ids, src_ofs, strs to 0
		- test some specific inputs and make sure that synapses are responding exactly
*/
#[test]
fn test_cortex() {
	let mut cortex = Cortex::new(define_prtrgns(), define_prtareas());
	let area_name = "v1_t";

	hybrid::test_cycles(&mut cortex, area_name);
}



#[test]
fn test_learning() {
	let mut cortex = Cortex::new(define_prtrgns(), define_prtareas());
	let area_name = "v1_t";
	let si_layer_name = "iv_inhib_t";

	hybrid::test_learning(&mut cortex, si_layer_name, area_name);
}


#[test]
fn test_kernels() {
	// let hrz_demarc_opt = ocl::BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", 128 as i32);
	// let build_options = cmn::build_options().add(hrz_demarc_opt);
	// let ocl = ocl::OclProgQueue::new(build_options);
	// let dims = ocl::CorticalDimensions::new(16, 16, 1, 0, Some(ocl.get_max_work_group_size()));
	let (ocl, dims) = init_ocl();

	test_safe_dim_ofs(&ocl, dims.clone());

	ocl.release_components();
}

fn test_safe_dim_ofs(ocl: &ocl::OclProgQueue, dims: ocl::CorticalDimensions) {
	let mut dim_ids = ocl::Envoy::<u32>::shuffled(dims, 0, 15, &ocl);
	let mut dim_offs = ocl::Envoy::<i8>::shuffled(dims, -16, 15, &ocl);
	let mut safe_dim_offs = ocl::Envoy::<i8>::new(dims, 0, &ocl);

	let kern_test_safe_dim_ofs = ocl.new_kernel("test_safe_dim_ofs", 
		ocl::WorkSize::OneDim(dims.physical_len() as usize))
		.arg_env(&dim_ids)
		.arg_env(&dim_offs)
		.arg_scl(dims.u_size())
		.arg_env(&safe_dim_offs) 
	;

	kern_test_safe_dim_ofs.enqueue();

	print!("\ndim_ids:");
	dim_ids.print_simple();
	print!("\ndim_offs:");
	dim_offs.print_simple();
	print!("\nsafe_dim_offs:");
	safe_dim_offs.print_simple();
	//safe_dim_offs.read();

	for i in 0..safe_dim_offs.len() {
		let safe_dim_id: isize = dim_ids[i] as isize + safe_dim_offs[i] as isize;
		assert!(safe_dim_id >= 0);
		assert!(safe_dim_id < dims.u_size() as isize);
	}
}
