use cortex;
use readback_test;

pub fn run() {
	println!("=== test_3::run() ===");
	let mut my_cortex = cortex::Cortex::new();

	let mut test_vec1: Vec<u32> = Vec::new();

	for i in range(0, 1024) {
		test_vec1.push(i as u32);
	}

	

	
		readback_test::readback_test(
					&my_cortex, 
					&my_cortex.cortical_segments[0].columns.synapses.values.vec, 
					my_cortex.cortical_segments[0].columns.synapses.values.buf, 
					"test_synapse"
		);
	
	
	
	/*
		readback_test::readback_test(
					&my_cortex, 
					&my_cortex.cortical_segments[0].cells.axons.target_cell_synapses.vec, 
					my_cortex.cortical_segments[0].cells.axons.target_cell_synapses.buf, 
					"test_cell_axon"
		);
	*/
	
	/*
		readback_test::readback_test(
					&my_cortex, 
					&my_cortex.cortical_segments[0].cells.axons.target_cells.vec, 
					my_cortex.cortical_segments[0].cells.axons.target_cells.buf, 
					"test_cell_axon"
		);
	*/

	my_cortex.release_components();

	println!("=== test_3.run() complete ===");
}
