use std::num::{ Int };
use std::default::{ Default }; 
use std::fmt::String;

use ocl;

pub static C_DEFAULT: &'static str = "\x1b[0m";
pub static C_RED: &'static str = "\x1b[91m";
pub static C_CYA: &'static str = "\x1b[36m";
pub static C_GRE: &'static str = "\x1b[32m";
pub static C_BLU: &'static str = "\x1b[94m";
pub static C_MAG: &'static str = "\x1b[95m";
pub static C_PUR: &'static str = "\x1b[35m";
pub static C_ORA: &'static str = "\x1b[33m";
pub static C_YEL: &'static str = "\x1b[93m";

pub static KERNELS_FILE_NAME: &'static str = "bismit.cl";

pub const CORTICAL_SEGMENTS_TOTAL: usize = 2;
pub const SENSORY_SEGMENTS_TOTAL: usize = 2;
pub const MOTOR_SEGMENTS_TOTAL: usize = 1;

pub const HYPERCOLUMNS_PER_SEGMENT: usize = 16;		// appears to cause lots of delay... 256 is slow

pub const SYNAPSE_WEIGHT_ZERO: u8 = 16;
pub const SYNAPSE_WEIGHT_INITIAL_DEVIATION: u8 = 3;
pub const DENDRITE_INITIAL_THRESHOLD: u8 = 16;

pub const COLUMNS_PER_HYPERCOLUMN: usize = 64;
//pub const COLUMNS_PER_ADDRESS_BLOCK: usize = 16u;
pub const CELLS_PER_COLUMN: usize = 16;
pub const DENDRITES_PER_NEURON: usize = 16;
pub const SYNAPSES_PER_DENDRITE: usize = 16;
pub const AXONS_PER_NEURON: usize = DENDRITES_PER_NEURON * SYNAPSES_PER_DENDRITE;
pub const SYNAPSES_PER_NEURON: usize = SYNAPSES_PER_DENDRITE * DENDRITES_PER_NEURON;

pub const COLUMNS_PER_SEGMENT: usize = COLUMNS_PER_HYPERCOLUMN * HYPERCOLUMNS_PER_SEGMENT;
pub const COLUMN_AXONS_PER_SEGMENT: usize = AXONS_PER_NEURON * COLUMNS_PER_SEGMENT;
pub const COLUMN_DENDRITES_PER_SEGMENT: usize = DENDRITES_PER_NEURON * COLUMNS_PER_SEGMENT;
pub const COLUMN_SYNAPSES_PER_SEGMENT: usize = SYNAPSES_PER_DENDRITE * COLUMN_DENDRITES_PER_SEGMENT;

pub const CELLS_PER_SEGMENT: usize = CELLS_PER_COLUMN * COLUMNS_PER_SEGMENT;
pub const CELL_AXONS_PER_SEGMENT: usize = AXONS_PER_NEURON * CELLS_PER_SEGMENT;
pub const CELL_DENDRITES_PER_SEGMENT: usize = DENDRITES_PER_NEURON * CELLS_PER_SEGMENT;
pub const CELL_SYNAPSES_PER_SEGMENT: usize = SYNAPSES_PER_DENDRITE * CELL_DENDRITES_PER_SEGMENT;

pub const SENSORY_CHORD_WIDTH: usize = 1024; // COLUMNS_PER_SEGMENT;
pub const MOTOR_CHORD_WIDTH: usize = 2;


/*
pub fn print_synapse_values(synapse: &mut Synapse, ocl: &ocl::Ocl) {

	let read_buf = ocl::new_read_buffer(&mut synapse.values.vec, ocl.context);
	let kern = ocl::new_kernel(ocl.program, "get_synapse_values");

	ocl::set_kernel_arg(0, synapse.values.buf, kern);
	ocl::set_kernel_arg(1, read_buf, kern);

	ocl::enqueue_kernel(kern, ocl.command_queue, synapse.values.vec.len());

	ocl::enqueue_read_buffer(&mut synapse.values.vec, read_buf, ocl.command_queue);

	ocl::release_mem_object(read_buf);

	println!("Printing Synapse Values...");
	let mut color: &'static str;
	for i in range(0, synapse.values.vec.len()) {
		if synapse.values.vec[i] != 0u8 {
			color = common::C_ORA;
			print!("({}[{}]:{}{})", color, i, synapse.values.vec[i], common::C_DEFAULT);
		} else {
			//color = common::C_DEFAULT;
		}
	}
	println!("");
}
*/

//pub fn print_component_vec_values<T: Primitive + Int + Zero + Show>(vec: Vec<T>) {

pub fn print_vec<T: Int + String + Default>(vec: &Vec<T>) {

	let every = 1000;

	println!("Printing Component Vector (len:{}, every:{}) Values...", vec.len(), every);
	let mut color: &'static str;
	for i in range(0, vec.len() / every) {
		let ie = i * every;
		if vec[ie] != Default::default() {
			color = C_ORA;
		} else {
			// color = C_DEFAULT;
			break	// temporary bullshit so we don't print 0's
		}
		print!("({}[{}]:{}{})", color, ie, vec[ie], C_DEFAULT);
	}
	println!("");
}

