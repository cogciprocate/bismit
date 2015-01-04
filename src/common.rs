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

pub const CORTICAL_SEGMENTS_TOTAL: uint = 1;
pub const SENSORY_SEGMENTS_TOTAL: uint = 2;
pub const MOTOR_SEGMENTS_TOTAL: uint = 1;

pub const HYPERCOLUMNS_PER_SEGMENT: uint = 16;		// appears to cause lots of delay... 256 is slow

pub const SYNAPSE_WEIGHT_ZERO: u8 = 16;
pub const SYNAPSE_WEIGHT_INITIAL_DEVIATION: u8 = 3;
pub const DENDRITE_INITIAL_THRESHOLD: u8 = 16;

pub const COLUMNS_PER_HYPERCOLUMN: uint = 64u;
//pub const COLUMNS_PER_ADDRESS_BLOCK: uint = 16u;
pub const CELLS_PER_COLUMN: uint = 16u;
pub const DENDRITES_PER_NEURON: uint = 16u;
pub const AXONS_PER_NEURON: uint = (DENDRITES_PER_NEURON * SYNAPSES_PER_DENDRITE);
pub const SYNAPSES_PER_DENDRITE: uint = 16u;

pub const COLUMNS_PER_SEGMENT: uint = COLUMNS_PER_HYPERCOLUMN * HYPERCOLUMNS_PER_SEGMENT;
pub const COLUMN_AXONS_PER_SEGMENT: uint = AXONS_PER_NEURON * COLUMNS_PER_SEGMENT;
pub const COLUMN_DENDRITES_PER_SEGMENT: uint = DENDRITES_PER_NEURON * COLUMNS_PER_SEGMENT;
pub const COLUMN_SYNAPSES_PER_SEGMENT: uint = SYNAPSES_PER_DENDRITE * COLUMN_DENDRITES_PER_SEGMENT;

pub const CELLS_PER_SEGMENT: uint = CELLS_PER_COLUMN * COLUMNS_PER_SEGMENT;
pub const CELL_AXONS_PER_SEGMENT: uint = AXONS_PER_NEURON * CELLS_PER_SEGMENT;
pub const CELL_DENDRITES_PER_SEGMENT: uint = DENDRITES_PER_NEURON * CELLS_PER_SEGMENT;
pub const CELL_SYNAPSES_PER_SEGMENT: uint = SYNAPSES_PER_DENDRITE * CELL_DENDRITES_PER_SEGMENT;

pub const SENSORY_CHORD_WIDTH: uint = 1024;
pub const MOTOR_CHORD_WIDTH: uint = 2;

pub const READBACK_TEST_ITERATIONS: uint = 20000;  // 10,000,000 takes >>> 15 min
