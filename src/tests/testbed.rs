use cmn::{ CorticalDimensions };
use proto::{ ProtoLayerMap, ProtoLayerMaps, ProtoAreaMaps, Axonal, Spatial, Horizontal, Sensory, Thalamic, layer, Protocell, Protofilter, Protoinput };
use thalamus::{ Thalamus };
use ocl::{ OclContext, OclProgQueue };
use cortex::{ Cortex };


pub static PRIMARY_AREA_NAME: &'static str 	= "v1";
pub static INHIB_LAYER_NAME: &'static str 	= "iv_inhib";
const REPEATS_PER_IMAGE: usize 				= 1;


pub fn define_protolayer_maps() -> ProtoLayerMaps {
	let mut proto_layer_maps: ProtoLayerMaps = ProtoLayerMaps::new();

	proto_layer_maps.add(ProtoLayerMap::new("visual", Sensory)
		//.layer("test_noise", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("motor_in", 1, layer::DEFAULT, Axonal(Horizontal))
		//.layer("olfac", 1, layer::DEFAULT, Axonal(Horizontal))
		.layer("eff_in", 0, layer::EFFERENT_INPUT /*| layer::INTERAREA*/, Axonal(Spatial))
		.layer("aff_in", 0, layer::AFFERENT_INPUT /*| layer::INTERAREA*/, Axonal(Spatial))
		.layer("out", 1, layer::AFFERENT_OUTPUT | layer::EFFERENT_OUTPUT, Axonal(Spatial))
		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["aff_in"], 600)) 
		.layer("iv_inhib", 0, layer::DEFAULT, 
			Protocell::new_inhibitory(4, "iv"))
		.layer("iii", 3, layer::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(2, 4, vec!["iii"], 1200).apical(vec!["eff_in"]))
	);

	proto_layer_maps.add(ProtoLayerMap::new("external", Thalamic)
		.layer("ganglion", 1, layer::AFFERENT_OUTPUT | layer::AFFERENT_INPUT, Axonal(Spatial))
	);

	proto_layer_maps
}

pub fn define_protoareas() -> ProtoAreaMaps {
	let area_side = 32 as u32;

	let protoareas = ProtoAreaMaps::new()		

		.area_ext("v0", "external", 
			// area_side * 2, area_side * 2,
			area_side, area_side,
			// area_side / 2, area_side / 2, 
			Protoinput::IdxReader { 
				file_name: "data/train-images-idx3-ubyte", 
				repeats: REPEATS_PER_IMAGE, 
				scale: 1.3,
			},

			None, 
			Some(vec!["v1"]),
		)

		.area("v1", "visual", 
			// area_side * 2, area_side * 2,
			area_side, area_side,
			// area_side / 2, area_side / 2,
			// 128, 128,
			Some(vec![Protofilter::new("retina", Some("filters.cl"))]),			
			// Some(vec!["b1"]),
			None,
		)

		// .area("b1", "visual", 
		// 	// area_side * 2, area_side * 2,			
		// 	area_side, area_side,
		// 	//32, 32,
		// 	//256, 256,
		//  	None,		 	
		//  	// Some(vec!["a1"]),
		//  	None,
		// )

		// .area("a1", "visual", area_side, area_side, None, None)
	;

	protoareas
}


// FRESH_CORTEX(): Mmmm... Yummy.
pub fn fresh_cortex() -> Cortex {
	Cortex::new(define_protolayer_maps(), define_protoareas())
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
