#![allow(non_snake_case, unused_imports)]
// use std::ops::{ Range };
// use std::iter;
// use std::io::{ Write };
// use std::mem;
// use rand;

use proto::*;

// use ocl;
use cortical_area::{ /*CorticalArea,*/ CorticalAreaTest };
use map::{ AreaMapTest };
use synapses::{ SynapsesTest };
use dendrites::{ DendritesTest, /*DenCoords*/ };
use axon_space::{ /*AxnCoords,*/ AxonSpaceTest };
// use cortex::{ Cortex };
use cmn::{ /*self,*/ /*CelCoords,*/ DataCellLayer, DataCellLayerTest };
use super::{ testbed, util };


const DENS_TEST_ITERATIONS: usize = 500;
const CELS_TEST_ITERATIONS: usize = 1;


#[test]
fn test_dens() {
	let mut cortex = testbed::fresh_cortex();
	let mut area = cortex.area_mut(testbed::PRIMARY_AREA_NAME);

	// area.ptal_mut().dens_mut().syns_mut().set_all_to_zero();
	area.ptal_mut().dens_mut().set_all_to_zero(true);

	// SET SOURCE SLICE TO UNUSED SLICE FOR EVERY SYNAPSE:
	let unused_slc_id = area.area_map().base_axn_slc_by_flag(layer::UNUSED_TESTING);
	area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_all_to(unused_slc_id);

	for i in 0..DENS_TEST_ITERATIONS {

		/*=============================================================================
		===================================== INIT ====================================
		=============================================================================*/

		// area.ptal_mut().dens_mut().set_all_to_zero(true);
		// area.ptal_mut().dens_mut().syns_mut().set_all_to_zero();
		// area.axns.states.set_all_to(0);

		let cel_coords = area.ptal_mut().rand_cel_coords();
		let den_coords = area.ptal_mut().dens_mut().rand_den_coords(&cel_coords);

		let den_dims = den_coords.dims().clone();

		// GET SOURCE SLICE TO USE TO SIMULATE INPUT:
		let cel_syn_range = den_coords.cel_syn_range_tftsec(area.ptal().dens().syns().syns_per_den_l2());
		let src_axn_slc_id = area.area_map().base_axn_slc_by_flag(layer::AFFERENT_INPUT);		

		// GET THE AXON INDEX CORRESPONDING TO OUR CELL AND SOURCE SLICE:
		let src_axn_idx = area.area_map().axn_idx(src_axn_slc_id, cel_coords.v_id, 
					0, cel_coords.u_id, 0).unwrap();

		// PRINT SOME DEBUG INFO IN CASE OF FAILURE:
		print!("\n");
		println!("{}", cel_coords);
		println!("{}", den_coords);
		println!("Axon Info: src_axn_slc_id: {}, src_axn_idx: {}", src_axn_slc_id, src_axn_idx);

		/*=============================================================================
		=========================== ACTIVATE AXON AND CYCLE ===========================
		=============================================================================*/

		// SET SOURCE SLICE TO AFF IN SLICE FOR OUR CELL'S SYNAPSES ONLY:
		area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(src_axn_slc_id, 
			cel_syn_range.clone());

		// WRITE INPUT:
		area.write_to_axon(128, src_axn_idx as usize);

		// CYCLE SYNS AND DENS:
		area.ptal_mut().dens_mut().cycle();	

		/*=============================================================================
		=================================== EVALUATE ==================================
		=============================================================================*/

		let mut result = vec![0];

		// CHECK EACH DENDRITE ON OUR TEST CELL:
		for den_idx in den_coords.cel_den_range_tftsec() {
			area.ptal().dens().states.read_direct(&mut result[..], den_idx as usize);
			let den_state = result[0];

			print!("\n");
			println!("dens.state[{}]: '{}'", den_coords.idx, den_state);
			// print_all(area, " - Dens - ");
			// print!("\n");

			// ENSURE THAT THE DENDRITE IS ACTIVE:
			assert!(den_state != 0, "Error: dendrite not activated on test cell.");
		}

		// <<<<< TODO: TEST OTHER RANDOM OR NEARBY CELLS >>>>>

		/*=============================================================================
		=================================== CLEAN UP ==================================
		=============================================================================*/

		area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(unused_slc_id, cel_syn_range);
		area.write_to_axon(0, src_axn_idx as usize);
	}

	// print!("\n");
	// panic!(" -- DEBUGGING -- ");
}



