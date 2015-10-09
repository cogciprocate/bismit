// use std::ops::{ Range };
use std::iter;
use std::io::{ /*self,*/ Write, /*Stdout*/ };
// use std::mem;
use rand;

use interactive::{ /*self,*/ /*input_czar,*/ /*InputCzar, InputKind*/ };
use proto::*;

use cortical_area::{ CorticalAreaTest };
use cmn::{ CelCoords };
use synapses::tests::{ SynCoords };
use axon_space::{ AxnCoords, AxonSpaceTest };
use cortex::{ Cortex };
use cmn::{ self, DataCellLayer };
use super::{ testbed };


// TEST_PYR_SYN_LEARNING(): 
// Set up conditions in which a synapse should learn and its neighbors should not.
/* 			
		Choose a pyramidal cell.
		Activate the column axon (mcol/sst) for that cell's column.

		Choose a synapse from that cell.		
		
			Check for safety/validity.	
			Confirm that the pyramidal cell has the correct flags (none):
				PYR_PREV_CONCRETE_FLAG: u8 		= 0b10000000;	// 128	(0x80)
				PYR_BEST_IN_COL_FLAG: u8 		= 0b01000000;	// 64	(0x40)
				PYR_PREV_STP_FLAG: u8 			= 0b00100000;	// 32	(0x20)
				PYR_PREV_FUZZY_FLAG: u8			= 0b00010000;	// 16	(0x10)
			Determine whether or not the synapse has the correct flags (none):
				SYN_STP_FLAG: u8				= 0b00000001;
				SYN_STD_FLAG: u8				= 0b00000010;
				SYN_CONCRETE_FLAG: u8			= 0b00001000;
			Check flags on other synapses (should be none).

		Set synapse strengths to zero for entire cell.
		Find its source pyr axon index.
		Activate that pyr axon (to 196).

		Cycle.
			Check:
				Pyr should have no flags set.
					- It should have CONCRETE flag if more than den thresh syns have been activated.
				Syn should have no flags set.
				Other syns on cell should have no flags or value
				Value should be non-zero (if src pyr axn was 196, syn should be 226).
				Values of other synapses should be zero.
		Learn.
			Check. 
				Value should be unchanged. 
				Pyr should have CONCRETE flag only.
				Syn should have STP & CONCRETE, others should have nothing.
		
		Verify that nearby synapses have undergone LTD.
		Deactivate column and nearby pyr axon.

		Cycle.
			Check.
		Learn.
			Check. 
*/
#[test]
fn test_ptal_syn_learning() {		
	let mut cortex = Cortex::new(testbed::define_protolayer_maps(), testbed::define_protoareas());
	let mut area = cortex.area_mut(testbed::PRIMARY_AREA_NAME);
	//let pyr_out_axn_idz = area.area_map().axn_idz(area.ptal().axn_slc_base());
	//let pyr_col_in_axn_idz = area.area_map().axn_idz(area.psal().axn_slc_base());
	let psal_axn_slc = area.psal().axn_slc_base();
	let ptal_axn_slc = area.ptal().axn_slc_base();

	println!("\nInitiating synapse learning test for the primary temporal associative layer \
		of area: '{}'.", testbed::PRIMARY_AREA_NAME);

	for i in 0..1 {
		let pyr = CelCoords::new_random(area.ptal());
		let syn = SynCoords::new_random(area.ptal().dims(), pyr);
		let axn_in = AxnCoords::from_cel_coords(psal_axn_slc, &syn.cel_coords, area.area_map()).unwrap();
		let axn_out = AxnCoords::from_cel_coords(ptal_axn_slc, &syn.cel_coords, area.area_map()).unwrap();

		area.write_to_axon(axn_in, 255);

		area.axns.states.print_simple();
		println!("syn: {:?}", syn);
	}

	assert!(false, " JUST DEBUGGING :) ");
}





