
// use cmn::{ self, CorticalDims };
// use proto::{ ProtolayerMap, ProtolayerMaps, ProtoareaMaps, ProtoareaMap, Cellular, Axonal, Spatial, Horizontal, Sensory, Thalamic, layer, Protocell, Protofilter, Protoinput };
use cortex::{ /*self,*/ Cortex };
// use thalamus::{ Thalamus };
// use ocl::{ Buffer, WorkDims, Context, ProQue, BufferDims, ProgramBuilder, BuildOption };
// use interactive::{ input_czar, InputCzar, InputKind };
use super::{ hybrid, kernels, testbed, TestBed };


//     IDEAS FOR TESTS:
//         - set synapse src_ids, src_ofs, strs to 0
//         - test some specific inputs and make sure that synapses are responding exactly



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

