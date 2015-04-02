#![allow(non_camel_case_types, dead_code, unused_variables, 
	unused_assignments, unused_imports, unused_mut, unused_features)]

#![feature(core, rand, io, collections, path, libc, std_misc, old_path)]

extern crate num;
extern crate microcosm;
extern crate time;
#[macro_use]
extern crate rand;
#[macro_use] 
extern crate bitflags;


mod common;
mod cl_h;
mod ocl;
mod chord;
mod sense;
mod envoy;
//mod axon_space;
//mod syn_segs;
//mod column;
mod cells;
mod axons;
mod dendrites;
mod synapses;
mod columns;
mod peak_column;
mod pyramidal;
//mod cort_seg;
//mod thalamus;
mod cortex;
mod cortical_regions;
mod cortical_areas;
mod cortical_region_layer;
mod protocell;
//mod protocell;
//mod sub_cortex;
mod test_miccos;
//mod test_readback;
//mod test_3;
mod test_4;
mod tests;

use num::Integer;

fn main() {
	print!("================= Bismit: main() running... =================");
	let time_start = time::get_time();
	// test_1::run_kernel();
	// sense::ascii_sense();
	// test_3::run();
	// test_casting::run();
	// hello_world::run();

	
	test_4::test_cycle();
	//test_miccos::run();
	

	let time_complete = time::get_time() - time_start;
	println!("\n====== Bismit: main() complete in: {}.{} sec. ======", time_complete.num_seconds(), time_complete.num_milliseconds());
}


// #[link(name = "OpenCL")]
// #[cfg(target_os = "linux")]
// #[link_args = "-L$OPENCL_LIB -lOpenCL"]


/*

-===- MUSICAL NOTATION FOR SDRs (> SDR = Chord <) -===-
Note: One address in the SDR
Chord: All of the Notes


*/ 
