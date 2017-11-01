//! Automated (`cargo test`) versions of tests which can be also be from a
//! non-test build.

use cortex::Cortex;
use subcortex::{InputGenerator, /*Subcortex*/};
use super::{hybrid, kernels, testbed, TestBed};


//     IDEAS FOR TESTS:
//         - set synapse src_ids, src_ofs, strs to 0
//         - test some specific inputs and make sure that synapses are responding exactly



#[test]
fn cortex() {
    let layer_map_schemes = testbed::define_layer_map_schemes();
    let area_schemes = testbed::define_protoareas();

    let input_gen = InputGenerator::new(&layer_map_schemes, &area_schemes, "v0").unwrap();
    // let subcortex = Subcortex::new().nucleus(input_gen);
    let mut cortex = Cortex::builder(layer_map_schemes, area_schemes)
        .subcortical_nucleus(input_gen)
        .build().unwrap();

    hybrid::cycles(&mut cortex, testbed::PRIMARY_AREA_NAME);
}


#[test]
fn kernels() {
    let testbed = TestBed::new();
    kernels::axn_idxs(&testbed);
}

