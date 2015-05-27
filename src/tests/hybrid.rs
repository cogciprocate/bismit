use std::ops::{ Range };
use std::iter;

use super::input_czar::{ self, InputCzar, InputVecKind };
//use cortex::Cortex;
use proto::*;
//use proto::areas::{ Protoareas, ProtoareasTrait };
//use proto::regions::{ Protoregions, Protoregion, ProtoregionKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites};
use cortex::{ self, Cortex };
use cmn;
use ocl;


pub fn test_cycles(cortex: &mut Cortex) {
	
	/*cortex.cortical_area.ssts.get_mut("iv").unwrap().dens.syns.src_col_xy_offs.set_all_to(0);
	cortex.cortical_area.pyrs.get_mut("iii").unwrap().dens.syns.src_col_xy_offs.set_all_to(0);

	cortex.cortical_area.ssts.get_mut("iv").unwrap().dens.cycle();
	cortex.cortical_area.pyrs.get_mut("iii").unwrap().dens.cycle();*/

		//#####  TRY THIS OUT SOMETIME  #####
	//let pyrs_input_len = cortex.cortical_area.pyrs.get_mut("iii").unwrap().len();
	//let mut vec_pyrs = iter::repeat(0).take().collect();
	//input_czar::vec_band_512_fill(&mut vec_pyrs);
	//let pyr_axn_ranges = cortex.cortical_area.layer_input_ranges("iii", cortex.cortical_area.pyrs.get_mut("iii").unwrap().dens.syns.den_kind());
	//write_to_axons(axn_range, vec1);
	let mut vec1 = iter::repeat(0).take(cortex.cortical_area.dims.columns() as usize).collect();
	input_czar::sdr_stripes((cmn::SYNAPSE_SPAN_LIN as usize * 2), &mut vec1);

	print!("\nSpiny Stellate:");
	cortex.write_vec(0, "thal", &vec1);
	test_1(&mut cortex.cortical_area.ssts.get_mut("iv").unwrap().dens);

	print!("\nPyramidal:");
	cortex.write_vec(0, "iii", &vec1);
	test_1(&mut cortex.cortical_area.pyrs.get_mut("iii").unwrap().dens);
}


fn test_1(mut dens: &mut Dendrites) {
	dens.syns.src_col_xy_offs.set_all_to(0);
	dens.cycle();
	check_syns_and_dens(&mut dens);
}


fn check_syns_and_dens(dens: &mut Dendrites) {
	let syns_per_cel_l2: usize = dens.syns.dims().per_cel_l2_left().unwrap() as usize;
	let dens_per_cel_l2: usize = dens.dims().per_cel_l2_left().unwrap() as usize;
	let cels_per_group: usize = cmn::SYNAPSE_SPAN_LIN as usize;
	let syns_per_group: usize = cels_per_group << syns_per_cel_l2;
	let dens_per_group: usize = cels_per_group << dens_per_cel_l2;
	let syn_actv_group_thresh = syns_per_group / 4;
	let den_actv_group_thresh = dens_per_group;


	let mut cel_idz: usize = 0;
	let mut syn_states_ttl: usize = 0;
	let mut den_states_ttl: usize = 0;

	dens.confab();

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
			print!("\n   -Inactive-");
			assert!(syn_states_ttl < syn_actv_group_thresh);
			assert!(den_states_ttl < den_actv_group_thresh);
		} else {
			print!("\n   -Active-");
			assert!(syn_states_ttl > syn_actv_group_thresh);
			assert!(den_states_ttl > den_actv_group_thresh);
		}


		print!("\nSYN [{} - {}], syn_states_ttl: {}({})", cel_idz, (cel_idz + cels_per_group - 1), syn_states_ttl, syn_actv_group_thresh);
		print!("\nDEN [{} - {}], den_states_ttl: {}({})", cel_idz, (cel_idz + cels_per_group - 1), den_states_ttl, den_actv_group_thresh);


		cel_idz += cels_per_group;
	}
}
