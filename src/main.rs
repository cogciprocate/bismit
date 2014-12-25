#![allow(non_camel_case_types, dead_code, unstable, experimental, deprecated, unused_variables)]

extern crate microcosm;

//mod test_1;
//mod test_2;
//mod test_3;
mod hello_world;
mod ocl;
mod cl_h;
mod sense;
mod cortex;
mod sub_cortex;
mod chord;
mod common;
mod test_miccos;

fn main() {
	//test_1::run_kernel();
	//sense::ascii_sense();
	//test_3::run();
	//hello_world::run();
	test_miccos::run();
}


//#[link(name = "OpenCL")]
//#[cfg(target_os = "linux")]
//#[link_args = "-L$OPENCL_LIB -lOpenCL"]


/*

-===- MUSICAL NOTATION FOR SDRs (> SDR = Chord <) -===-
Note: One address in the SDR
Chord: All of the Notes


*/ 
