#![allow(non_camel_case_types, dead_code, unstable, deprecated, unused_variables, unused_imports)]

extern crate microcosm;
//extern crate time;

//mod test_3;
//mod test4;
mod common;
mod cl_h;
mod ocl;
mod chord;
mod sense;
mod cortical_component;
mod column_neurons;
mod cell_neurons;
mod sensory_segment;
mod cortex;
mod sub_cortex;
mod test_miccos;
mod readback_test;

fn main() {
	println!("====== Bismit: main() running... ======");
	let time_start = 0u32;		// time::get_time().sec;
	//test_1::run_kernel();
	//sense::ascii_sense();
	//test_3::run();

	//test_casting::run();
	
	//hello_world::run();
	
	//test4::readback_test();


	test_miccos::run();

	println!("====== Bismit: main() complete in: {} sec. ======", 0u32 - time_start);
	// println!("====== Bismit: main() complete in: {} sec. ======", time::get_time().sec - time_start);
}


//#[link(name = "OpenCL")]
//#[cfg(target_os = "linux")]
//#[link_args = "-L$OPENCL_LIB -lOpenCL"]


/*

-===- MUSICAL NOTATION FOR SDRs (> SDR = Chord <) -===-
Note: One address in the SDR
Chord: All of the Notes


*/ 
