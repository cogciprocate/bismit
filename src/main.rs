#![allow(non_camel_case_types, dead_code, unused_variables, unused_mut, unused_assignments, unused_imports,)]

//#![feature(rand, io, collections, path, libc, std_misc, old_path)]
#![feature(collections)]

extern crate num;
extern crate microcosm;
extern crate libc;
extern crate time;

#[macro_use]
extern crate rand;
#[macro_use] 
extern crate bitflags;



mod cmn;
//mod cl_h;
mod ocl;
mod chord;
mod sense;
//mod envoy;
//mod axon_space;
//mod syn_segs;
//mod column;
mod region_cells;
mod axons;
mod dendrites;
mod synapses;
mod minicolumns;
mod peak_column;
mod pyramidals;
//mod cort_seg;
//mod thalamus;
mod cortex;
//mod protocell;
//mod sub_cortex;
//mod test_miccos;
//mod test_readback;
//mod test_3;
mod tests;
mod proto;
mod energy;


//use num::Integer;

fn main() {
	print!("================= Bismit: main() running... =================");
	let time_start = time::get_time();
	// test_1::run_kernel();
	// sense::ascii_sense();
	// test_3::run();
	// test_casting::run();
	// hello_world::run();

	
	tests::interactive::run();
	//test_miccos::run();
	

	let time_complete = time::get_time() - time_start;
	println!("\n====== Bismit: main() complete in: {}.{} sec. ======", time_complete.num_seconds(), time_complete.num_milliseconds());
}







// #[link(name = "OpenCL")]
// #[cfg(target_os = "linux")]
// #[link_args = "-L$OPENCL_LIB -lOpenCL"]

