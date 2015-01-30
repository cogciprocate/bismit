#![allow(non_camel_case_types, dead_code, unstable, deprecated, unused_variables, unused_assignments, unused_imports)]

extern crate microcosm;
extern crate time;

mod common;
mod cl_h;
mod ocl;
mod chord;
mod sense;
mod envoy;
//mod column;
mod cell;
//mod cortical_segment;
mod sensory_segment;
mod cortex;
mod sub_cortex;
mod test_miccos;
mod test_readback;
mod test_3;
mod test_4;

fn main() {
	println!("====== Bismit: main() running... ======");
	let time_start = time::get_time().sec;
	// test_1::run_kernel();
	// sense::ascii_sense();
	// test_3::run();

	// test_casting::run();
	
	// hello_world::run();
	
	test_4::test_cycle_dens();


	// test_miccos::run();

	println!("\n====== Bismit: main() complete in: {} sec. ======", time::get_time().sec - time_start);
}


// #[link(name = "OpenCL")]
// #[cfg(target_os = "linux")]
// #[link_args = "-L$OPENCL_LIB -lOpenCL"]


/*

-===- MUSICAL NOTATION FOR SDRs (> SDR = Chord <) -===-
Note: One address in the SDR
Chord: All of the Notes


*/ 
