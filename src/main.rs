#![allow(non_camel_case_types, dead_code, unstable, experimental, deprecated, unused_variable)]

mod test_1;
mod test_2;
mod ocl;
mod cl_h;
mod sense;
//mod space_req;

fn main() {
	//space_req::space_req()
	//test_1::run_kernel();
	sense::ascii_sense();
	test_2::run();
}


//#[link(name = "OpenCL")]
//#[cfg(target_os = "linux")]
//#[link_args = "-L$OPENCL_LIB -lOpenCL"]


/*

-===- MUSICAL NOTATION FOR SDRs (> SDR = Chord <) -===-
Note: One address in the SDR
Chord: All of the Notes


*/
