pub static SDR_LETTERS: uint = 16;
pub static SDR_TARGETS_PER_LETTER: uint = 4;
pub static SDR_LETTERS_PER_TARGET: uint = 1;

fn init() {
	let sdr: Vec<ocl::cl_ushort> = Vec::with_capacity(SDR_LETTERS);
	let sdr_sources_map: Vec<ocl::cl_ushort> = Vec::with_capacity(SDR_LETTERS * SDR_TARGETS_PER_LETTER * SDR_LETTERS_PER_TARGET);
	let sdr_targets_map: Vec<ocl::cl_ushort> = Vec::with_capacity(SDR_LETTERS * SDR_TARGETS_PER_LETTER * SDR_LETTERS_PER_TARGET);

	

}
