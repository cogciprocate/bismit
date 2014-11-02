
pub fn space_req() {

	static NEURONS_PER_COLUMN: uint = 32;


		let axon: uint = 16; // (8bit neuron address <uint8> + 8bit synapse address <uint8>)
			
			let synapse: uint = 8; // (8bit weight <int8>)

			let state: uint = 16; // (last input state for each of the 16 synapses)

			let threshold: uint = 16; // (16bit threshold value)
		
		let dendrite: uint = (synapse * 16) + history;

	let neuron: uint = (dendrite * 16) + (axon * 256);


		let neuron_state: uint = NEURONS_PER_COLUMN * 32; // 32 bit int for each neuron

	let column: uint = (neuron * NEURONS_PER_COLUMN) + axon + dendrite + neuron_state;


	let hypercolumn: uint = column * 64;


	

	println!("column(b): {}", column);
	println!("hypercolumn(b): {}", hypercolumn/8);

	//let column: uint = (neuron * 16) + dendrite +


	/*
	BISMIT TYPES:


	Neuron: (672b)
		-Axon (32b) x 256 =
			-Synapse Address<u8> (1b Synapse + 1b Neuron = 2b) x 

		-Dendrite (40b) x 16 =
			-Synapses x 16
				-Weight<i4> 16 x (2b) =

		-Area of influence = 4 hypercolumns

	Column: 11,488
		-Axon (32b) x 1 = 32b
		-Dendrite (40b) x 16 = 640b
		-Neuron (672b) x 1-256 (16 default) = 10,752b
		-Neuron State + History (16b) x 4 = 64b
		Input from previous level
		Outputs to next level

	Hypercolumn: 735,232
		-Column (11,488) x 64
		-ActiveOutput (1b) x 1
	*/
}