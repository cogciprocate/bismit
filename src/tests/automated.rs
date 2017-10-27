//! Automated (`cargo test`) versions of tests which can be also be from a
//! non-test build.

use cortex::Cortex;
use super::{hybrid, kernels, testbed, TestBed};


//     IDEAS FOR TESTS:
//         - set synapse src_ids, src_ofs, strs to 0
//         - test some specific inputs and make sure that synapses are responding exactly



#[test]
fn cortex() {
    let mut cortex = Cortex::builder(testbed::define_layer_map_schemes(),
        testbed::define_protoareas()).build().unwrap();
    hybrid::cycles(&mut cortex, testbed::PRIMARY_AREA_NAME);
}


#[test]
fn kernels() {
    let testbed = TestBed::new();
    kernels::axn_idxs(&testbed);
}

