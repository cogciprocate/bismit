// use std::ops::{ Range };
use std::iter;
use std::io::{ self, Write, /*Stdout*/ };
// use std::mem;
//use rand;

use interactive::{ /*self,*/ input_czar, /*InputCzar, InputKind*/ };
//use proto::*;
// use synapses::{ Synapses };
use dendrites::{ Dendrites };
use pyramidals::{ PyramidalLayer };
use cortex::{ /*self,*/ Cortex };
use cmn::{ self, DataCellLayer, DataCellLayerTest };
// use ocl::{ self, Envoy };


/*	HYBRID TESTS: Tests runnable in either an interactive or automated manner
		Useful for:
			- designing the test itself
			- troubleshooting test failures
			- diagnosing tangential issues not explicitly checked for

		- Additional hybrid tests in tests::learning
*/



 



pub fn test_cycles(cortex: &mut Cortex, area_name: &str) {
	let emsg = "\ntests::hybrid::test_cycles()";
	
	/*cortex.area_mut(area_name).psal_mut().dens.syns().src_col_v_offs.set_all_to(0);
	cortex.area_mut(area_name).ptal_mut().dens.syns().src_col_v_offs.set_all_to(0);

	cortex.area_mut(area_name).psal_mut().dens.cycle();
	cortex.area_mut(area_name).ptal_mut().dens.cycle();*/

		//#####  TRY THIS OUT SOMETIME  #####
	//let pyrs_input_len = cortex.area_mut(area_name).ptal_mut().len();
	//let mut vec_pyrs = iter::repeat(0).take().collect();
	//input_czar::vec_band_512_fill(&mut vec_pyrs);
	//let pyr_axn_ranges = cortex.area_mut(area_name).layer_input_ranges("iii", cortex.area_mut(area_name).ptal_mut().dens.syns().den_kind());
	//write_to_axons(axn_range, vec1);
	let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.area_mut(area_name).dims.columns() as usize).collect();
	input_czar::sdr_stripes((cmn::SYNAPSE_SPAN_RHOMBAL_AREA as usize * 2), true, &mut vec1);

	println!("Primary Spatial Associative Layer...");
	//let psal_name = cortex.area(area_name).psal().layer_name();
	//cortex.write(area_name, psal_name, &vec1);
	cortex.area(area_name).psal().soma().write_direct(&vec1, 0);
	test_syn_and_den_states(&mut cortex.area_mut(area_name).psal_mut().dens_mut());

	println!("Primary Temporal Associative Layer...");
	//let ptal_name = cortex.area(area_name).ptal().layer_name();
	//cortex.write(area_name, ptal_name, &vec1);
	cortex.area(area_name).ptal().soma().write_direct(&vec1, 0);
	test_syn_and_den_states(&mut cortex.area_mut(area_name).ptal_mut().dens_mut());
	test_pyr_preds(&mut cortex.area_mut(area_name).ptal_mut());
}


fn test_inhib(cortex: &mut Cortex) {

}
 

// TEST PYRAMIDAL CELLS 'PREDICTIVENESS' AKA: SOMA STATES
fn test_pyr_preds(pyrs: &mut PyramidalLayer) {
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

	println!("   test_pyr_preds(): {} ", super::PASS_STR);
}


fn test_syn_and_den_states(dens: &mut Dendrites) {
	let emsg = "\ntests::hybrid::test_syn_and_den_states()";

	io::stdout().flush().unwrap();
	dens.syns_mut().src_col_v_offs.set_all_to(0);
	dens.cycle();

	let syns_per_tuft_l2: usize = dens.syns().dims().per_tuft_l2_left() as usize;
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
			syn_states_ttl += (dens.syns().states[syn_idx] >> 7) as usize;
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

	println!("   test_syn_and_den_states(): {} ", super::PASS_STR);
}
