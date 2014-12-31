#![allow(non_camel_case_types, dead_code, unstable, experimental, deprecated, unused_variables, unused_imports)]

extern crate microcosm;
extern crate time;

//mod test_1;
//mod test_2;
mod test_3;
//mod hello_world;
mod common;
mod cl_h;
mod ocl;
mod chord;
mod sense;
mod cortical_component;
mod neurons_column;
mod neurons_cell;
mod cortex;
mod sub_cortex;
mod test_miccos;
mod readback_test;
mod test_casting;

fn main() {
	println!("====== Bismit: main() running... ======");
	let time_start = time::get_time().sec;
	//test_1::run_kernel();
	//sense::ascii_sense();
	test_3::run();

	test_casting::run();
	
	//hello_world::run();
	//test_miccos::run();

	println!("====== Bismit: main() complete in: {} sec. ======", time::get_time().sec - time_start);
}


//#[link(name = "OpenCL")]
//#[cfg(target_os = "linux")]
//#[link_args = "-L$OPENCL_LIB -lOpenCL"]


/*

-===- MUSICAL NOTATION FOR SDRs (> SDR = Chord <) -===-
Note: One address in the SDR
Chord: All of the Notes


*/ 
