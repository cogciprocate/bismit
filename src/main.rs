#![allow(non_camel_case_types, dead_code, unused_variables, unused_mut, unused_assignments, unused_imports,)]
#![feature(vec_push_all, clone_from_slice, vec_resize, drain)]

extern crate num;
extern crate microcosm;
extern crate libc;
extern crate time;
//extern crate yaml_rust;

#[macro_use]
extern crate rand;
#[macro_use] 
extern crate bitflags;
#[macro_use] 
extern crate enum_primitive;


pub use cmn::input_source;

mod cmn;
//mod cl_h;
mod ocl;
//mod chord;
//mod sense;
//mod envoy;
//mod axon_space;
//mod syn_segs;
//mod column;
mod cortical_area;
mod axons;
mod dendrites;
mod synapses;
mod minicolumns;
mod iinn;

//mod iinn_new;

mod pyramidals;
mod spiny_stellates;
//mod cort_seg;
//mod thalamus;
mod cortex;
mod thalamus;
//mod protocell;
//mod sub_cortex;
//mod test_miccos;
//mod test_readback;
//mod test_3;
mod proto;
//mod energy;
mod encode;
mod sensory_filter;
//mod input_source;

//#[cfg(test)]
mod tests;

//use num::Integer;

fn main() {
	println!("================= Bismit: main() running... =================");
	let time_start = time::get_time();
	// test_1::run_kernel();
	// sense::ascii_sense();
	// test_3::run();
	// test_casting::run();
	// hello_world::run();

	if true {
		tests::interactive::run(0);
	} else {
		for i in 0..20 {
			tests::interactive::run(7000);
		}
	}
	
	//test_miccos::run();

	//test_reader();
	

	// <<<<< MOVE THIS TO CMN AND MAKE A FUNCTION FOR IT >>>>>
	let time_complete = time::get_time() - time_start;
	let t_sec = time_complete.num_seconds();
	let t_ms = time_complete.num_milliseconds() - (t_sec * 1000);
	println!("\n====== Bismit: main() complete in: {}.{} seconds ======", t_sec, t_ms);
}

// fn test_reader() {
// 	let mut mr = encode::IdxReader::new("data/train-images-idx3-ubyte");

// 	let sdr_side = 28usize;

// 	let mut vec1: Vec<u8> = std::iter::repeat(0).take(sdr_side * sdr_side).collect();

// 	println!("test_reader():\n");

// 	for i in 0..10 {
// 		mr.next(&mut vec1[..]);

// 		println!("Frame {}:\n", i);

// 		for pl in 0..sdr_side {
// 			for pc in 0..sdr_side {
// 				let idx = (pl * sdr_side) + pc;
// 				print!(" {:02X}", vec1[idx]);
// 			}
// 			println!("");
// 		}
// 	}

// }





// #[link(name = "OpenCL")]
// #[cfg(target_os = "linux")]
// #[link_args = "-L$OPENCL_LIB -lOpenCL"]

