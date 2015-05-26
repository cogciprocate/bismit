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
	
	/*cortex.cortical_area.mcols.dens.syns.src_col_xy_offs.set_all_to(0);
	cortex.cortical_area.pyrs.dens.syns.src_col_xy_offs.set_all_to(0);

	cortex.cortical_area.mcols.dens.cycle();
	cortex.cortical_area.pyrs.dens.cycle();*/

		//#####  TRY THIS OUT SOMETIME  #####
	//let pyrs_input_len = cortex.cortical_area.pyrs.len();
	//let mut vec_pyrs = iter::repeat(0).take().collect();
	//input_czar::vec_band_512_fill(&mut vec_pyrs);
	//let pyr_axn_ranges = cortex.cortical_area.layer_input_ranges("iii", cortex.cortical_area.pyrs.dens.syns.den_kind());
	//write_to_axons(axn_range, vec1);

	print!("\nSpiny Stellate:");
	test_1("thal", cortex);

	print!("\nPyramidal:");
	test_1("iii", cortex);
}


fn test_1(input_slice_name: &'static str, cortex: &mut Cortex) {
	let columns = cortex.cortical_area.dims.columns();
	let mut vec1 = iter::repeat(0).take(columns as usize).collect();

	input_czar::sdr_stripes((cmn::SYNAPSE_SPAN_LIN as usize * 2), &mut vec1);
	cortex.write_vec(0, input_slice_name, &vec1);

	let mut dens = &mut cortex.cortical_area.mcols.dens;

	dens.syns.src_col_xy_offs.set_all_to(0);
	dens.cycle();
	check_syns_and_dens(&mut dens);
}


fn check_syns_and_dens(dens: &mut Dendrites) {
	let syns_per_cel_l2: usize = dens.syns.dims.per_cel_l2_left().unwrap() as usize;
	let dens_per_cel_l2: usize = dens.dims.per_cel_l2_left().unwrap() as usize;
	let cels_per_group: usize = cmn::SYNAPSE_SPAN_LIN as usize;
	let syns_per_group: usize = cels_per_group << syns_per_cel_l2;
	let dens_per_group: usize = cels_per_group << dens_per_cel_l2;
	let syn_actv_group_thresh = syns_per_group / 4;
	let den_actv_group_thresh = dens_per_group;


	let mut cel_idz: usize = 0;
	let mut states_ttl: usize = 0;

	dens.confab();

	loop {
		if (cel_idz + cels_per_group) > dens.dims.cells() as usize {
			break;
		}

		states_ttl = 0;

		let syn_idz = cel_idz << syns_per_cel_l2;
		let den_idz = cel_idz << dens_per_cel_l2;

		//println!("\nsyn_idz: {}, syns_per_cel: {}, syns_per_group: {}", syn_idz, 1 << syns_per_cel_l2, syns_per_group);

		for syn_idx in syn_idz..(syn_idz + syns_per_group) {
			states_ttl += (dens.syns.states[syn_idx] >> 7) as usize;
		}

		if (cel_idz & 512) == 0 {
			print!("\n   -Inactive-");
			assert!(states_ttl < syn_actv_group_thresh);
		} else {
			print!("\n   -Active-");
			assert!(states_ttl > syn_actv_group_thresh);
		}


		print!("\nSYN [{} - {}], states_ttl: {}({})", cel_idz, (cel_idz + cels_per_group - 1), states_ttl, syn_actv_group_thresh);

		states_ttl = 0;

		for den_idx in den_idz..(den_idz + dens_per_group) {
			states_ttl += (dens.states[den_idx]) as usize;
		}

		print!("\nDEN [{} - {}], states_ttl: {}({})", cel_idz, (cel_idz + cels_per_group - 1), states_ttl, den_actv_group_thresh);


		cel_idz += cels_per_group;
	}
}
