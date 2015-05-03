#[cfg(test)]

//use super::*;
//use cortex::Cortex;
use cortex::{ Cortex };

#[test]
fn test_cortex() {

	let mut cortex = cortex::Cortex::new();

	/* MAKE THIS A STRUCT OR SOMETHING */
	let scw = common::SENSORY_CHORD_WIDTH;
	let scl_fct_log2 = common::log2(scw / 1024);
	let scw_1_2 = scw >> 1;
	let scw_1_4 = scw >> 2;
	let scw_3_4 = scw - scw_1_4;
	let scw_1_8 = scw >> 3;
	let scw_3_8 = scw_1_2 - scw_1_8;
	let scw_5_8 = scw_1_2 + scw_1_8;
	let scw_1_16 = scw >> 4;

	let mut vec1: Vec<ocl::cl_uchar> = Vec::with_capacity(scw as usize);

	for i in range(0, scw) {
		vec1.push(0);
	}

	cortex.sense_vec_no_cycle(0, "pre_thal", &mut vec1);
	cortex.sense_vec_no_cycle(0, "post_thal", &mut vec1);


	vec1.clear();
	for i in range(0, scw) {
		if i >= scw_1_2 - (scw_1_16 / 2) && i < scw_1_2 + (scw_1_16 / 2) {
		//if ((i >= scw_1_4 - scw_1_16) && (i < scw_1_4 + scw_1_16)) || ((i >= scw_3_4 - scw_1_16) && (i < scw_3_4 + scw_1_16)) {
		//if i >= scw_3_8 && i < scw_5_8 {
		//if (i >= scw_1_2 - scw_1_16 && i < scw_1_2 + scw_1_16) || (i < scw_1_16) || (i >= (scw - scw_1_16)) {
		//if i >= scw_3_8 && i < scw_5_8 {
		//if i < scw_1_16 {
			vec1.push(254);
		} else {
			vec1.push(0);
		}
	}



	if SHUFFLE_ONCE {
		common::shuffle_vec(&mut vec1);
	}



    assert!(super::test_4::test_cycle());

    assert!(synapse_test());
}


fn synapse_test(cortex: &Cortex, vec1: &Vec<ocl::cl_uchar>) -> bool {
	vec1.clear();

	// Write blank input and check synapses for columns and stuff

	// Then do some other simple tests
}
