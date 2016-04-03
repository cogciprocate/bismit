
use cortex::Cortex;
use super::{hybrid, kernels, testbed, TestBed};


//     IDEAS FOR TESTS:
//         - set synapse src_ids, src_ofs, strs to 0
//         - test some specific inputs and make sure that synapses are responding exactly



#[test]
fn cortex() {
    let mut cortex = Cortex::new(testbed::define_protolayer_maps(), testbed::define_protoareas());
    hybrid::cycles(&mut cortex, testbed::PRIMARY_AREA_NAME);
}


#[test]
fn kernels() {
    let testbed = TestBed::new();
    kernels::axn_idxs(&testbed);
}

