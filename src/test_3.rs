use cortex;

pub fn run() {
	println!("=== test_3::run() ===");
	let mut my_cortex = cortex::Cortex::new();

	my_cortex.init();

	my_cortex.readback_test(&my_cortex.synapses, my_cortex.synapses_buff, "test_synapse");
	my_cortex.readback_test(&my_cortex.axons, my_cortex.axons_buff, "test_axon");
	my_cortex.readback_test(&my_cortex.dendrite_states, my_cortex.dendrite_states_buff, "test_axon");

	my_cortex.release_components();

	println!("=== test_3.run() complete ===");
}