// #[test]
// fn test_learning_cell_range_() {
// 	let mut cortex = Cortex::new(testbed::define_protolayer_maps(), testbed::define_protoareas());
// 	test_learning_cell_range(&mut cortex, testbed::INHIB_LAYER_NAME, testbed::PRIMARY_AREA_NAME);
// } 

// #[test]
// fn test_learning_activation_() {
// 	let mut cortex = Cortex::new(testbed::define_protolayer_maps(), testbed::define_protoareas());
// 	test_learning_activation(&mut cortex, testbed::PRIMARY_AREA_NAME);
// }


// NEED A 'TestParameters' STRUCT OF SOME SORT TO UNTANGLE THIS AND MOVE STUFF INTO CHILD FUNCTIONS
#[test]
pub fn test_learning_activation(/*cortex: &mut Cortex,*/ /*area_name: &str*/) {
	let mut cortex = Cortex::new(testbed::define_protolayer_maps(), testbed::define_protoareas());
	let area_name = testbed::PRIMARY_AREA_NAME;

	let emsg = "\ntests::hybrid::test_pyr_activation()";
	let activation_test_runs = 2;

	let learning_test_runs = 1;

	let layer_name = cortex.area_mut(area_name).ptal_name();
	let ff_layer_name = cortex.area_mut(area_name).psal_name();

	let src_slc_ids = cortex.area_mut(area_name).area_map().proto_layer_map()
		.src_slc_ids(layer_name, DendriteKind::Distal);
	let src_slc_id = src_slc_ids[0];
	
	let ff_layer_axn_idz = cortex.area_mut(area_name).mcols.ff_layer_axn_idz();


	let cels_len = cortex.area_mut(area_name).ptal().dims().cells() as usize;

	let cols_len = cortex.area_mut(area_name).dims().columns() as usize;

	let (cels_axn_idz, _) = {// CELS IN SCOPE
		let cels = cortex.area_mut(area_name).ptal_mut();

		{ // SET ALL SYNAPSES TO THE SAME SOURCE AXON SLICE AND ZEROS ELSEWHERE
			let mut syns = cels.dens_mut().syns_mut();
			syns.src_slc_ids.set_all_to(src_slc_id);
			syns.src_col_v_offs.set_all_to(0);
			syns.strengths.set_all_to(0);
			syns.states.set_all_to(0);
			syns.flag_sets.set_all_to(0);
		}

		cels.set_all_to_zero();
		//cels.flag_sets.set_all_to(0);

		cels.axn_range()
	};

	let (dens_per_tuft, syns_per_tuft, syns_per_den) = {// CELS IN SCOPE
		let cels = cortex.area_mut(area_name).ptal_mut();

		let dens_per_tuft = cels.dens_mut().dims().per_cel() as usize;
		let syns_per_tuft = cels.dens_mut().syns_mut().dims().per_cel() as usize;

		assert!(syns_per_tuft % dens_per_tuft == 0);

		let syns_per_den = syns_per_tuft / dens_per_tuft;
		(dens_per_tuft, syns_per_tuft, syns_per_den)
	};


	let mut vec_ff: Vec<u8> = iter::repeat(0).take(cortex.area_mut(area_name).dims.columns() as usize).collect();

	println!("Running {} activation tests...", activation_test_runs);

	for i in 0..activation_test_runs {	//	TEST CORRECT AXON ACTIVATION
		let last_run = activation_test_runs - 1 == i;
		let cel_idx = rand::random::<usize>() & (cels_len - 1);
		let col_id = cel_idx & (cols_len - 1);

		println!("[{}] => ", cel_idx);

		vec_ff[col_id] = 100;

		//cortex.write(area_name, ff_layer_name, &vec_ff);
		cortex.area(area_name).psal().soma().write_direct(&vec_ff, 0);

		// write_input(&self, sdr: &Sdr, layer_flags: layer::ProtolayerFlags)

		if last_run {
			println!("\nACTIVATING CELLS... ");
		}


		/*	TEST ACTIVATION

		*/
		// FIRST ACTIVATION:
		cortex.area_mut(area_name).mcols().activate();

		{// AXNS IN SCOPE
			let axns = &mut cortex.area_mut(area_name).axns;
			axns.states.read();

			let cel_axn_state = axns.states[cels_axn_idz + cel_idx];

			if last_run {
				println!("layer '{}' axons (cels_axn_idz: {}, cel_idx: {}): ", layer_name, cels_axn_idz, cel_idx);
				cmn::print_vec(&axns.states.vec()[cels_axn_idz..(cels_axn_idz + cels_len)], 1, None, None, false);
				println!("\ncell[{}] axon state: {}", cel_idx, cel_axn_state);

				println!(" => ");
			}

			for i in 0..cels_len {				
				if i & (cols_len - 1) == col_id {
					print!("[{}]", i);
					assert!(axns.states[cels_axn_idz + i] == cel_axn_state);
				} else {
					assert!(axns.states[cels_axn_idz + i] == 0);
				}
			}


			{
				//	TODO: TEST FLAG CORRECTNESS (before and after)
			}
		}



		/*	TEST PYR LEARNING
				- set half of the synapses on a random dendrite belonging to our target cell to 100
					- may need to reset some flags or other things
				- run activate() again
				- ensure that the only active cell is our target cell, and that it's fellow columners are inactive

		*/

		println!("   Running {} activation-learning tests... ", learning_test_runs);


		/*  SYNAPSE STUFF SHOULD BE REUSABLE (for any cell type)  */

		// PICK A RANDOM HALF OF DENDRITE SYNAPSES
		let first_half: bool = rand::random::<bool>();

		// CHOOSE RANDOM DEN ID
		let den_id = rand::random::<usize>() & (dens_per_tuft - 1);

		// DETERMINE DEN_IDX
		let den_idx = (cel_idx * dens_per_tuft) + den_id;

		// DEFINE FIRST AND (LAST + 1) SYN INDEXES
		let syn_idz = den_idx * syns_per_den;
		let syn_idn = syn_idz + syns_per_den;

		// DEFINE ' ' FOR ACTIVE HALF
		let syn_tar_half_idz = syn_idz + if first_half {0} else {syns_per_den >> 1};
		let syn_tar_half_idn = syn_tar_half_idz + (syns_per_den >> 1);

		
		for i in 0..learning_test_runs {
			let last_learning_run = i == (learning_test_runs - 1);

			// REACTIVATE FF AXON
			vec_ff[col_id] = 100;
			//cortex.write(area_name, ff_layer_name, &vec_ff);
			cortex.area(area_name).psal().soma().write_direct(&vec_ff, 0);


			{// CELS IN SCOPE
				let cels = cortex.area_mut(area_name).ptal_mut();

				if last_run && last_learning_run {
					println!("uINDEXES: first_half: {}, den_id: {}, den_idx: {}, syn_idz: {}, syn_idn: {}, syn_tar_half_idz: {}, syn_tar_half_idn: {}", first_half, den_id, den_idx, syn_idz, syn_idn, syn_tar_half_idz, syn_tar_half_idn);
				}

				for syn_idx in syn_tar_half_idz..syn_tar_half_idn {
					cels.dens_mut().syns_mut().states[syn_idx] = 128;
				}


				if last_run && last_learning_run {
					println!("\nWRITING SYNAPSES AND CYCLING CELLS... ");
				}

				cels.dens_mut().syns_mut().states.write();

				cels.dens_mut().cycle_self_only();
				cels.cycle_self_only();

			}


			/* 	MUST CALL MINICOLUMN_OUTPUT() (__kernel void col_output() KERNEL TO DETERMINE IF ANY PYRS ARE ACTIVE
					- col_output() will cycle through each column's pyrs and set the (what should be a)
						flag declaring whether or not at least one pyr in the column is predictive
					- the output to the minicolumn's axon shouldn't affect tests at all
			*/
			cortex.area(area_name).mcols.output();


			{// CELS IN SCOPE
				let cels = cortex.area_mut(area_name).ptal_mut();

				if last_run && last_learning_run {
					cels.print_cel(cel_idx);
				}
			}



			// PRINT AXONS ETC.
			// AXNS IN SCOPE -- DO NOT EDIT ME -- MULTIPLE BLOCKS EXIST (until we move to separate fn)
			if last_run && last_learning_run {
				let axns = &mut cortex.area_mut(area_name).axns;
				axns.states.read();
				let cel_axn_state = axns.states[cels_axn_idz + cel_idx];

				println!("layer '{}' axons (cels_axn_idz: {}, cel_idx: {}): ", layer_name, cels_axn_idz, cel_idx);
				cmn::print_vec(&axns.states.vec()[cels_axn_idz..(cels_axn_idz + cels_len)], 1, None, None, false);
				println!("\ncell[{}] axon state: {}", cel_idx, cel_axn_state);
			}
			



			// SECOND ACTIVATION:
			// TODO TEST: should see cell axon go higher
			if true { 
				if last_run && last_learning_run {
					println!("\nACTIVATING CELLS AGAIN (2ND TIME)... ");
				}

				cortex.area_mut(area_name).mcols().activate();
			}


			if last_run && last_learning_run {
				cortex.area_mut(area_name).ptal_mut().print_cel(cel_idx);
			}

			// TODO: TEST FOR CORRECT FLAG_SETS

			// LEARNING
			if true { 
				if last_run && last_learning_run {
					println!("\nPERFORMING LEARNING... ");
				}

				cortex.area_mut(area_name).ptal_mut().learn();

				if last_run && last_learning_run {
					cortex.area_mut(area_name).ptal_mut().print_cel(cel_idx);
				}
			}
			


			/*  SIMULATE NEXT CYCLE()  */

			// DEACTIVATE FF AXON
			vec_ff[col_id] = 0;
			//cortex.write(area_name, ff_layer_name, &vec_ff);
			cortex.area(area_name).psal().soma().write_direct(&vec_ff, 0);


			// DEACTIVATE SYNAPSES
			let den_idz = cel_idx * dens_per_tuft;
			let syn_idz = den_idz * syns_per_den;

			// RESET ENTIRE CELL TO ZERO (even though only half of one dendrite should be active)
			for syn_idx in syn_idz..(syn_idz + syns_per_tuft) {
				cortex.area_mut(area_name).ptal_mut().dens_mut().syns_mut().states[syn_idx] = 0;
			}

			// WRITE AND CYCLE
			cortex.area_mut(area_name).ptal_mut().dens_mut().syns_mut().states.write();
			cortex.area_mut(area_name).ptal_mut().dens_mut().cycle_self_only();
			cortex.area_mut(area_name).ptal_mut().cycle_self_only();

			// ACTIVATE AND LEARN
			cortex.area_mut(area_name).mcols().activate();
			cortex.area_mut(area_name).ptal_mut().learn();

			// PRINT AND TEST
			if last_run && last_learning_run {
				println!("\nPERFORMING LEARNING AGAIN (2ND CYCLE) -- SHOULD SEE LTP... ");
				cortex.area_mut(area_name).ptal_mut().print_cel(cel_idx);
			}



			// TODO: TEST FOR LTP



			// PRINT AXONS ETC.
			// AXNS IN SCOPE -- DO NOT EDIT ME -- MULTIPLE BLOCKS EXIST (until we move to separate fn)
			if last_run && last_learning_run {
				let axns = &mut cortex.area_mut(area_name).axns;
				axns.states.read();
				let cel_axn_state = axns.states[cels_axn_idz + cel_idx];

				println!("layer '{}' axons (cels_axn_idz: {}, cel_idx: {}): ", layer_name, cels_axn_idz, cel_idx);
				cmn::print_vec(&axns.states.vec()[cels_axn_idz..(cels_axn_idz + cels_len)], 1, None, None, false);
				println!("\ncell[{}] axon state: {}", cel_idx, cel_axn_state);
			}			

			// CLEAR CURRENTLY SET FF VALUE BACK TO ZERO FOR NEXT RUN (should set entire vector to 0s)
			vec_ff[col_id] = 0;

			for i in 0..cols_len {
				assert!(vec_ff[i] == 0);
			}
		}	
	}

	println!("test_activation(): {} ", super::PASS_STR);
	//assert!(false);
}









