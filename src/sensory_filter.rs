use ocl::{ self, ProQueue, WorkSize, Envoy, };
use cmn::{ self, /*CorticalDims,*/ HexTilePlane, Sdr };
use axon_space::{ AxonSpace };
use proto::{ layer };
use map::{ AreaMap };


pub struct SensoryFilter {
	filter_name: String,
	cl_file_name: Option<String>,
	area_name: &'static str,
	//dims: CorticalDims,
	input: Envoy<ocl::cl_uchar>,
	kern_cycle: ocl::Kernel,
}

impl SensoryFilter {
	pub fn new(
				filter_name: String, 
				cl_file_name: Option<String>, 
				area_map: &AreaMap,
				//area_name: &'static str,
				//dims: CorticalDims, 
				axns: &AxonSpace,
				//base_axn_slc: u8,
				ocl: &ProQueue, 
		) -> SensoryFilter 
	{
		let base_axn_slc_ids = area_map.axn_base_slc_ids_by_flag(layer::AFFERENT_INPUT);
		assert!(base_axn_slc_ids.len() == 1);
		let base_axn_slc = base_axn_slc_ids[0];

		let dims = area_map.slc_src_area_dims(base_axn_slc, layer::AFFERENT_INPUT);
		assert!(dims.depth() == 1, "\nAfferent input layer depths of more than one for cortical \
			areas with sensory filters are not yet supported. Please set the depth of any \
			afferent input layers with filters to 1.");

		let input = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);		

		let kern_cycle = ocl.create_kernel(filter_name.clone(),
			WorkSize::ThreeDims(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			.lws(WorkSize::ThreeDims(1, 8, 8 as usize))
			.arg_env(&input)
			.arg_scl(base_axn_slc)
			.arg_env(&axns.states)
		;

		SensoryFilter {
			filter_name: filter_name,
			cl_file_name: cl_file_name,
			area_name: area_map.area_name(),
			//dims: dims,
			input: input,
			kern_cycle: kern_cycle,
		}
	}

	pub fn write(&mut self, sdr: &Sdr) {
		assert!(sdr.len() == self.input.len());
		self.input.write_direct(sdr, 0);
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue(None, None);
		//println!("Printing {} for {}:\n", &self.filter_name, self.area_name);
		//self.input.print_simple();
	}
}
