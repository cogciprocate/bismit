#![allow(non_snake_case)]
use std::ops::{ Range };
// use std::iter;
// use std::io::{ Write };
// use std::mem;
// use rand;

// use proto::*;


use ocl::{ /*self,*/ Envoy, EnvoyTest, /*WorkSize,*/ /*OclProgQueue, EnvoyDimensions,*/ OclNum };
// use super::{ TestBed };
use cortical_area::{ CorticalArea, CorticalAreaTest };
// use map::{ AreaMapTest };
use synapses::{ SynapsesTest };
use minicolumns::{ MinicolumnsTest };
// use dendrites::{ DendritesTest, /*DenCoords*/ };
// use axon_space::{ /*AxnCoords,*/ AxonSpaceTest };
// use cortex::{ Cortex };
use cmn::{ self, /*CelCoords,*/ DataCellLayer, DataCellLayerTest };
// use super::{ testbed };

/*=============================================================================
===============================================================================
=================================== UTILITY ===================================
===============================================================================
=============================================================================*/

bitflags! {
	#[derive(Debug)]
	flags PtalAlcoSwitches: u32 {
		const NONE				= 0b00000000,
		const ACTIVATE 			= 0b00000001,
		const LEARN				= 0b00000010,
		const CYCLE 			= 0b00000100,
		const OUTPUT 		 	= 0b00001000,
		const ALL 				= 0xFFFFFFFF,
	}
}


// ACTIVATE, LEARN, CYCLE, & OUTPUT
// pub fn al_cycle_depricate(area: &mut CorticalArea) {
// 	area.mcols().activate();
// 	area.ptal_mut().learn();
// 	area.ptal_mut().cycle();
// 	area.mcols().output();
// }

// ACTIVATE, LEARN, CYCLE, & OUTPUT
// pub fn ptal_alco(area: &mut CorticalArea, activ: bool, learn: bool, cycle: bool, output: bool) {
pub fn ptal_alco(area: &mut CorticalArea, switches: PtalAlcoSwitches, print: bool) {

	if switches.contains(ACTIVATE) {
		if print { printlny!("Activating..."); }
		area.mcols().activate();
	}

	if switches.contains(LEARN) {
		if print { printlny!("Learning..."); }
		area.ptal_mut().learn();
	}

	if switches.contains(CYCLE) {
		if print { printlny!("Cycling..."); }
		area.ptal_mut().cycle();
	}

	if switches.contains(OUTPUT) {
		if print { printlny!("Outputting..."); }
		area.mcols().output();
	}
}


pub fn confirm_syns(area: &mut CorticalArea, syn_range: &Range<usize>, state_neq: u8, 
		flag_set_eq: u8, strength_eq: i8) 
{
	for syn_idx in syn_range.clone() {
		area.ptal_mut().dens_mut().syns_mut().states.read();
		area.ptal_mut().dens_mut().syns_mut().flag_sets.read();
		area.ptal_mut().dens_mut().syns_mut().strengths.read();
		assert!(area.ptal_mut().dens_mut().syns_mut().states[syn_idx] != state_neq);
		assert!(area.ptal_mut().dens_mut().syns_mut().flag_sets[syn_idx] == flag_set_eq);
		assert!(area.ptal_mut().dens_mut().syns_mut().strengths[syn_idx] == strength_eq);
	}
}


pub fn assert_neq_range<T: OclNum>(env: &Envoy<T>, idx_range: &Range<usize>, val_neq: T) -> bool {
	for idx in idx_range.clone() {
		if env.read_idx_direct(idx) == val_neq { return false };
	}

	true
}


pub fn print_all(area: &mut CorticalArea, desc: &'static str) {
	//println!("\n - Confirm 1A - Activate");
	println!("{}", desc);	
	area.ptal_mut().print_all(true);
	// area.ptal_mut().dens_mut().syns_mut().print_all();
	area.print_aux();
	area.mcols_mut().print_all();
	area.print_axns();
}


pub fn compare_envoys<T: OclNum>(env1: &mut Envoy<T>, env2: &mut Envoy<T>) -> bool {	
	print!("\nVector comparison:\n");	
	assert!(env1.vec().len() == env2.vec().len());

	env1.read();
	env2.read();

	let mut failure = false;

	for i in 0..env1.vec().len() {
		let (e1_val, e2_val) = (env1.vec()[i], env2.vec()[i]);

		if e1_val != e2_val {
			failure = true;
			print!("{}", cmn::C_RED);
		} else {
			print!("{}", cmn::C_DEFAULT);
		}

		print!("[n:{}, v4:{}]{}", e1_val, e2_val, cmn::C_DEFAULT);
	}

	print!("\n");

	failure
}


// TEST_NEARBY(): Ensure that elements near a focal index are equal to a particular value.
//		- idz and idm (first and last elements) are also checked along with their nearby elements
// <<<<< TODO: THIS FUNCTION NEEDS SERIOUS STREAMLINING & OPTIMIZATION >>>>>
pub fn eval_others<T: OclNum>(env: &mut Envoy<T>, foc_idx: usize, other_val: T) {	// -> Result<(), &'static str>	
	// let mut checklist = Vec::new();
	let check_margin = 384;

	// assert!(env[foc_idx] == foc_val);

	// index[0]
	let idz = 0;
	// index[n]
	let idn = env.len();

	assert!(idn > 0);
	assert!(foc_idx < idn);

	env.read();

	if idn <= check_margin * 4 {
		// CHECK THE WHOLE LIST (except for foc_idx)
		unimplemented!();
	} else {
		let start_mrg = check_margin;
		let start_mrg_2 = check_margin * 2;

		let end_mrg = idn - check_margin;
		let end_mrg_2 = idn - (check_margin * 2);

		let foc_idx_l = if foc_idx >= start_mrg_2 {
			foc_idx - check_margin
		} else if foc_idx >= start_mrg {
			start_mrg
		} else {
			foc_idx
		};		

		let foc_idx_r = if foc_idx < end_mrg_2 {
			foc_idx + check_margin
		} else if foc_idx < end_mrg {
			end_mrg
		} else {
			foc_idx
		};

		let iter = (0usize..start_mrg) 			// start of list
			.chain(foc_idx_l..foc_idx)			// left of foc_idx
			.chain(foc_idx..foc_idx_r)			// right of foc_idx
			.chain(end_mrg..idn)				// end of list
			.filter(|&i| i != foc_idx);			// filter foc_idx itself from list

		for i in iter {
			// debug_assert!(i != foc_idx);
			// checklist.push(i);
			assert_eq!(env[i], other_val);
		}

		// println!("\n##### checklist: {:?} len: {}", checklist, checklist.len());
	}

}


#[test]
fn test_eval_others_UNIMPLEMENTED() {

}



// let foc_idx_l = match foc_idx {
		// 	idz...start_mrg => foc_idx,
		// 	start_mrg...start_mrg_2 => start_mrg,
		// 	_ => foc_idx - start_mrg,
		// };

		// let foc_idx_r = match foc_idx {
		// 	end_mrg...idn => foc_idx,
		// 	end_mrg_2...end_mrg => end_mrg,
		// 	_ => foc_idx + check_margin,
		// };
