use cortex;
use readback_test;

pub fn run() {
	println!("=== test_3::run() ===");
	let mut my_cortex = cortex::Cortex::new();

	/*
	readback_test::readback_test(
				&my_cortex, 
				&my_cortex.cortex_segments[0].columns.synapses.values.vec, 
				my_cortex.cortex_segments[0].columns.synapses.values.buff, 
				"test_synapse"
	);
	*/
	
	//my_cortex.readback_test(&my_cortex.axons, my_cortex.axons_buff, "test_axon");
	
	readback_test::readback_test(
				&my_cortex, 
				&my_cortex.cortex_segments[0].cells.axons.target_cell_synapses.vec, 
				my_cortex.cortex_segments[0].cells.axons.target_cell_synapses.buff, 
				"test_cell_axon"
	);
	
	//my_cortex.readback_test(&my_cortex.dendrite_states, my_cortex.dendrite_states_buff, "test_axon");


	my_cortex.release_components();

	println!("=== test_3.run() complete ===");
}
