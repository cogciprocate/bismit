use cmn::{ CorticalDims };
use map;
use proto::{ ProtolayerMap, ProtolayerMaps, ProtoareaMaps, Axonal, Spatial, Horizontal, Sensory, Thalamic, Protocell, Protofilter, Protoinput };
use thalamus::{ Thalamus };
use ocl::{ Context, ProQue };
use cortex::{ Cortex };


pub static PRIMARY_AREA_NAME: &'static str 	= "v1";
pub static INHIB_LAYER_NAME: &'static str 	= "iv_inhib";
const CYCLES_PER_FRAME: usize 				= 1;


pub fn define_protolayer_maps() -> ProtolayerMaps {
	let mut proto_layer_maps: ProtolayerMaps = ProtolayerMaps::new();

	proto_layer_maps.add(ProtolayerMap::new("visual", Sensory)
		//.layer("test_noise", 1, map::DEFAULT, Axonal(Spatial))
		.layer("motor_in", 1, map::DEFAULT, Axonal(Horizontal))
		//.layer("olfac", 1, map::DEFAULT, Axonal(Horizontal))
		.layer("eff_in", 0, map::FB_IN /*| map::INTERAREA*/, Axonal(Spatial))
		.layer("aff_in", 0, map::FF_IN /*| map::INTERAREA*/, Axonal(Spatial))
		.layer("out", 1, map::FF_OUT | map::FB_OUT, Axonal(Spatial))
		.layer("unused", 1, map::UNUSED_TESTING, Axonal(Spatial))
		.layer("iv", 1, map::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["aff_in"], 600, 8)) 
		.layer("iv_inhib", 0, map::DEFAULT, 
			Protocell::new_inhibitory(4, "iv"))
		.layer("iii", 3, map::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(2, 4, vec!["iii"], 1200, 8).apical(vec!["eff_in"]))
	);

	proto_layer_maps.add(ProtolayerMap::new("external", Thalamic)
		.layer("ganglion", 1, map::FF_OUT, Axonal(Spatial))
	);

	proto_layer_maps
}

pub fn define_protoareas() -> ProtoareaMaps {
	let area_side = 32 as u32;

	let protoareas = ProtoareaMaps::new()		

		.area_ext("v0", "external", 
			// area_side * 2, area_side * 2,
			area_side, 
						// area_side / 2, area_side / 2, 
			Protoinput::IdxReader { 
				file_name: "data/train-images-idx3-ubyte", 
				cyc_per: CYCLES_PER_FRAME, 
				scale: 1.3,
			},

			None, 
			None,
		)

		.area("v1", "visual", 
			// area_side * 2, area_side * 2,
			area_side, 
			// area_side / 2, area_side / 2,
			// 128, 128,

			Some(vec![Protofilter::new("retina", Some("filters.cl"))]),			

			Some(vec!["v0"]),
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


/*=============================================================================
===============================================================================
================================== CORTEX 2 ===================================
===============================================================================
=============================================================================*/

// LOTS OF TUFTS, THRESHOLD AT MIN
pub fn cortex_with_lots_of_apical_tufts() -> Cortex {
	let area_name = PRIMARY_AREA_NAME;
	let lmap_name = "lm_test";

	let mut plmaps = ProtolayerMaps::new();

	plmaps.add(ProtolayerMap::new(lmap_name, Sensory)
		.layer("eff_in", 0, map::FB_IN, Axonal(Spatial))
		.layer("aff_in", 0, map::FF_IN, Axonal(Spatial))
		.layer("out", 1, map::FF_OUT | map::FB_OUT, Axonal(Spatial))
		.layer("test0", 1, map::DEFAULT, Axonal(Spatial))
		.layer("test1", 1, map::UNUSED_TESTING, Axonal(Spatial))
		.layer("test2", 1, map::UNUSED_TESTING, Axonal(Spatial))
		.layer("test3", 1, map::UNUSED_TESTING, Axonal(Spatial))
		// .layer("test4", 1, map::UNUSED_TESTING, Axonal(Spatial))
		// .layer("test5", 1, map::UNUSED_TESTING, Axonal(Spatial))
		.layer("unused", 1, map::UNUSED_TESTING, Axonal(Spatial))
		.layer("iv", 1, map::SPATIAL_ASSOCIATIVE, 
			Protocell::new_spiny_stellate(5, vec!["unused"], 1, 8))
		// .layer("iv_inhib", 0, map::DEFAULT, 
		// 	Protocell::new_inhibitory(4, "iv"))
		.layer("iii", 2, map::TEMPORAL_ASSOCIATIVE, 
			Protocell::new_pyramidal(2, 4, vec!["unused"], 1, 8)
				.apical(vec!["test1"])
				.apical(vec!["test2"])
				// .apical(vec!["test3"])
				// .apical(vec!["test4"])
				// .apical(vec!["test5"])
		)

	);

	plmaps.add(ProtolayerMap::new("dummy_lm", Thalamic)
		.layer("ganglion", 1, map::FF_OUT, Axonal(Spatial))
	);

	let pamaps = ProtoareaMaps::new()
		.area(area_name, lmap_name, 32, None, Some(vec!["dummy_area"]))

		// <<<<< VERY IMPORTANT: DO NOT DELETE! >>>>>
		// [FIXME] THIS EXTERNAL AREA MAY BE CAUSING INDEXING PROBLEMS
		.area_ext("dummy_area", "dummy_lm", 67, Protoinput::None, None, None)

		// .area_ext("dummy_area", "dummy_lm", 32, 32, Protoinput::None, None, Some(vec![area_name]))
	;

	Cortex::new(plmaps, pamaps)
}



// TESTBED {}: Stripped down cortex/cortical area
pub struct TestBed {
	pub ocl_context: Context,
	pub ocl_pq: ProQue,
	pub thal: Thalamus,
	pub dims: CorticalDims,
}

impl TestBed {
	pub fn new() -> TestBed {
		let proto_layer_maps = define_protolayer_maps();
		let proto_area_maps = define_protoareas();

		// proto_area_maps.freeze();

		let thal = Thalamus::new(proto_layer_maps, proto_area_maps);
		let area_map = thal.area_map(PRIMARY_AREA_NAME).clone();

		let ocl_context = Context::new(None, None).unwrap();
		let mut ocl_pq = ProQue::new(&ocl_context, None);
		ocl_pq.build(area_map.gen_build_options()).ok();

		let dims = area_map.dims().clone_with_physical_increment(ocl_pq.get_max_work_group_size());

		TestBed {
			ocl_context: ocl_context,
			ocl_pq: ocl_pq,
			thal: thal,
			dims: dims,
		}
	}
}

impl Drop for TestBed {
	fn drop(&mut self) {
    	print!("Releasing OpenCL components for test bed... ");
    	self.ocl_pq.release();
    	self.ocl_context.release();
    	print!(" ...complete. \n");
	}
}
