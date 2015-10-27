use ocl::{ self, OclProgQueue, WorkSize, Envoy, };
use cmn::{ self, /*CorticalDimensions,*/ HexTilePlane, Sdr };
use axon_space::{ AxonSpace };
use proto::{ layer };
use map::{ AreaMap };


pub struct SensoryFilter {
	filter_name: String,
	cl_file_name: Option<String>,
	area_name: &'static str,
	//dims: CorticalDimensions,
	input: Envoy<ocl::cl_uchar>,
	kern_cycle: ocl::Kernel,
}

impl SensoryFilter {
	pub fn new(
				filter_name: String, 
				cl_file_name: Option<String>, 
				area_map: &AreaMap,
				//area_name: &'static str,
				//dims: CorticalDimensions, 
				axns: &AxonSpace,
				//base_axn_slc: u8,
				ocl: &OclProgQueue, 
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

		let kern_cycle = ocl.new_kernel(filter_name.clone(),
			WorkSize::ThreeDim(dims.depth() as usize, dims.v_size() as usize, dims.u_size() as usize))
			.lws(WorkSize::ThreeDim(1, 8, 8 as usize))
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

	pub fn write(&self, sdr: &Sdr) {
		assert!(sdr.len() == self.input.len());
		self.input.write_direct(sdr, 0);
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue();
		//println!("Printing {} for {}:\n", &self.filter_name, self.area_name);
		//self.input.print_simple();
	}
}
