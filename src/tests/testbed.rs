use cmn::{ /*self,*/ CorticalDimensions };
use proto::{ ProtoLayerMap, ProtoLayerMaps, ProtoAreaMaps, /*ProtoAreaMap, Cellular,*/ Axonal, Spatial, Horizontal, Sensory, Thalamic, layer, Protocell, Protofilter, Protoinput };
// use cortex::{ self, Cortex };
use thalamus::{ Thalamus };
use ocl::{ /*Envoy, WorkSize,*/ OclContext, OclProgQueue, /*EnvoyDimensions, BuildOptions, BuildOption*/ };
// use interactive::{ input_czar, InputCzar, InputKind };
// use super::hybrid;
// use super::kernels;

pub static PRIMARY_AREA_NAME: &'static str = "v1";
pub static INHIB_LAYER_NAME: &'static str = "iv_inhib";

/* Eventually move defines to a config file or some such */
pub fn define_protolayer_maps() -> ProtoLayerMaps {
	let mut cort_regs: ProtoLayerMaps = ProtoLayerMaps::new();

	cort_regs.add(ProtoLayerMap::new("visual", Sensory)
		//.layer("test_noise", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("motor_in", 1, layer::DEFAULT, Axonal(Horizontal))
		//.layer("olfac", 1, layer::DEFAULT, Axonal(Horizontal))
		.layer("eff_in", 0, layer::EFFERENT_INPUT, Axonal(Spatial))
		//.layer("nothing", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("aff_in", 0, layer::AFFERENT_INPUT, Axonal(Spatial))
		.layer("out", 1, layer::AFFERENT_OUTPUT | layer::EFFERENT_OUTPUT, Axonal(Spatial))
		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["aff_in"], 600)) 
		.layer("iv_inhib", 0, layer::DEFAULT, 
			Protocell::new_inhibitory(4, "iv"))
		.layer("iii", 1, layer::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(0, 5, vec!["iii"], 1200).apical(vec!["eff_in"]))
	);

	cort_regs.add(ProtoLayerMap::new("external", Thalamic)
		.layer("ganglion", 1, layer::AFFERENT_OUTPUT | layer::AFFERENT_INPUT, Axonal(Spatial))
	);

	cort_regs
}

pub fn define_protoareas() -> ProtoAreaMaps {
	let area_side = 32 as u32;

	let mut protoareas = ProtoAreaMaps::new()

		.area_ext("v0", "external", area_side, area_side, 
			Protoinput::IdxReader { 
				file_name: "data/train-images-idx3-ubyte", 
				repeats: 1,
				scale: 1.1,
			},

			None, 
			Some(vec![PRIMARY_AREA_NAME]),
		)

		.area(PRIMARY_AREA_NAME, "visual", area_side, area_side, 
			Some(vec![Protofilter::new("retina", Some("filters.cl"))]),			
			Some(vec!["b1"]),
			//None,
		)

		.area("b1", "visual", area_side, area_side,
		 	None,
		 	//Some(vec!["a1"]),
		 	None,
		)
	;

	protoareas
}


// TESTBED {}: Stripped down cortex/cortical area
pub struct TestBed {
	pub ocl_context: OclContext,
	pub ocl: OclProgQueue,
	pub thal: Thalamus,
	pub dims: CorticalDimensions,
}

impl TestBed {
	pub fn new() -> TestBed {
		let proto_layer_maps = define_protolayer_maps();
		let mut proto_area_maps = define_protoareas();

		proto_area_maps.freeze();

		let thal = Thalamus::new(&proto_layer_maps, &proto_area_maps);
		let area_map = thal.area_map(PRIMARY_AREA_NAME).clone();

		let ocl_context = OclContext::new(None);
		let mut ocl = OclProgQueue::new(&ocl_context, None);
		ocl.build(area_map.gen_build_options());

		let dims = area_map.dims().clone_with_physical_increment(ocl.get_max_work_group_size());

		TestBed {
			ocl_context: ocl_context,
			ocl: ocl,
			thal: thal,
			dims: dims,
		}
	}
}

impl Drop for TestBed {
	fn drop(&mut self) {
    	print!("Releasing OpenCL components for test bed... ");
    	self.ocl.release_components();
    	self.ocl_context.release_components();
    	print!(" ...complete. \n");
	}
}
