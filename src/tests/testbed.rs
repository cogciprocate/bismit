use cmn::{ CorticalDimensions };
use proto::{ layer, ProtoLayerMap, ProtoLayerMaps, ProtoAreaMaps, Axonal, Spatial, Horizontal, Sensory, Thalamic, Protocell, Protofilter, Protoinput };
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
		.layer("unused", 1, layer::UNUSED_TESTING, Axonal(Spatial))
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

			Some(vec!["b1"]),
			// None,
		)

		.area("b1", "visual", 
			// area_side * 2, area_side * 2,			
			area_side, area_side,
			//32, 32,
			//256, 256,

		 	None,

		 	// Some(vec!["a1"]),
		 	None,
		)

		// .area("a1", "visual", area_side, area_side, None, None)
	;

	protoareas
}


// FRESH_CORTEX(): Mmmm... Yummy.
pub fn fresh_cortex() -> Cortex {
	Cortex::new(define_protolayer_maps(), define_protoareas())
}


/*=============================================================================
===============================================================================
================================== CORTEX 2 ===================================
===============================================================================
=============================================================================*/

pub fn init_test_cortex_2() -> Cortex {
	let area_name = PRIMARY_AREA_NAME;
	let lmap_name = "lm_test";

	let mut plmaps = ProtoLayerMaps::new();

	plmaps.add(ProtoLayerMap::new(lmap_name, Sensory)
		.layer("eff_in", 0, layer::EFFERENT_INPUT, Axonal(Spatial))
		.layer("aff_in", 0, layer::AFFERENT_INPUT, Axonal(Spatial))
		.layer("out", 1, layer::AFFERENT_OUTPUT | layer::EFFERENT_OUTPUT, Axonal(Spatial))
		.layer("unused", 1, layer::UNUSED_TESTING, Axonal(Spatial))
		.layer("test1", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("test2", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("test3", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("test4", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("test5", 1, layer::DEFAULT, Axonal(Spatial))
		.layer("iv", 1, layer::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["unused"], 600))
		// .layer("iv_inhib", 0, layer::DEFAULT, 
		// 	Protocell::new_inhibitory(4, "iv"))
		.layer("iii", 3, layer::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(2, 4, vec!["unused"], 400)
				.apical(vec!["test1"])
				.apical(vec!["test2"])
				.apical(vec!["test3"])
				.apical(vec!["test4"])
				.apical(vec!["test5"]))
	);

	plmaps.add(ProtoLayerMap::new("dummy_lm", Thalamic)
		.layer("ganglion", 1, layer::AFFERENT_OUTPUT | layer::AFFERENT_INPUT, Axonal(Spatial))
	);

	let pamaps = ProtoAreaMaps::new()
		.area(area_name, lmap_name, 48, 48, None, None)
		.area_ext("dummy_area", "dummy_lm", 64, 63, Protoinput::None, None, Some(vec![area_name]))
	;

	Cortex::new(plmaps, pamaps)
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
