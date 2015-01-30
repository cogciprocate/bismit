use cortex;
use test_readback;

pub fn run() {
	println!("=== test_3::run() ===");
	let mut my_cortex = cortex::Cortex::new();

	let mut test_vec1: Vec<u32> = Vec::new();

	for i in range(0, 1024) {
		test_vec1.push(i as u32);
	}

	

	
		test_readback::test_readback(
					&my_cortex, 
					&my_cortex.cells.synapses.values.vec, 
					my_cortex.cells.synapses.values.buf, 
					"test_synapse"
		);
	
	
	
	/*
		test_readback::test_readback(
					&my_cortex, 
					&my_cortex.cells.axons.target_cell_synapses.vec, 
					my_cortex.cells.axons.target_cell_synapses.buf, 
					"test_cell_axon"
		);
	*/
	
	/*
		test_readback::test_readback(
					&my_cortex, 
					&my_cortex.cells.axons.target_cells.vec, 
					my_cortex.cells.axons.target_cells.buf, 
					"test_cell_axon"
		);
	*/

	my_cortex.release_components();

	println!("=== test_3.run() complete ===");
}
