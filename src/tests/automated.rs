
// use cmn::{ self, CorticalDimensions };
// use proto::{ ProtoLayerMap, ProtoLayerMaps, ProtoAreaMaps, ProtoAreaMap, Cellular, Axonal, Spatial, Horizontal, Sensory, Thalamic, layer, Protocell, Protofilter, Protoinput };
use cortex::{ /*self,*/ Cortex };
// use thalamus::{ Thalamus };
// use ocl::{ Envoy, WorkSize, OclContext, OclProgQueue, EnvoyDimensions, BuildOptions, BuildOption };
// use interactive::{ input_czar, InputCzar, InputKind };
use super::{ hybrid, kernels, testbed, TestBed };


// 	IDEAS FOR TESTS:
// 		- set synapse src_ids, src_ofs, strs to 0
// 		- test some specific inputs and make sure that synapses are responding exactly



#[test]
fn test_cortex() {
	let mut cortex = Cortex::new(testbed::define_protolayer_maps(), testbed::define_protoareas());
	hybrid::test_cycles(&mut cortex, testbed::PRIMARY_AREA_NAME);
}


#[test]
fn test_kernels() {
	let testbed = TestBed::new();
	kernels::test_axn_idxs(&testbed);
}



// pub fn init_ocl() -> (OclProgQueue, CorticalDimensions) {
// 	let mut build_options = gen_build_options();

// 	let ocl_context = OclContext::new(None);
// 	let mut ocl = OclProgQueue::new(&ocl_context, None);
// 	ocl.build(build_options);

// 	let dims = CorticalDimensions::new(32, 32, 1, 0, Some(ocl.get_max_work_group_size()));

// 	(ocl, dims)
// }


// pub fn gen_build_options() -> BuildOptions {

// 	proto_area_maps.freeze();

// 	let thal = Thalamus::new(&proto_layer_maps, &proto_area_maps);

// 	let mut build_options = cmn::base_build_options()
// 		.opt("HORIZONTAL_AXON_ROW_DEMARCATION", 128 as i32)
// 		.opt("AXN_SLC_COUNT", self.slices.depth() as i32)
// 		.add_opt(BuildOption::with_str_val("AXN_SLC_IDZS", literal_list(self.slices.axn_idzs())))
// 		.add_opt(BuildOption::with_str_val("AXN_SLC_V_SIZES", literal_list(self.slices.v_sizes())))
// 		.add_opt(BuildOption::with_str_val("AXN_SLC_U_SIZES", literal_list(self.slices.u_sizes())))
// 		.add_opt(BuildOption::with_str_val("AXN_SLC_V_SCALES", literal_list(self.slices.v_scales())))
// 		.add_opt(BuildOption::with_str_val("AXN_SLC_U_SCALES", literal_list(self.slices.u_scales())))
// 	;

// 	cmn::load_builtin_kernel_files(&mut build_options);

// 	build_options
// }