/*	TEST THAT CORRECT RANGE OF CELLS IS BEING AFFECTED BY A SINGLE LEARN
		Simulate a learning situation for a single sst
			- set axn ofs to 0
			- set strs to 0
			- choose 1 target cell, stimulate half of its synapses
			- make sure all other cells are still at 0 str
			- make sure that our target cell has the correct half of its synapse strs increased

			- perform regrowth
			- check offs to ensure change
*/
#[test]
pub fn test_learning_cell_range(/*cortex: &mut Cortex,*/ /*ilyr_name: &'static str, area_name: &str*/) {
	let mut cortex = Cortex::new(testbed::define_protolayer_maps(), testbed::define_protoareas());
	let (ilyr_name, area_name) = (testbed::INHIB_LAYER_NAME, testbed::PRIMARY_AREA_NAME);
	//let psal_name = cortex.area(area_name).psal_name();
	//println!("##### hybrid::test_learning(): psal_name: {}", psal_name);
	//let ptal_name = cortex.area(area_name).ptal_name();
	_test_sst_learning(&mut cortex, ilyr_name, area_name);
	//_test_pyr_learning(cortex, ptal_name, area_name);
}

fn _test_sst_learning(cortex: &mut Cortex, /*layer_name: &'static str,*/ ilyr_name: &'static str, area_name: &str) {
	let emsg = "\ntests::hybrid::_test_sst_learning()";


	let (dens_per_tuft, syns_per_tuft, syns_per_den) = {// CELS IN SCOPE
		let cels = cortex.area_mut(area_name).ptal_mut();

		let dens_per_tuft = cels.dens_mut().dims().per_cel() as usize;
		let syns_per_tuft = cels.dens_mut().syns_mut().dims().per_cel() as usize;

		assert!(syns_per_tuft % dens_per_tuft == 0);

		let syns_per_den = syns_per_tuft / dens_per_tuft;
		(dens_per_tuft, syns_per_tuft, syns_per_den)
	};

	//let em99 = &format!("{}: {}; layer_name: {} ", emsg, "cel_idx (em99)", layer_name);
	let cel_idx_mask = (cortex.area_mut(area_name).psal().dims().cells() as usize) - 1;
	let cel_idx = rand::random::<usize>() & cel_idx_mask;


	{
		let cels = cortex.area_mut(area_name).ptal_mut();

		//let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.area_mut(area_name).dims.columns() as usize).collect();

		//let cel_syns = &mut ;
		cels.dens_mut().syns_mut().src_col_v_offs.set_all_to(0);
		cels.dens_mut().syns_mut().strengths.set_all_to(0);
		cels.dens_mut().syns_mut().states.set_all_to(0);

		let first_half: bool = rand::random::<bool>();
		let per_cel = cels.dens_mut().syns_mut().dims().per_cel() as usize;

		let cel_syn_idz = cel_idx << cels.dens_mut().syns_mut().dims().per_tuft_l2_left();
		let cel_syn_tar_idz = cel_syn_idz + if first_half {0} else {per_cel >> 1};
		let cel_syn_tar_idn = cel_syn_tar_idz + (per_cel >> 1);
		
		println!("\n{}: cel_idx: {}, per_cel: {}, cel_syn_tar_idz: {}, cel_syn_tar_idn: {}", emsg, cel_idx, per_cel, cel_syn_tar_idz, cel_syn_tar_idn);

		for syn_idx in cel_syn_tar_idz..cel_syn_tar_idn {
			cels.dens_mut().syns_mut().states[syn_idx] = 255;
		}

		cels.dens_mut().syns_mut().states.write();
		cels.dens_mut().cycle_self_only();
		//cels.soma().cycle_self_only();
	}

	cortex.area_mut(area_name).iinns.get_mut(ilyr_name).expect(&format!("{}: {}", emsg, "ilyr_name")).cycle(false);


	{
		let cels = cortex.area_mut(area_name).psal_mut();

		for i in 0..100 {
			cels.learn();
		}

		cels.dens_mut().confab();

		cels.print_cel(cel_idx);
		
		println!("\nREGROWING... ");
		cels.regrow();

		cels.print_cel(cel_idx);
	}

	//assert!(false);

	//println!("ALL CELLS: cell.syn_strengths[{:?}]: ", cel_syn_idz..(cel_syn_idz + per_cel));
	//cmn::print_vec(&cels.dens_mut().syns_mut().strengths.vec()[..], 1, None, None, false);

	//check src_col_v_offs
	//check strengths
	//check offs and strs for other cells to make sure they're untouched

}





