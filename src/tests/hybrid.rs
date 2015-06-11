use std::ops::{ Range };
use std::iter;
use std::io::{ self, Write, Stdout };
use std::mem;
use rand;

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


pub static PASS_STR: &'static str = "\x1b[1;32mpass\x1b[0m";

/*	TEST THAT CORRECT RANGE OF CELLS IS BEING AFFECTED BY A SINGLE LEARN
		Simulate a learning situation for a single sst
			- (done) set axn ofs to 0
			- (done) set strs to 0
			- (done) choose 1 target cell, stimulate half of its synapses
			- make sure all other cells are still at 0 str
			- make sure that our target cell has the correct half of its synapse strs increased

			- (done)perform regrowth
			- check offs to ensure change
*/
pub fn test_learning(cortex: &mut Cortex) {
	_test_sst_learning(cortex, "iv");
	_test_pyr_learning(cortex, "iii");
}

fn _test_sst_learning(cortex: &mut Cortex, layer_name: &'static str) {
	let emsg = "tests::hybrid::test_sst_learning()";

	let cels = cortex.cortical_area.ssts.get_mut(layer_name).expect(emsg);

	//let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.cortical_area.dims.columns() as usize).collect();

	//let cel_syns = &mut ;
	cels.dens.syns.src_col_xy_offs.set_all_to(0);
	cels.dens.syns.strengths.set_all_to(0);
	cels.dens.syns.states.set_all_to(0);

	let first_half: bool = rand::random::<bool>();
	let per_cel = cels.dens.syns.dims().per_cel().expect(emsg) as usize;

	let cel_idx = rand::random::<usize>() & ((cels.dims().cells() as usize) - 1);
	let cel_syn_idz = cel_idx << cels.dens.syns.dims().per_cel_l2_left().expect(emsg);
	let cel_syn_tar_idz = cel_syn_idz + if first_half {0} else {per_cel >> 1};
	let cel_syn_tar_idn = cel_syn_tar_idz + (per_cel >> 1);
	
	println!("\n{}: cel_idx: {}, per_cel: {}, cel_syn_tar_idz: {}, cel_syn_tar_idn: {}", emsg, cel_idx, per_cel, cel_syn_tar_idz, cel_syn_tar_idn);

	for syn_idx in cel_syn_tar_idz..cel_syn_tar_idn {
		cels.dens.syns.states[syn_idx] = 255;
	}

	cels.dens.syns.states.write();
	cels.dens.cycle_self_only();
	//cels.soma().cycle_self_only();

	cortex.cortical_area.iinns.get_mut("iv_inhib").expect(&emsg).cycle();

	for i in 0..100 {
		cels.ltp();
	}

	cels.dens.confab();

	cels.print_cel(cel_idx);
	
	println!("\nREGROWING... ");
	cels.regrow();

	cels.print_cel(cel_idx);

	print!("\nALL CELLS: cell.syn_strengths[{:?}]: ", cel_syn_idz..(cel_syn_idz + per_cel));
	cmn::print_vec(&cels.dens.syns.strengths.vec[..], 1, None, None, false);

	//check src_col_xy_offs
	//check strengths
	//check offs and strs for other cells to make sure they're untouched

}

pub fn _test_pyr_learning(cortex: &mut Cortex, layer_name: &'static str) {
	let emsg = "tests::hybrid::test_pyr_learning()";

	let cels = cortex.cortical_area.pyrs.get_mut(layer_name).expect(emsg);

	//let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.cortical_area.dims.columns() as usize).collect();

	//let cel_syns = &mut ;
	cels.dens.syns.src_col_xy_offs.set_all_to(0);
	cels.dens.syns.strengths.set_all_to(0);
	cels.dens.syns.states.set_all_to(0);

	let first_half: bool = rand::random::<bool>();
	let per_cel = cels.dens.syns.dims().per_cel().expect(emsg) as usize;

	let cel_idx = rand::random::<usize>() & ((cels.dims().cells() as usize) - 1);
	let cel_syn_idz = cel_idx << cels.dens.syns.dims().per_cel_l2_left().expect(emsg);
	let cel_syn_tar_idz = cel_syn_idz + if first_half {0} else {per_cel >> 1};
	let cel_syn_tar_idn = cel_syn_tar_idz + (per_cel >> 1);
	
	println!("\n{}: cel_idx: {}, per_cel: {}, cel_syn_tar_idz: {}, cel_syn_tar_idn: {}", emsg, cel_idx, per_cel, cel_syn_tar_idz, cel_syn_tar_idn);

	for syn_idx in cel_syn_tar_idz..cel_syn_tar_idn {
		cels.dens.syns.states[syn_idx] = 255;
	}

	cels.dens.syns.states.write();
	cels.dens.cycle_self_only();
	//cels.soma().cycle_self_only();

	cortex.cortical_area.iinns.get_mut("iv_inhib").expect(&emsg).cycle();

	for i in 0..100 {
		cels.ltp();
	}

	cels.dens.confab();

	cels.print_cel(cel_idx);
	
	println!("\nREGROWING... ");
	cels.regrow();

	cels.print_cel(cel_idx);

	print!("\nALL CELLS: cell.syn_strengths[{:?}]: ", cel_syn_idz..(cel_syn_idz + per_cel));
	cmn::print_vec(&cels.dens.syns.strengths.vec[..], 1, None, None, false);

	//check src_col_xy_offs
	//check strengths
	//check offs and strs for other cells to make sure they're untouched

}



