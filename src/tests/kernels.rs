
use cmn::{ self, /*CorticalDims*/ };
// use proto::{ ProtolayerMap, ProtolayerMaps, ProtoareaMaps, ProtoareaMap, Cellular, Axonal, Spatial, Horizontal, Sensory, Thalamic, layer, Protocell, Protofilter, Protoinput };
// use cortex::{ self, Cortex };
use ocl::{ Buffer, SimpleDims, /*ProQue, BufferDims,*/ /*OclNum*/ };
// use interactive::{ input_czar, InputCzar, InputKind };
// use super::hybrid;
use super::{ TestBed, util };
 


// TEST THAT:
//        - VECTORIZED AND NON-VECTORIZED INDEX RESOLUTION FUNCTIONS RETURN THE SAME RESULTS
//         - KERNEL CALCULATED AXON INDEXES FALL WITHIN THE CORRECT RANGE (ON THE APPROPRIATE SLICE)
//         - 
pub fn test_axn_idxs(testbed: &TestBed) {
    let syn_reach = cmn::SYNAPSE_REACH as i8;

    let u_offs = Buffer::<i8>::with_vec_shuffled((0 - syn_reach, syn_reach + 1), 
        &testbed.dims, &testbed.ocl_pq.queue()); 
    let v_offs = Buffer::<i8>::with_vec_shuffled((0 - syn_reach, syn_reach + 1), 
        &testbed.dims, &testbed.ocl_pq.queue());

    let mut outs_sc = Buffer::<u32>::with_vec(&testbed.dims, testbed.ocl_pq.queue());
    let mut outs_v4 = Buffer::<u32>::with_vec(&testbed.dims, testbed.ocl_pq.queue());

    let kern_sc = testbed.ocl_pq.create_kernel("test_axn_idxs_scl")
        .gws(SimpleDims::Three(testbed.dims.depth() as usize, testbed.dims.v_size() as usize,
            testbed.dims.u_size() as usize))
        .arg_buf(&u_offs)        
        .arg_buf(&v_offs)
        .arg_buf(&outs_sc) 
        //.arg_buf(&outs_v4) 
    ;

    let kern_v4 = testbed.ocl_pq.create_kernel("test_axn_idxs_vec4")
        .gws(SimpleDims::Three(testbed.dims.depth() as usize, testbed.dims.v_size() as usize, 
            (testbed.dims.u_size() / 4) as usize))
        .arg_buf(&u_offs)        
        .arg_buf(&v_offs)
        //.arg_buf(&outs_sc) 
        .arg_buf(&outs_v4) 
    ;

    kern_sc.enqueue();
    kern_v4.enqueue();

    let failure = util::compare_buffers(&mut outs_sc, &mut outs_v4);

    if failure { panic!("Vectorized and non-vectorized kernel results are not equal.") };
}



// pub fn test_safe_dim_ofs(ocl: &ProQue, dims: CorticalDims) {
//     let mut dim_ids = Buffer::<u32>::shuffled(dims, 0, 15, &ocl);
//     let mut dim_offs = Buffer::<i8>::shuffled(dims, -16, 15, &ocl);
//     let mut safe_dim_offs = Buffer::<i8>::new(dims, 0, &ocl);

//     let kern_test_safe_dim_ofs = ocl.create_kernel_with_dims("test_safe_dim_ofs", 
//         SimpleDims::One(dims.len() as usize))
//         .arg_buf(&dim_ids)
//         .arg_buf(&dim_offs)
//         .arg_scl(dims.u_size())
//         .arg_buf(&safe_dim_offs) 
//     ;

//     kern_test_safe_dim_ofs.enqueue();

//     println!("dim_ids:");
//     dim_ids.print_simple();
//     println!("dim_offs:");
//     dim_offs.print_simple();
//     println!("safe_dim_offs:");
//     safe_dim_offs.print_simple();
//     //safe_dim_offs.fill_vec();

//     for i in 0..safe_dim_offs.len() {
//         let safe_dim_id: i64 = dim_ids[i] as i64 + safe_dim_offs[i] as i64;
//         assert!(safe_dim_id >= 0);
//         assert!(safe_dim_id < dims.u_size() as i64);
//     }
// }