// pub fn _test_pyr_learning(cortex: &mut Cortex, layer_name: &str, area_name: &str) {
// 	let emsg = "tests::hybrid::test_pyr_learning()";

// 	{
// 		let cels = cortex.area_mut(area_name).ptal_mut();

// 		//let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.area_mut(area_name).dims.columns() as usize).collect();

// 		//let cel_syns = &mut ;
// 		cels.dens_mut().syns_mut().src_col_v_offs.set_all_to(0);
// 		cels.dens_mut().syns_mut().strengths.set_all_to(0);
// 		cels.dens_mut().syns_mut().states.set_all_to(0);

// 		let first_half: bool = rand::random::<bool>();
// 		let per_cel = cels.dens_mut().syns_mut().dims().per_cel() as usize;

// 		let cel_idx = rand::random::<usize>() & ((cels.dims().cells() as usize) - 1);
// 		let cel_syn_idz = cel_idx << cels.dens_mut().syns_mut().dims().per_tuft_l2_left();
// 		let cel_syn_tar_idz = cel_syn_idz + if first_half {0} else {per_cel >> 1};
// 		let cel_syn_tar_idn = cel_syn_tar_idz + (per_cel >> 1);

// 		let col_id = cel_idx & (cels.dims().columns() as usize - 1);
		
