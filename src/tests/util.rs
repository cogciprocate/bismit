#![allow(non_snake_case)]
use std::ops::{ Range };
// use std::iter;
// use std::io::{ Write };
// use std::mem;
// use rand;

// use proto::*;


use ocl::{ /*self,*/ Envoy, /*WorkSize,*/ /*OclProgQueue, EnvoyDimensions,*/ OclNum };
// use super::{ TestBed };
use cortical_area::{ CorticalArea, CorticalAreaTest };
// use map::{ AreaMapTest };
use synapses::{ SynapsesTest };
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




// CYCLE, ACTIVATE, & LEARN
pub fn al_cycle(area: &mut CorticalArea) {
	area.mcols.activate();
	area.ptal_mut().learn();
	area.ptal_mut().cycle();
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


pub fn print_all(area: &mut CorticalArea, desc: &'static str) {
	//println!("\n - Confirm 1A - Activate");
	println!("\n{}", desc);
	area.ptal_mut().print_all(true);
	// area.ptal_mut().dens_mut().syns_mut().print_all();
	area.print_aux();
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
