use ocl::{ self, OclProgQueue, WorkSize, Envoy, CorticalDimensions };

pub struct SensoryFilter {
	filter_name: String,
	cl_file_name: Option<String>,
	dims: CorticalDimensions,
	input: Envoy<ocl::cl_uchar>,
	kern_cycle: ocl::Kernel,
}

impl SensoryFilter {
	pub fn new(filter_name: String, cl_file_name: Option<String>, dims: CorticalDimensions, ocl: &OclProgQueue) -> SensoryFilter {

		let input = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);

		let kern_cycle = ocl.new_kernel(filter_name.clone(), 
			WorkSize::ThreeDim(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			.lws(WorkSize::ThreeDim(1, 8, 8 as usize));

		SensoryFilter {
			filter_name: filter_name,
			cl_file_name: cl_file_name,
			dims: dims,
			input: input,
			kern_cycle: kern_cycle,
		}
	}
}