// 		println!("\n{}: cel_idx: {}, per_cel: {}, cel_syn_tar_idz: {}, cel_syn_tar_idn: {}", emsg, cel_idx, per_cel, cel_syn_tar_idz, cel_syn_tar_idn);

// 		for syn_idx in cel_syn_tar_idz..cel_syn_tar_idn {
// 			cels.dens_mut().syns_mut().states[syn_idx] = 255;
// 		}

// 		cels.dens_mut().syns_mut().states.write();
// 		cels.dens_mut().cycle_self_only();
// 		//cels.soma().cycle_self_only();
// 	}

// 	cortex.area_mut(area_name).iinns.get_mut("iv_inhib").expect(emsg).cycle(false);	

// 	let cels = cortex.area_mut(area_name).ptal_mut();

// 	for i in 0..100 {
// 		cels.learn();
// 	}

// 	cels.dens_mut().confab();

// 	cels.print_cel(cel_idx);
	
// 	println!("\nREGROWING... ");
// 	cels.regrow();

// 	cels.print_cel(cel_idx);

// 	println!("ALL CELLS: cell.syn_strengths[{:?}]: ", cel_syn_idz..(cel_syn_idz + per_cel));
// 	cmn::print_vec(&cels.dens_mut().syns_mut().strengths.vec()[..], 1, None, None, false);

// 	//check src_col_v_offs
// 	//check strengths
// 	//check offs and strs for other cells to make sure they're untouched

// }