pub fn test_cycles(cortex: &mut Cortex) {
	let emsg = "tests::hybrid::test_cycles()";
	
	/*cortex.cortical_area.ssts.get_mut("iv").expect(emsg).dens.syns.src_col_xy_offs.set_all_to(0);
	cortex.cortical_area.pyrs.get_mut("iii").expect(emsg).dens.syns.src_col_xy_offs.set_all_to(0);

	cortex.cortical_area.ssts.get_mut("iv").expect(emsg).dens.cycle();
	cortex.cortical_area.pyrs.get_mut("iii").expect(emsg).dens.cycle();*/

		//#####  TRY THIS OUT SOMETIME  #####
	//let pyrs_input_len = cortex.cortical_area.pyrs.get_mut("iii").expect(emsg).len();
	//let mut vec_pyrs = iter::repeat(0).take().collect();
	//input_czar::vec_band_512_fill(&mut vec_pyrs);
	//let pyr_axn_ranges = cortex.cortical_area.layer_input_ranges("iii", cortex.cortical_area.pyrs.get_mut("iii").expect(emsg).dens.syns.den_kind());
	//write_to_axons(axn_range, vec1);
	let mut vec1: Vec<u8> = iter::repeat(0).take(cortex.cortical_area.dims.columns() as usize).collect();
	input_czar::sdr_stripes((cmn::SYNAPSE_SPAN_LIN as usize * 2), &mut vec1);
	
	print!("\nSpiny Stellate...");
	cortex.write_vec(0, "thal", &vec1);
	test_syn_and_den_states(&mut cortex.cortical_area.ssts.get_mut("iv").expect(emsg).dens);

	print!("\nPyramidal...");
	cortex.write_vec(0, "iii", &vec1);
	test_syn_and_den_states(&mut cortex.cortical_area.pyrs.get_mut("iii").expect(emsg).dens);
	test_pyr_preds(&mut cortex.cortical_area.pyrs.get_mut("iii").expect(emsg));
}

fn test_inhib(cortex: &mut Cortex) {

}
 

fn test_pyr_preds(pyrs: &mut PyramidalCellularLayer) {
	let emsg = "tests::hybrid::test_pyr_preds()";

	io::stdout().flush().unwrap();
	pyrs.dens.states.set_all_to(0);

	let dens_per_cel = pyrs.dens.dims().per_cel().expect(emsg) as usize;
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

	let pyrs_len = pyrs.soma().len() as usize;
	//pyrs.dens.states.print_simple();
	pyrs.soma_mut().read();

	for i in 0..pyrs_len {
		//print!("([{}]:{})", i, pyrs.soma()[i]);
		if i == 0 || i == (pyrs_len - 1) {
			assert!(pyrs.soma()[i] > 0);
		} else {
			assert!(pyrs.soma()[i] == 0);
		}
	}

	print!("\n   test_pyr_preds(): {} ", PASS_STR);
}


fn test_syn_and_den_states(dens: &mut Dendrites) {
	let emsg = "tests::hybrid::test_syn_and_den_states()";

	io::stdout().flush().unwrap();
	dens.syns.src_col_xy_offs.set_all_to(0);
	dens.cycle();

	let syns_per_cel_l2: usize = dens.syns.dims().per_cel_l2_left().expect(emsg) as usize;
	let dens_per_cel_l2: usize = dens.dims().per_cel_l2_left().expect(emsg) as usize;
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

	print!("\n   test_syn_and_den_states(): {} ", PASS_STR);
}
