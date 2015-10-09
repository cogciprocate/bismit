
use cmn::{ self, /*CorticalDimensions*/ };
// use proto::{ ProtoLayerMap, ProtoLayerMaps, ProtoAreaMaps, ProtoAreaMap, Cellular, Axonal, Spatial, Horizontal, Sensory, Thalamic, layer, Protocell, Protofilter, Protoinput };
// use cortex::{ self, Cortex };
use ocl::{ Envoy, WorkSize, /*OclProgQueue, EnvoyDimensions,*/ OclNum };
// use interactive::{ input_czar, InputCzar, InputKind };
// use super::hybrid;
use super::{ TestBed };
 


// TEST THAT:
//		- VECTORIZED AND NON-VECTORIZED INDEX RESOLUTION FUNCTIONS RETURN THE SAME RESULTS
// 		- KERNEL CALCULATED AXON INDEXES FALL WITHIN THE CORRECT RANGE (ON THE APPROPRIATE SLICE)
// 		- 


pub fn test_axn_idxs(testbed: &TestBed) {
	let syn_reach = cmn::SYNAPSE_REACH as i8;

	let u_offs = Envoy::<i8>::shuffled(testbed.dims, 0 - syn_reach, syn_reach + 1, &testbed.ocl); 
	let v_offs = Envoy::<i8>::shuffled(testbed.dims, 0 - syn_reach, syn_reach + 1, &testbed.ocl);

	let mut outs_sc = Envoy::<u32>::new(testbed.dims, 0, &testbed.ocl);
	let mut outs_v4 = Envoy::<u32>::new(testbed.dims, 0, &testbed.ocl);

	let kern_sc = testbed.ocl.new_kernel("test_axn_idxs_scl".to_string(), 
		WorkSize::ThreeDim(testbed.dims.depth() as usize, testbed.dims.v_size() as usize, testbed.dims.u_size() as usize))
		.arg_env(&u_offs)		
		.arg_env(&v_offs)
		.arg_env(&outs_sc) 
		//.arg_env(&outs_v4) 
	;

	let kern_v4 = testbed.ocl.new_kernel("test_axn_idxs_vec4".to_string(), 
		WorkSize::ThreeDim(testbed.dims.depth() as usize, testbed.dims.v_size() as usize, (testbed.dims.u_size() / 4) as usize))
		.arg_env(&u_offs)		
		.arg_env(&v_offs)
		//.arg_env(&outs_sc) 
		.arg_env(&outs_v4) 
	;

	kern_sc.enqueue();
	kern_v4.enqueue();

	let failure = compare_envoys(&mut outs_sc, &mut outs_v4);

	if failure { panic!("Vectorized and non-vectorized kernel results are not equal.") };
}


pub fn compare_envoys<T: OclNum>(env1: &mut Envoy<T>, env2: &mut Envoy<T>) -> bool {	
	print!("\nVector comparison:\n");	
	assert!(env1.vec().len() == env2.vec().len());

	env1.read();
	env2.read();

	let mut failure = false;

	for i in 0..env1.vec().len() {
		let (e1_val, e2_val) = (env1.vec()[i], env2.vec()[i]);

		if e1_val != e2_val {
			failure = true;
			print!("{}", cmn::C_RED);
		} else {
			print!("{}", cmn::C_DEFAULT);
		}

		print!("[n:{}, v4:{}]{}", e1_val, e2_val, cmn::C_DEFAULT);
	}

	print!("\n");

	failure
}


// pub fn test_safe_dim_ofs(ocl: &OclProgQueue, dims: CorticalDimensions) {
// 	let mut dim_ids = Envoy::<u32>::shuffled(dims, 0, 15, &ocl);
// 	let mut dim_offs = Envoy::<i8>::shuffled(dims, -16, 15, &ocl);
// 	let mut safe_dim_offs = Envoy::<i8>::new(dims, 0, &ocl);

// 	let kern_test_safe_dim_ofs = ocl.new_kernel("test_safe_dim_ofs".to_string(), 
// 		WorkSize::OneDim(dims.len() as usize))
// 		.arg_env(&dim_ids)
// 		.arg_env(&dim_offs)
// 		.arg_scl(dims.u_size())
// 		.arg_env(&safe_dim_offs) 
// 	;

// 	kern_test_safe_dim_ofs.enqueue();

// 	println!("dim_ids:");
// 	dim_ids.print_simple();
// 	println!("dim_offs:");
// 	dim_offs.print_simple();
// 	println!("safe_dim_offs:");
// 	safe_dim_offs.print_simple();
// 	//safe_dim_offs.read();

// 	for i in 0..safe_dim_offs.len() {
// 		let safe_dim_id: isize = dim_ids[i] as isize + safe_dim_offs[i] as isize;
// 		assert!(safe_dim_id >= 0);
// 		assert!(safe_dim_id < dims.u_size() as isize);
// 	}
// }

