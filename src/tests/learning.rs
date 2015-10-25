#![allow(non_snake_case)]
// use std::ops::{ Range };
// use std::iter;
// use std::io::{ Write };
// use std::mem;
// use rand;

use ocl::{ EnvoyTest };
use proto::{ layer };
// use ocl;
use cortical_area::{ CorticalArea, CorticalAreaTest };
// use map::{ AreaMapTest };
use synapses::{ SynapsesTest };
// use dendrites::{ DendritesTest, /*DenCoords*/ };
use axon_space::{ AxonSpaceTest };
// use cortex::{ Cortex };
use cmn::{ self, /*CelCoords,*/ DataCellLayer, DataCellLayerTest };
use super::{ util };


//=============================================================================
//=============================================================================
//================================== TESTS ====================================
//=============================================================================
//=============================================================================




// TEST_CEL_SYN_LEARNING(): 
// Set up conditions in which a synapse should learn and its neighbors should not.
/* 			
		Choose a pyramidal cell.
		Activate the column axon (mcol/sst) for that cell's column.

		Choose a synapse from that cell.		
		
			Check for safety/validity.	
			Confirm that the pyramidal cell has the correct flags (none):
				CEL_PREV_CONCRETE_FLAG: u8 		= 0b10000000;	// 128	(0x80)
				CEL_BEST_IN_COL_FLAG: u8 		= 0b01000000;	// 64	(0x40)
				CEL_PREV_STP_FLAG: u8 			= 0b00100000;	// 32	(0x20)
				CEL_PREV_FUZZY_FLAG: u8			= 0b00010000;	// 16	(0x10)
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


	NOTES:
		- It is assumed that all axons, dendrites, and synapses for our cortical area are completely zeroed.
		- 'slc' and 'slc_id' are synonymous
		- unused_slc_id is actually unused! Remove?
*/
pub fn _test_pyr_learning(area: &mut CorticalArea, unused_slc_id: u8, prx_src_slc: u8,
			fake_neighbor_slc: u8, iter: usize) 
{
	let aff_out_slcs = area.area_map().base_axn_slc_ids_by_flag(layer::AFFERENT_OUTPUT);
	assert!(aff_out_slcs.len() == 1);
	let aff_out_slc = aff_out_slcs[0];

	

	//=============================================================================
	//=================================== INIT ====================================
	//=============================================================================

	// Get a random cell and a random synapse on that cell:
	let cel_coords = area.ptal_mut().rand_cel_coords();

	// Base slice for primary temporal pyramidals of our layer:
	let ptal_axn_slc_idz = area.ptal_mut().base_axn_slc();
	// assert!(ptal_axn_slc_idz == cel_coords.slc_id_lyr, "cel_coords axon slice mismatch");
	assert_eq!(ptal_axn_slc_idz, cel_coords.axn_slc_id - cel_coords.slc_id_lyr);
	let syn_coords = area.ptal_mut().dens_mut().syns_mut()
		.rand_syn_coords(&cel_coords);

	// Our cell's proximal source axon (the column spatial axon):
	let prx_src_axn_idx = area.area_map().axn_idz(prx_src_slc) + cel_coords.col_id();

	// Our cell's axon:
	let cel_axn_idx = cel_coords.cel_axn_idx(area.area_map());

	// <<<<< TODO: CHECK THAT THIS CEL_AXN_IDX FALLS WITHIN THE CORRECT RANGE FOR
	// PYRAMIDAL AXON INDEXES FOR THE LAYER >>>>>
	// 
	// !@$%!@#$!@#$!@#!@#$!    THIS IS CURRENTLY WRONG!     !$@!#$!@#$@#!$!@!$@#$
// 
// 						PROBABLY FIXED
	//

	// Our cell's COLUMN output axon:
	let aff_out_axn_idx = area.area_map().axn_idz(aff_out_slc) + cel_coords.col_id();

	// A random, nearby axon for our cell to use as a distal source axon:
	let (fn_v_ofs, fn_u_ofs, fn_col_id, fake_neighbor_axn_idx) = area.rand_safe_src_axn(&cel_coords, fake_neighbor_slc);

	//================================ SYN RANGE ==================================
	// A random dendrite id on the cell tuft:
	// let den_id_cel_tft = 
	// The synapse range for entire tuft in which our random synapse resides:
	let syn_range = syn_coords.cel_syn_range_tftsec();

	// The first half of the synapses on our tuft (UNUSED):
	let syn_range_first_half = syn_range.start..(syn_range.start + syn_range.len() / 2);

	// The second half of the synapses on our tuft:
	let syn_range_second_half = (syn_range.start + syn_range.len() / 2)
		..(syn_range.start + syn_range.len());

	// The synapse count for our cell's entire layer (all slices, cells, and tufts):
	let syn_range_all = 0..area.ptal_mut().dens_mut().syns_mut().states.len();

	// Set the sources for the synapses on the second half of our chosen tuft to our preselected nearby axon:
	// <<<<< TODO: IMPLEMENT THIS (for efficiency): >>>>>
	// 		area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(unused_slc_id, den_syn_range);
	for syn_idx in syn_range_second_half.clone() {
		area.ptal_mut().dens_mut().syns_mut().set_src_offs(fn_v_ofs, fn_u_ofs, syn_idx as usize);
		area.ptal_mut().dens_mut().syns_mut().set_src_slc(fake_neighbor_slc, syn_idx as usize);
	}

	// PRINT ALL THE THINGS!:
	let syn_val = area.ptal_mut().dens_mut().syns_mut().syn_state(syn_coords.idx);
	let fake_neighbor_axn_val = area.axn_state(fake_neighbor_axn_idx as usize);

	
	println!("\nDEBUG INFO - PRINT ALL THE THINGS!: \n\
		{mt}[prx_src]: prx_src_axn_idx (prx_src_slc: {}, col_id: {}): {} \n\
		{mt}[dst_src]: fake_neighbor_axn_idx (''_slc: {}, col_id: {}): {}, \n\
		{mt}[cel_axn]: cel_axn_idx (cel_coords.axn_slc_id: {}, col_id: {}): {} \n\
		{mt}[col_out]: aff_out_axn_idx (aff_out_slc: {}, col_id: {}): {}, \n\n\
		\
		{mt}fn_v_ofs: {}, fn_u_ofs: {}, \n\
		{mt}fake_neighbor_axn_val: {}, syn_val: {}, syn_tft_range: {:?}, syn_active_range: {:?}, \n\
		{mt}syn_coords: {}",		

		prx_src_slc, cel_coords.col_id(), prx_src_axn_idx,
		fake_neighbor_slc, fn_col_id, fake_neighbor_axn_idx, 
		cel_coords.axn_slc_id, cel_coords.col_id(), cel_axn_idx,
		aff_out_slc, cel_coords.col_id(), aff_out_axn_idx, 

		fn_v_ofs, fn_u_ofs,
		fake_neighbor_axn_val, syn_val, syn_range, syn_range_second_half, 
		syn_coords,
		mt = cmn::MT);




	// This and every other util::print_all() is very expensive:
	util::print_all(area, "\n - Confirm Init - ");

	//=============================================================================
	//===================================== 0 =====================================
	//=============================================================================
	println!("\n ========================== 0: Initialization ========================== \n");

	// Activate distal source axon:
	printlny!("Activating distal source axon: [{}]...", fake_neighbor_axn_idx); 
	area.activate_axon(fake_neighbor_axn_idx);

	// Cycle and output for column:
	printlny!("Cycling and outputting: calling area.ptal_mut().cycle() and area.mcols().output()..."); 
	area.ptal_mut().cycle();
	area.mcols().output();


	util::print_all(area, "\n - Confirm 0 - ");
	util::confirm_syns(area, &syn_range_second_half, 0, 0, 0);

	print!("\n");
	// <<<<< TODO: VERIFY THAT MCOLS.OUTPUT() AXON IS ACTIVE (and print it's idx) >>>>>
	// <<<<< TODO: CHECK CELL AXON (should be zero here and active on next step) >>>>>

	// Ensure key axons are active:
	assert!(area.read_from_axon(fake_neighbor_axn_idx) > 0);
	printlny!("Pyramidal cell fake neighbor axon is correctly active.");

	// Ensure minicolumn is predictive as a result of pyramidal activity:
	assert!(area.mcols().flag_sets.read_idx_direct(cel_coords.col_id() as usize) == cmn::MCOL_IS_VATIC_FLAG);
	printlny!("Minicolumn is correctly vatic (predictive).");

	//=============================================================================
	//==================================== 1A =====================================
	//=============================================================================
	println!("\n ========================== 1: Premonition ========================== ");
	println!(" ============================== 1A =============================== \n");


	// ACTIVATE, & LEARN ONLY
	printlny!("Activating pyramidals: calling area.mcols().activate()...");
	area.mcols().activate();
	// area.ptal_mut().learn();
	// area.ptal_mut().cycle();
	// area.mcols().output();

	util::print_all(area, "\n - Confirm 1A - ");
	util::confirm_syns(area, &syn_range_second_half, 0, 0, 0);

	print!("\n");

	// Ensure our cell is flagged best in (mini) column:
	assert!(area.ptal().flag_sets.read_idx_direct(cel_coords.idx() as usize) == cmn::CEL_BEST_IN_COL_FLAG);
	printlny!("Our cell is correctly flagged best in column.");

	//=============================================================================
	//================================= 1B ===================================
	//=============================================================================
	println!("\n ========================== 1B ========================== \n");

	// ACTIVATE, LEARN &, CYCLE
	printlny!("Learning: calling area.ptal_mut().learn()...");
	// area.mcols().activate();
	area.ptal_mut().learn();
	// area.ptal_mut().cycle();
	// area.mcols().output();

	util::print_all(area, "\n - Confirm 1B - ");
	util::confirm_syns(area, &syn_range_second_half, 0, 0, 0);

	print!("\n");

	// <<<<< TODO: Ensure our cells synapses have not learned anything: >>>>>

	//=============================================================================
	//=================================== 1C ===================================
	//=============================================================================
	println!("\n ========================== 1C ========================== \n");

	// ACTIVATE PTAL SYNAPSE SOURCE AXON
	// printlny!("Activating distal source axon: [{}]...", fake_neighbor_axn_idx);
	area.activate_axon(fake_neighbor_axn_idx);

	// ACTIVATE, LEARN &, CYCLE
	printlny!("Cycling and outputting: calling area.ptal_mut().cycle() and area.mcols().output()..."); 
	// area.mcols().activate();
	// area.ptal_mut().learn();
	area.ptal_mut().cycle();
	area.mcols().output();

	util::print_all(area, "\n - Confirm 1C - ");
	util::confirm_syns(area, &syn_range_second_half, 0, 0, 0);

	print!("\n");

	//=============================================================================
	//=================================== 2A ===================================
	//=============================================================================
	println!("\n ========================== 2: Vindication ========================== ");
	println!(" ============================== 2A =============================== \n");

	// PRINT SLICE MAP FOR REFERENCE
	// area.area_map().print_slc_map();

	// ACTIVATE COLUMN PSAL AXON
	printlny!("{cy}Activating proximal source axon: [{}]...{cd}", 
		prx_src_axn_idx, cy = cmn::C_YEL, cd = cmn::C_DEFAULT);
	area.activate_axon(prx_src_axn_idx);
	// ACTIVATE PTAL SYNAPSE SOURCE AXON
	// area.activate_axon(fake_neighbor_axn_idx as usize);

	// ACTIVATE, LEARN &, CYCLE
	printlny!("Activating pyramidals: calling area.mcols().activate()...");
	area.mcols().activate();
	// area.ptal_mut().learn();
	// area.ptal_mut().cycle();
	// area.mcols().output();

	util::print_all(area, "\n - Confirm 2A - ");
	// util::confirm_syns(area, &syn_range_second_half, 0, 0, 0);

	print!("\n");

	// ##### ADD ME: assert!(THE PYRAMIDAL OUTPUT AXON (NOT SOMA) IS ACTIVE)
	// THIS IS CURRENTLY NOT ACTIVATING!!!

	// MOVED THIS FROM 1B -- PROBABLY WAS IN WRONG SPOT
	// printlny!("\nConfirming flag sets...");
	// assert!(util::assert_neq_range(&area.ptal().dens().syns().flag_sets, &syn_range_second_half, 0));

	//=============================================================================
	//=================================== 2B ===================================
	//=============================================================================
	println!("\n ========================== 2B ========================== \n");

	// ACTIVATE COLUMN PSAL AXON
	//area.activate_axon(prx_src_axn_idx as usize);
	// ACTIVATE PTAL SYNAPSE SOURCE AXON
	//area.activate_axon(fake_neighbor_axn_idx as usize);

	// ACTIVATE, LEARN &, CYCLE
	// area.mcols().activate();
	printlny!("Learning: calling area.ptal_mut().learn()...");
	area.ptal_mut().learn();
	// area.ptal_mut().cycle();
	// area.mcols().output();

	util::print_all(area, "\n - Confirm 2B - ");
	// util::confirm_syns(area, &syn_range_first_half, 0, 0, 0);

	print!("\n");

	// <<<<< TODO: assert!(chosen-half of syns are +1, others are -1) >>>>>
	// CURRENTLY: indexes are a mess

	//=============================================================================
	//=================================== 2C ===================================
	//=============================================================================
	println!("\n ========================== 2C ========================== \n");

	// ACTIVATE COLUMN PSAL AXON
	//area.activate_axon(prx_src_axn_idx as usize);
	// ACTIVATE PTAL SYNAPSE SOURCE AXON
	//area.activate_axon(fake_neighbor_axn_idx as usize);

	// ACTIVATE, LEARN &, CYCLE
	printlny!("Cycling and outputting: calling area.ptal_mut().cycle() and area.mcols().output()..."); 
	// area.mcols().activate();
	// area.ptal_mut().learn();
	area.ptal_mut().cycle();
	area.mcols().output();

	util::print_all(area, "\n - Confirm 2C - ");

	print!("\n");

	//=============================================================================
	//=================================== 3 ===================================
	//=============================================================================
	println!("\n ========================== 3: Termination ========================== ");
	println!(" =============================== 3 =============================== \n");

	printlny!("Deactivating all axons...");
	// ZERO COLUMN PSAL AXON
	area.deactivate_axon(prx_src_axn_idx);
	// ZERO PTAL SYNAPSE SOURCE AXON
	area.deactivate_axon(fake_neighbor_axn_idx);

	// ACTIVATE, LEARN &, CYCLE
	printlny!("Activating, learning, cycling, and outputting...");
	util::al_cycle(area);

	util::print_all(area, "\n - Confirm 3 - ");

	print!("\n");

	//=============================================================================
	//=================================== CLEAN UP ===================================
	//=============================================================================
	println!("\n ========================== 4: Clean-up-ification ========================== \n");

	printlny!("Cleaning up...");
	area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(unused_slc_id, syn_range.clone());
	area.ptal_mut().dens_mut().syns_mut().src_col_v_offs.set_range_to(0, syn_range.clone());
	area.ptal_mut().dens_mut().syns_mut().src_col_u_offs.set_range_to(0, syn_range.clone());

	print!("\n");
	panic!(" -- DEBUGGING -- ");
}


// pub const CEL_PREV_CONCRETE_FLAG: u8 		= 128	(0x80)
// pub const CEL_BEST_IN_COL_FLAG: u8 			= 64	(0x40)
// pub const CEL_PREV_STP_FLAG: u8 				= 32	(0x20)
// pub const CEL_PREV_FUZZY_FLAG: u8			= 16	(0x10)

// pub const SYN_STP_FLAG: u8					= 1;
// pub const SYN_STD_FLAG: u8					= 2;
// pub const SYN_CONCRETE_FLAG: u8				= 8;

