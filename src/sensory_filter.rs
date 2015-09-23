use ocl::{ self, OclProgQueue, WorkSize, Envoy, };
use cmn::{ self, CorticalDimensions };
use axons::{ Axons };

pub struct SensoryFilter {
	filter_name: String,
	cl_file_name: Option<String>,
	area_name: &'static str,
	dims: CorticalDimensions,
	input: Envoy<ocl::cl_uchar>,
	kern_cycle: ocl::Kernel,
}

impl SensoryFilter {
	pub fn new(
				filter_name: String, 
				cl_file_name: Option<String>, 
				area_name: &'static str,
				dims: CorticalDimensions, 
				axns: &Axons,
				axn_base_slc: u8,
				ocl: &OclProgQueue, 
	) -> SensoryFilter {

		let input = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);

		let kern_cycle = ocl.new_kernel(filter_name.clone(),
			WorkSize::ThreeDim(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			.lws(WorkSize::ThreeDim(1, 8, 8 as usize))
			.arg_env(&input)
			.arg_scl(axn_base_slc)
			.arg_env(&axns.states)
		;

		SensoryFilter {
			filter_name: filter_name,
			cl_file_name: cl_file_name,
			area_name: area_name,
			dims: dims,
			input: input,
			kern_cycle: kern_cycle,
		}
	}

	pub fn write(&self, sdr: &[ocl::cl_uchar]) {
		assert!(sdr.len() == self.input.len());
		self.input.write_direct(sdr, 0);
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue();
		//println!("Printing {} for {}:\n", &self.filter_name, self.area_name);
		//self.input.print_simple();
	}
}