// TEST_CEL_TUFTS():
/*
		Test that input on each dendridic tuft is reaching the cell soma.
*/
#[test]
fn test_cel_tufts() {
	let mut cortex = testbed::init_test_cortex_2();
	let mut area = cortex.area_mut(testbed::PRIMARY_AREA_NAME);

	area.ptal_mut().dens_mut().set_all_to_zero(true);
	area.axns.states.set_all_to(0);

	// SET SOURCE SLICE TO UNUSED SLICE FOR EVERY SYNAPSE:
	let unused_slc_id = area.area_map().base_axn_slc_by_flag(layer::UNUSED_TESTING);
	area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_all_to(unused_slc_id);

	for i in 0..CELS_TEST_ITERATIONS {

		/*=============================================================================
		===================================== INIT ====================================
		=============================================================================*/

		let cel_coords = area.ptal_mut().rand_cel_coords();
		let den_coords = area.ptal_mut().dens_mut().rand_den_coords(&cel_coords);

		let den_dims = den_coords.dims().clone();

		// GET SOURCE SLICE TO USE TO SIMULATE INPUT:
		let cel_syn_range = den_coords.cel_syn_range_tftsec(area.ptal().dens().syns().syns_per_den_l2());
		let src_axn_slc_id = area.area_map().base_axn_slc_by_flag(layer::AFFERENT_INPUT);		

		// GET THE AXON INDEX CORRESPONDING TO OUR CELL AND SOURCE SLICE:
		let src_axn_idx = area.area_map().axn_idx(src_axn_slc_id, cel_coords.v_id, 
					0, cel_coords.u_id, 0).unwrap();

		// PRINT SOME DEBUG INFO IN CASE OF FAILURE:
		print!("\n");
		println!("{}", cel_coords);
		println!("{}", den_coords);
		println!("Axon Info: src_axn_slc_id: {}, src_axn_idx: {}", src_axn_slc_id, src_axn_idx);

		/*=============================================================================
		=========================== ACTIVATE AXON AND CYCLE ===========================
		=============================================================================*/

		// SET SOURCE SLICE TO AFF IN SLICE FOR OUR CELL'S SYNAPSES ONLY:
		area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(src_axn_slc_id, 
			cel_syn_range.clone());

		// WRITE INPUT:
		area.write_to_axon(128, src_axn_idx as usize);

		// CYCLE SYNS AND DENS:
		area.ptal_mut().dens_mut().cycle();	

		/*=============================================================================
		=================================== EVALUATE ==================================
		=============================================================================*/

		let mut den_state = vec![0];

		// DEBUG INFO (uncomment when needed):
		// print!("Synapse src_slc_ids: ");
		// area.ptal_mut().dens_mut().syns_mut().src_slc_ids.print(1, None, Some(cel_syn_range.clone()), true);
		// util::print_all(area, " -- TEST_CEL_TUFTS() -- ");
		// print!("\n");

		// CHECK EACH DENDRITE ON OUR TEST CELL:
		for den_idx in den_coords.cel_den_range_tftsec() {
			area.ptal().dens().states.read_direct(&mut den_state[0..1], den_idx as usize);
			// let den_state = result[0];

			println!("dens.state[{}]: '{}'", den_idx, den_state[0]);

			// ENSURE THAT THE DENDRITE IS ACTIVE:
			assert!(den_state[0] != 0, "Error: dendrite not activated on test cell.");
		}

		// <<<<< TODO: TEST OTHER RANDOM OR NEARBY CELLS >>>>>

		/*=============================================================================
		=================================== CLEAN UP ==================================
		=============================================================================*/

		area.ptal_mut().dens_mut().syns_mut().src_slc_ids.set_range_to(unused_slc_id, cel_syn_range);
		area.write_to_axon(0, src_axn_idx as usize);
	}

	// print!("\n");
	// panic!(" -- DEBUGGING -- ");
}
