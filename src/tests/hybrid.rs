use std::ops::{ Range };
use std::iter;
use std::io::{ self, Write, Stdout };
use std::mem;
use rand;

use super::input_czar::{ self, InputCzar, InputKind };
use proto::*;
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use pyramidals::{ PyramidalCellularLayer };
use cortex::{ self, Cortex };
use cmn;
use ocl::{ self, Envoy };


/*	HYBRID TESTS: Tests runnable in either an interactive or automated manner
		Useful for:
			- designing the test itself
			- troubleshooting test failures
			- diagnosing tangential issues not explicitly checked for
*/


pub static PASS_STR: &'static str = "\x1b[1;32mpass\x1b[0m";
 


// NEED A 'TestParameters' STRUCT OF SOME SORT TO UNTANGLE THIS AND MOVE STUFF INTO CHILD FUNCTIONS
pub fn test_activation_and_learning(cortex: &mut Cortex, area_name: &str) {
	let emsg = "\ntests::hybrid::test_pyr_activation()";
	let activation_test_runs = 2;

	let learning_test_runs = 5;

	let layer_name = cortex.area_mut(area_name).ptal_name();
	let ff_layer_name = cortex.area_mut(area_name).psal_name();

	let src_slc_ids = cortex.area_mut(area_name).protolayer_map().src_slc_ids(layer_name, DendriteKind::Distal);
	let src_slc_id = src_slc_ids[0];
	
	let ff_layer_axn_idz = cortex.area_mut(area_name).mcols.ff_layer_axn_idz();


	let cels_len = cortex.area_mut(area_name).ptal().dims().cells() as usize;

	let cols_len = cortex.area_mut(area_name).dims().columns() as usize;

	let (cels_axn_idz, _) = {// CELS IN SCOPE
		let cels = cortex.area_mut(area_name).ptal_mut();

		// SET ALL SYNAPSES TO THE SAME SOURCE AXON SLICE AND ZEROS ELSEWHERE
		cels.dens_mut().syns.src_slc_ids.set_all_to(src_slc_id);
		cels.dens_mut().syns.src_col_v_offs.set_all_to(0);
		cels.dens_mut().syns.strengths.set_all_to(0);
		cels.dens_mut().syns.states.set_all_to(0);
		cels.dens_mut().syns.flag_sets.set_all_to(0);

		cels.set_all_to_zero();
		//cels.flag_sets.set_all_to(0);

		cels.axn_range()
	};

	let (dens_per_tuft, syns_per_tuft, syns_per_den) = {// CELS IN SCOPE
		let cels = cortex.area_mut(area_name).ptal_mut();

		let dens_per_tuft = cels.dens_mut().dims().per_cel() as usize;
		let syns_per_tuft = cels.dens_mut().syns.dims().per_cel() as usize;

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

		cortex.write(area_name, ff_layer_name, &vec_ff);

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
				cmn::print_vec(&axns.states.vec[cels_axn_idz..(cels_axn_idz + cels_len)], 1, None, None, false);
				println!("\ncell[{}] axon state: {}", cel_idx, cel_axn_state);

				println!(" => ");
			}

			for i in 0..cels_len {
				if i & (cols_len - 1) == col_id {
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
			cortex.write(area_name, ff_layer_name, &vec_ff);


			{// CELS IN SCOPE
				let cels = cortex.area_mut(area_name).ptal_mut();

				if last_run && last_learning_run {
					println!("uINDEXES: first_half: {}, den_id: {}, den_idx: {}, syn_idz: {}, syn_idn: {}, syn_tar_half_idz: {}, syn_tar_half_idn: {}", first_half, den_id, den_idx, syn_idz, syn_idn, syn_tar_half_idz, syn_tar_half_idn);
				}

				for syn_idx in syn_tar_half_idz..syn_tar_half_idn {
					cels.dens_mut().syns.states[syn_idx] = 128;
				}


				if last_run && last_learning_run {
					println!("\nWRITING SYNAPSES AND CYCLING CELLS... ");
				}

				cels.dens_mut().syns.states.write();

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
			{// AXNS IN SCOPE -- DO NOT EDIT ME -- MULTIPLE BLOCKS EXIST (until we move to separate fn)
				if last_run && last_learning_run {
					let axns = &mut cortex.area_mut(area_name).axns;
					axns.states.read();
					let cel_axn_state = axns.states[cels_axn_idz + cel_idx];

					println!("layer '{}' axons (cels_axn_idz: {}, cel_idx: {}): ", layer_name, cels_axn_idz, cel_idx);
					cmn::print_vec(&axns.states.vec[cels_axn_idz..(cels_axn_idz + cels_len)], 1, None, None, false);
					println!("\ncell[{}] axon state: {}", cel_idx, cel_axn_state);
				}
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
			cortex.write(area_name, ff_layer_name, &vec_ff);


			// DEACTIVATE SYNAPSES
			let den_idz = cel_idx * dens_per_tuft;
			let syn_idz = den_idz * syns_per_den;

			// RESET ENTIRE CELL TO ZERO (even though only half of one dendrite should be active)
			for syn_idx in syn_idz..(syn_idz + syns_per_tuft) {
				cortex.area_mut(area_name).ptal_mut().dens_mut().syns.states[syn_idx] = 0;
			}

			// WRITE AND CYCLE
			cortex.area_mut(area_name).ptal_mut().dens_mut().syns.states.write();
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
			{// AXNS IN SCOPE -- DO NOT EDIT ME -- MULTIPLE BLOCKS EXIST (until we move to separate fn)
				if last_run && last_learning_run {
					let axns = &mut cortex.area_mut(area_name).axns;
					axns.states.read();
					let cel_axn_state = axns.states[cels_axn_idz + cel_idx];

					println!("layer '{}' axons (cels_axn_idz: {}, cel_idx: {}): ", layer_name, cels_axn_idz, cel_idx);
					cmn::print_vec(&axns.states.vec[cels_axn_idz..(cels_axn_idz + cels_len)], 1, None, None, false);
					println!("\ncell[{}] axon state: {}", cel_idx, cel_axn_state);
				}
			}



			// CLEAR CURRENTLY SET FF VALUE BACK TO ZERO FOR NEXT RUN (should set entire vector to 0s)
			vec_ff[col_id] = 0;

			for i in 0..cols_len {
				assert!(vec_ff[i] == 0);
			}
		}	
	}

	println!("test_activation(): {} ", PASS_STR);
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
pub fn test_learning(cortex: &mut Cortex, ilyr_name: &'static str, area_name: &str) {
	let psal_name = cortex.area(area_name).psal_name();
	println!("##### hybrid::test_learning(): psal_name: {}", psal_name);
	//let ptal_name = cortex.area_mut(area_name).ptal_name();
	_test_sst_learning(cortex, psal_name, ilyr_name, area_name);
	//_test_pyr_learning(cortex, ptal_name);
}

fn _test_sst_learning(cortex: &mut Cortex, layer_name: &'static str, ilyr_name: &'static str, area_name: &str) {
	let emsg = "\ntests::hybrid::_test_sst_learning()";


	let (dens_per_tuft, syns_per_tuft, syns_per_den) = {// CELS IN SCOPE
		let cels = cortex.area_mut(area_name).ptal_mut();

		let dens_per_tuft = cels.dens_mut().dims().per_cel() as usize;
		let syns_per_tuft = cels.dens_mut().syns.dims().per_cel() as usize;

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
		cels.dens_mut().syns.src_col_v_offs.set_all_to(0);
		cels.dens_mut().syns.strengths.set_all_to(0);
		cels.dens_mut().syns.states.set_all_to(0);

		let first_half: bool = rand::random::<bool>();
		let per_cel = cels.dens_mut().syns.dims().per_cel() as usize;

		let cel_syn_idz = cel_idx << cels.dens_mut().syns.dims().per_tuft_l2_left();
		let cel_syn_tar_idz = cel_syn_idz + if first_half {0} else {per_cel >> 1};
		let cel_syn_tar_idn = cel_syn_tar_idz + (per_cel >> 1);
		
		println!("\n{}: cel_idx: {}, per_cel: {}, cel_syn_tar_idz: {}, cel_syn_tar_idn: {}", emsg, cel_idx, per_cel, cel_syn_tar_idz, cel_syn_tar_idn);

		for syn_idx in cel_syn_tar_idz..cel_syn_tar_idn {
			cels.dens_mut().syns.states[syn_idx] = 255;
		}

		cels.dens_mut().syns.states.write();
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

	//println!("ALL CELLS: cell.syn_strengths[{:?}]: ", cel_syn_idz..(cel_syn_idz + per_cel));
	//cmn::print_vec(&cels.dens_mut().syns.strengths.vec[..], 1, None, None, false);

	//check src_col_v_offs
	//check strengths
	//check offs and strs for other cells to make sure they're untouched

}


/*pub fn _test_pyr_learning(cortex: &mut Cortex, layer_name: &'static str) {
	let emsg = "tests::hybrid::test_pyr_learning()";

	let cels = cortex.area_mut(area_name).ptal_mut();

	//let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.area_mut(area_name).dims.columns() as usize).collect();

	//let cel_syns = &mut ;
	cels.dens_mut().syns.src_col_v_offs.set_all_to(0);
	cels.dens_mut().syns.strengths.set_all_to(0);
	cels.dens_mut().syns.states.set_all_to(0);

	let first_half: bool = rand::random::<bool>();
	let per_cel = cels.dens_mut().syns.dims().per_cel().expect(emsg) as usize;

	let cel_idx = rand::random::<usize>() & ((cels.dims().cells() as usize) - 1);
	let cel_syn_idz = cel_idx << cels.dens_mut().syns.dims().per_tuft_l2_left();
	let cel_syn_tar_idz = cel_syn_idz + if first_half {0} else {per_cel >> 1};
	let cel_syn_tar_idn = cel_syn_tar_idz + (per_cel >> 1);

	let col_id = cel_idx & (cels.dims().columns() as usize - 1);
	
	println!("\n{}: cel_idx: {}, per_cel: {}, cel_syn_tar_idz: {}, cel_syn_tar_idn: {}", emsg, cel_idx, per_cel, cel_syn_tar_idz, cel_syn_tar_idn);

	for syn_idx in cel_syn_tar_idz..cel_syn_tar_idn {
		cels.dens_mut().syns.states[syn_idx] = 255;
	}

	cels.dens_mut().syns.states.write();
	cels.dens_mut().cycle_self_only();
	//cels.soma().cycle_self_only();

	cortex.area_mut(area_name).iinns.get_mut("iv_inhib").expect(emsg).cycle();

	for i in 0..100 {
		cels.learn();
	}

	cels.dens_mut().confab();

	cels.print_cel(cel_idx);
	
	println!("\nREGROWING... ");
	cels.regrow();

	cels.print_cel(cel_idx);

	println!("ALL CELLS: cell.syn_strengths[{:?}]: ", cel_syn_idz..(cel_syn_idz + per_cel));
	cmn::print_vec(&cels.dens_mut().syns.strengths.vec[..], 1, None, None, false);

	//check src_col_v_offs
	//check strengths
	//check offs and strs for other cells to make sure they're untouched

}*/



pub fn test_cycles(cortex: &mut Cortex, area_name: &str) {
	let emsg = "\ntests::hybrid::test_cycles()";
	
	/*cortex.area_mut(area_name).psal_mut().dens.syns.src_col_v_offs.set_all_to(0);
	cortex.area_mut(area_name).ptal_mut().dens.syns.src_col_v_offs.set_all_to(0);

	cortex.area_mut(area_name).psal_mut().dens.cycle();
	cortex.area_mut(area_name).ptal_mut().dens.cycle();*/

		//#####  TRY THIS OUT SOMETIME  #####
	//let pyrs_input_len = cortex.area_mut(area_name).ptal_mut().len();
	//let mut vec_pyrs = iter::repeat(0).take().collect();
	//input_czar::vec_band_512_fill(&mut vec_pyrs);
	//let pyr_axn_ranges = cortex.area_mut(area_name).layer_input_ranges("iii", cortex.area_mut(area_name).ptal_mut().dens.syns.den_kind());
	//write_to_axons(axn_range, vec1);
	let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.area_mut(area_name).dims.columns() as usize).collect();
	input_czar::sdr_stripes((cmn::SYNAPSE_SPAN_RHOMBAL_AREA as usize * 2), true, &mut vec1);

	println!("Primary Spatial Associative Layer...");
	let psal_name = cortex.area(area_name).psal().layer_name();
	cortex.write(area_name, psal_name, &vec1);
	test_syn_and_den_states(&mut cortex.area_mut(area_name).psal_mut().dens_mut());

	println!("Primary Temporal Associative Layer...");
	let ptal_name = cortex.area(area_name).ptal().layer_name();
	cortex.write(area_name, ptal_name, &vec1);
	test_syn_and_den_states(&mut cortex.area_mut(area_name).ptal_mut().dens_mut());
	test_pyr_preds(&mut cortex.area_mut(area_name).ptal_mut());
}

fn test_inhib(cortex: &mut Cortex) {

}
 

// TEST PYRAMIDAL CELLS 'PREDICTIVENESS' AKA: SOMA STATES
fn test_pyr_preds(pyrs: &mut PyramidalCellularLayer) {
	let emsg = "\ntests::hybrid::test_pyr_preds()";

	io::stdout().flush().unwrap();
	pyrs.dens_mut().states.set_all_to(0);

	let dens_per_tuft = pyrs.dens_mut().dims().per_tuft() as usize;
	println!("\n##### dens_per_tuft: {}", dens_per_tuft);
	//let dens_len = pyrs.dens_mut().states.len() as usize;	
	let pyrs_len = pyrs.soma().len() as usize;
	let den_tuft_len = pyrs_len * dens_per_tuft;

	// WRITE 255 TO THE DENDRITES CORRESPONDING TO THE FIRST AND LAST CELL
	// FOR THE FIRST TUFT ONLY
	for i in 0..dens_per_tuft {
		pyrs.dens_mut().states[i] = 255;
	}
	
	let last_cel_den_idz =  den_tuft_len - dens_per_tuft;

	for i in last_cel_den_idz..den_tuft_len {
		pyrs.dens_mut().states[i] = 255;
	}

	// WRITE THE DENDRITE STATES TO DEVICE
	pyrs.dens_mut().states.write();

	// CYCLE THE PYRAMIDAL CELL ONLY, WITHOUT CYCLING IT'S DENS OR SYNS (WHICH WOULD OVERWRITE THE ABOVE)
	pyrs.cycle_self_only();	
	
	// READ THE PYRAMIDAL CELL SOMA STATES (PREDS)
	pyrs.soma_mut().read();
	//pyrs.dens_mut().states.print_simple();
	//pyrs.soma_mut().print_simple();

	// TEST TO MAKE SURE THAT *ONLY* THE FIRST AND LAST CELL HAVE NON-ZERO VALUES
	for idx in 0..pyrs_len {
		//print!("([{}]:{})", i, pyrs.soma()[i]);
		if idx == 0 || idx == (pyrs_len - 1) {
			assert!(pyrs.soma()[idx] > 0);
		} else {
			assert!(pyrs.soma()[idx] == 0);
		}
	}

	println!("   test_pyr_preds(): {} ", PASS_STR);
}


fn test_syn_and_den_states(dens: &mut Dendrites) {
	let emsg = "\ntests::hybrid::test_syn_and_den_states()";

	io::stdout().flush().unwrap();
	dens.syns.src_col_v_offs.set_all_to(0);
	dens.cycle();

	let syns_per_tuft_l2: usize = dens.syns.dims().per_tuft_l2_left() as usize;
	let dens_per_tuft_l2: usize = dens.dims().per_tuft_l2_left() as usize;
	let cels_per_group: usize = cmn::SYNAPSE_SPAN_RHOMBAL_AREA as usize;
	let syns_per_group: usize = cels_per_group << syns_per_tuft_l2;
	let dens_per_group: usize = cels_per_group << dens_per_tuft_l2;
	let actv_group_thresh = syns_per_group / 4;
	//let den_actv_group_thresh = dens_per_group;

	//println!("Threshold: {}", actv_group_thresh);

	let mut cel_idz: usize = 0;
	let mut syn_states_ttl: usize = 0;
	let mut den_states_ttl: usize = 0;

	dens.confab();

	let mut test_failed: bool = false;

	loop {
		if (cel_idz + cels_per_group) > dens.dims().cells() as usize {
			break;
		}

		syn_states_ttl = 0;
		den_states_ttl = 0;

		let syn_idz = cel_idz << syns_per_tuft_l2;
		let den_idz = cel_idz << dens_per_tuft_l2;

		println!("\nsyn_idz: {}, syns_per_tuft: {}, syns_per_group: {}", syn_idz, 1 << syns_per_tuft_l2, syns_per_group);

		for syn_idx in syn_idz..(syn_idz + syns_per_group) {
			syn_states_ttl += (dens.syns.states[syn_idx] >> 7) as usize;
		}

		for den_idx in den_idz..(den_idz + dens_per_group) {
			den_states_ttl += (dens.states[den_idx]) as usize;
		}

		if (cel_idz & 512) == 0 {
			println!("   -Inactive-");

			if (syn_states_ttl < actv_group_thresh) || (den_states_ttl < actv_group_thresh) {
				test_failed = true;
			}

			/*assert!(syn_states_ttl > actv_group_thresh);
			assert!(den_states_ttl > actv_group_thresh);*/

		} else {
			println!("   -Active-");

			if (syn_states_ttl > actv_group_thresh) || (den_states_ttl > actv_group_thresh) {
				test_failed = true;
			}

			/*assert!(syn_states_ttl < actv_group_thresh);
			assert!(den_states_ttl < actv_group_thresh);*/

		}

		println!("SYN [{} - {}]: {}", cel_idz, (cel_idz + cels_per_group - 1), syn_states_ttl);
		print!("   DEN [{} - {}]: {}", cel_idz, (cel_idz + cels_per_group - 1), den_states_ttl);

		io::stdout().flush().unwrap();

		cel_idz += cels_per_group;
	}

	assert!(test_failed);

	println!("   test_syn_and_den_states(): {} ", PASS_STR);
}
