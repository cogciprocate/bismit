use std::ops::{ Range };
use std::iter;
use std::io::{ self, Write, Stdout };

use super::input_czar::{ self, InputCzar, InputVecKind };
//use cortex::Cortex;
use proto::*;
//use proto::areas::{ Protoareas, ProtoareasTrait };
//use proto::regions::{ Protoregions, Protoregion, ProtoregionKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites};
use pyramidals::{ PyramidalCellularLayer };
use cortex::{ self, Cortex };
use cmn;
use ocl::{ self, Envoy };


pub fn test_cycles(cortex: &mut Cortex) {
	
	/*cortex.cortical_area.ssts.get_mut("iv").expect("hybrid.rs").dens.syns.src_col_xy_offs.set_all_to(0);
	cortex.cortical_area.pyrs.get_mut("iii").expect("hybrid.rs").dens.syns.src_col_xy_offs.set_all_to(0);

	cortex.cortical_area.ssts.get_mut("iv").expect("hybrid.rs").dens.cycle();
	cortex.cortical_area.pyrs.get_mut("iii").expect("hybrid.rs").dens.cycle();*/

		//#####  TRY THIS OUT SOMETIME  #####
	//let pyrs_input_len = cortex.cortical_area.pyrs.get_mut("iii").expect("hybrid.rs").len();
	//let mut vec_pyrs = iter::repeat(0).take().collect();
	//input_czar::vec_band_512_fill(&mut vec_pyrs);
	//let pyr_axn_ranges = cortex.cortical_area.layer_input_ranges("iii", cortex.cortical_area.pyrs.get_mut("iii").expect("hybrid.rs").dens.syns.den_kind());
	//write_to_axons(axn_range, vec1);
	let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.cortical_area.dims.columns() as usize).collect();
	input_czar::sdr_stripes((cmn::SYNAPSE_SPAN_LIN as usize * 2), &mut vec1);
	
	print!("\nSpiny Stellate...");
	cortex.write_vec(0, "thal", &vec1);
	test_syn_and_den_states(&mut cortex.cortical_area.ssts.get_mut("iv").expect("hybrid.rs").dens);

	print!("\nPyramidal...");
	cortex.write_vec(0, "iii", &vec1);
	test_syn_and_den_states(&mut cortex.cortical_area.pyrs.get_mut("iii").expect("hybrid.rs").dens);
	test_pyr_preds(&mut cortex.cortical_area.pyrs.get_mut("iii").expect("hybrid.rs"));
}
 
// TEST THAT CORRECT RANGE OF CELLS IS BEING AFFECTED BY A SINGLE LEARN
// Just simulate a learning situation for a single pyramidal
fn test_learning_basic(cortex: &mut Cortex) {

}


fn test_pyr_preds(pyrs: &mut PyramidalCellularLayer) {
	io::stdout().flush().unwrap();
	pyrs.dens.states.set_all_to(0);

	let dens_per_cel = pyrs.dens.dims().per_cel().expect("hybrid.rs") as usize;
	let dens_len = pyrs.dens.states.len() as usize;

	for i in 0..dens_per_cel {
		pyrs.dens.states[i] = 255;
	}

	let last_cell_idz =  dens_len - dens_per_cel;

	for i in last_cell_idz..dens_len {
		pyrs.dens.states[i] = 255;
	}

	//pyrs.dens.states[50] = 255;
	pyrs.dens.states.write();
	pyrs.cycle_self_only();

	let pyrs_len = pyrs.preds.len() as usize;
	//pyrs.dens.states.print_simple();
	pyrs.preds.read();

	for i in 0..pyrs_len {
		//print!("([{}]:{})", i, pyrs.preds[i]);
		if i == 0 || i == (pyrs_len - 1) {
			assert!(pyrs.preds[i] > 0);
		} else {
			assert!(pyrs.preds[i] == 0);
		}
	}

	print!("\n   test_pyr_preds(): pass ");
}


fn test_syn_and_den_states(dens: &mut Dendrites) {
	io::stdout().flush().unwrap();
	dens.syns.src_col_xy_offs.set_all_to(0);
	dens.cycle();

	let syns_per_cel_l2: usize = dens.syns.dims().per_cel_l2_left().expect("hybrid.rs") as usize;
	let dens_per_cel_l2: usize = dens.dims().per_cel_l2_left().expect("hybrid.rs") as usize;
	let cels_per_group: usize = cmn::SYNAPSE_SPAN_LIN as usize;
	let syns_per_group: usize = cels_per_group << syns_per_cel_l2;
	let dens_per_group: usize = cels_per_group << dens_per_cel_l2;
	let actv_group_thresh = syns_per_group / 4;
	//let den_actv_group_thresh = dens_per_group;

	//print!("\nThreshold: {}", actv_group_thresh);

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

		let syn_idz = cel_idz << syns_per_cel_l2;
		let den_idz = cel_idz << dens_per_cel_l2;

		//println!("\nsyn_idz: {}, syns_per_cel: {}, syns_per_group: {}", syn_idz, 1 << syns_per_cel_l2, syns_per_group);

		for syn_idx in syn_idz..(syn_idz + syns_per_group) {
			syn_states_ttl += (dens.syns.states[syn_idx] >> 7) as usize;
		}

		for den_idx in den_idz..(den_idz + dens_per_group) {
			den_states_ttl += (dens.states[den_idx]) as usize;
		}

		if (cel_idz & 512) == 0 {
			//print!("\n   -Inactive-");

			if (syn_states_ttl < actv_group_thresh) || (den_states_ttl < actv_group_thresh) {
				test_failed = true;
			}

		} else {
			//print!("\n   -Active-");

			if (syn_states_ttl > actv_group_thresh) || (den_states_ttl > actv_group_thresh) {
				test_failed = true;
			}

		}

		//print!("\nSYN [{} - {}]: {}", cel_idz, (cel_idz + cels_per_group - 1), syn_states_ttl);
		//print!("   DEN [{} - {}]: {}", cel_idz, (cel_idz + cels_per_group - 1), den_states_ttl);

		//io::stdout().flush().unwrap();

		cel_idz += cels_per_group;
	}

	assert!(test_failed);

	print!("\n   test_1(): pass ");
}
