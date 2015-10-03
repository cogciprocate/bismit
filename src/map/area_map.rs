use num::{ Num };
use std::fmt::{ Display };
use std::ops::{ Range };
use std::collections::{ HashMap };
//use std::num::ToString;

use ocl::{ BuildOptions, BuildOption };
use proto::{ layer, ProtoLayerMaps, ProtoLayerMap, Protolayer, ProtolayerFlags, ProtoAreaMaps, ProtoAreaMap };
use cmn::{ self, CorticalDimensions, SliceDimensions };
use map::{ SliceMap, InterAreaInfoCache };
use map::slice_map;

// 	AREAMAP { }:
// 		- Move in functionality from the 'execution phase' side of ProtoAreaMap and ProtoLayerMap.
//		- Leave the 'init phase' stuff to the proto-*s.
#[derive(Clone)]
pub struct AreaMap {
	area_name: &'static str,
	dims: CorticalDimensions,
	ia_cache: InterAreaInfoCache,


	// aff_in_layer_name: &'static str,
	// eff_in_layer_name: &'static str,

	slices: SliceMap,

	hrz_demarc: u8,

	// Create maps for each aspect which have their own types and are queryable 
	// into sub-lists of the same type

	// layers: LayerMap
	// slices: SliceMap
	// etc...

	//emsg: &'static str,
	// other new types: TuftMap/CellMap
	proto_area_map: ProtoAreaMap,
	proto_layer_map: ProtoLayerMap,
}

impl AreaMap {
	pub fn new(proto_area_map: &ProtoAreaMap, plmaps: &ProtoLayerMaps, pamaps: &ProtoAreaMaps) -> AreaMap {
		let proto_area_map = proto_area_map.clone();			
		let mut proto_layer_map = plmaps[proto_area_map.region_name].clone();
		proto_layer_map.freeze(&proto_area_map);

		println!("{}AREAMAP::NEW(): area name: {}, eff areas: {:?}, aff areas: {:?}", cmn::MT, proto_area_map.name, 
			proto_area_map.eff_areas, proto_area_map.aff_areas);

		let dims = proto_area_map.dims().clone_with_depth(proto_layer_map.depth_total());		
		let hrz_demarc = proto_layer_map.hrz_demarc();
		let area_name = proto_area_map.name;

		let ia_cache = InterAreaInfoCache::new(
			&dims,
			&proto_area_map.eff_areas, // EFF AREAS
			&proto_area_map.aff_areas, // AFF AREAS
			proto_layer_map.layer_with_flag(layer::AFFERENT_INPUT), // AFF INPUT LAYER			
			proto_layer_map.layer_with_flag(layer::EFFERENT_INPUT), // EFF INPUT LAYER
			proto_layer_map.layer_with_flag(layer::AFFERENT_OUTPUT), // AFF & EFF OUTPUT LAYER
			pamaps,
		);

		let slices = SliceMap::new(&dims, &proto_area_map, &proto_layer_map, &ia_cache);		

		AreaMap {
			area_name: area_name,
			dims: dims,
			ia_cache: ia_cache,
			slices: slices,
			hrz_demarc: hrz_demarc,
			proto_area_map: proto_area_map,
			proto_layer_map: proto_layer_map,
			//emsg: emsg,
		}
	}	

	pub fn axn_base_slc_by_flag(&self, layer_flags: layer::ProtolayerFlags) -> u8 {
		self.proto_layer_map.layer_with_flag(layer_flags).expect("Cannot find layer").base_slc()
	}

	pub fn axn_range_by_flag(&self, layer_flags: layer::ProtolayerFlags) -> Range<u32> {				
		let (layer, layer_len) = self.layer_source_area_info(layer_flags);
		let layer_idz = self.axn_idz(layer.base_slc_id);
		layer_idz..(layer_idz + layer_len)
	}

	pub fn slc_src_area_dims(&self, slc_id: u8, layer_flags: layer::ProtolayerFlags) -> &SliceDimensions {
		//self.proto_layer_map.layer_with_flag(layer_flags).expect("Cannot find layer").layer_base_slc()

		// GET SOURCE AREA DIMS!
		// 		- get layer name
		// 		- get layer slice list
		// 		- return 
		match self.ia_cache.src_area_for_slc(slc_id, layer_flags) {
			Some(ref area) => area.dims(),
			None => panic!("Cannot find a slice with id: '{}' and flags: '{:?}'.", slc_id, layer_flags),
		}
	}

	pub fn input_src_area_names(&self, layer_flags: layer::ProtolayerFlags) -> &Vec<&'static str> {
		if layer_flags == layer::EFFERENT_INPUT {
			//panic!("Fix me");
			&self.proto_area_map.aff_areas
		} else if layer_flags == layer::AFFERENT_INPUT {
			//panic!("Fix me");
			&self.proto_area_map.eff_areas
		} else {
			panic!("\nAreaMap::input_src_area_names(): Can only be called with an \
				input layer flag as argument");
		}		
	}

	// LAYER_SOURCE_AREA_INFO(): DEPRICATE THIS UGLY BASTARD
	pub fn layer_source_area_info(&self, layer_flags: layer::ProtolayerFlags) -> (&Protolayer, u32) {
		let emsg = format!("\nAreaMap:: '{:?}' flag not set for any layer in area: '{}'.", 
			layer_flags, self.area_name);

		if layer_flags.contains(layer::AFFERENT_INPUT) {
			//println!("##### AFF IN CONTAINED");
			match &self.ia_cache.aff_in_layer { 
				&Some(ref l) => (l, self.ia_cache.eff_areas.axns_sum()), 
				&None => panic!(emsg), 
			}
		} else if layer_flags.contains(layer::EFFERENT_INPUT) {
			//println!("##### EFF IN CONTAINED");
			match &self.ia_cache.eff_in_layer { 
				&Some(ref l) => (l, self.ia_cache.aff_areas.axns_sum()), 
				&None => panic!(emsg), 
			}
		} else if layer_flags.contains(layer::AFFERENT_OUTPUT) {
			//println!("##### AFF OUT CONTAINED");
			match &self.ia_cache.out_layer { 
				&Some(ref l) => (l, self.dims.columns()), 
				&None => panic!(emsg), 
			} 
		} else if layer_flags.contains(layer::EFFERENT_OUTPUT) { // REDUNDANT (MERGE WITH ABOVE)
			//println!("##### EFF OUT CONTAINED");
			match &self.ia_cache.out_layer { 
				&Some(ref l) => (l, self.dims.columns()), 
				&None => panic!(emsg), 
			} 
		} else { 
			//println!("##### CALCULATING LAYER LENGTH OLD SCHOOL");
			let l = self.proto_layer_map.layer_with_flag(layer_flags).expect(&emsg);
			//let layer_len = axn_idz_2d(l.base_slc_id, self.dims.columns(), self.hrz_demarc);
			let layer_len = self.dims.columns();
			(l, layer_len)
		}
	}

	pub fn gen_build_options(&self) -> BuildOptions {
		let mut build_options = cmn::base_build_options()
			.opt("HORIZONTAL_AXON_ROW_DEMARCATION", self.hrz_demarc as i32)
			.opt("AXN_SLC_COUNT", self.slices.slc_count() as i32)
			.add_opt(BuildOption::with_str_val("AXN_SLC_IDZS", literal_list(self.slices.axn_idzs())))
			.add_opt(BuildOption::with_str_val("AXN_SLC_V_SIZES", literal_list(self.slices.v_sizes())))
			.add_opt(BuildOption::with_str_val("AXN_SLC_U_SIZES", literal_list(self.slices.u_sizes())))
			.add_opt(BuildOption::with_str_val("AXN_SLC_V_SCALES", literal_list(self.slices.v_scales())))
			.add_opt(BuildOption::with_str_val("AXN_SLC_U_SCALES", literal_list(self.slices.u_scales())))
		;

		// CUSTOM KERNELS
		match self.proto_area_map.filters {
			Some(ref protofilters) => {
				for pf in protofilters.iter() {
					match pf.cl_file_name() {
						Some(ref clfn)  => build_options.add_kern_file(clfn.clone()),
						None => (),
					}
				}
			},
			None => (),
		};

		cmn::load_builtin_kernel_files(&mut build_options);

		build_options
	}

	pub fn area_name(&self) -> &'static str {
		self.area_name
	}

	pub fn proto_area_map(&self) -> &ProtoAreaMap {
		&self.proto_area_map
	}

	pub fn proto_layer_map(&self) -> &ProtoLayerMap {
		&self.proto_layer_map
	}

	pub fn axn_idz(&self, slc_id: u8) -> u32 {
		self.slices.idz(slc_id)
	}

	pub fn slices(&self) -> &SliceMap {
		&self.slices
	}

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}
}


pub fn literal_list<T: Display>(vec: &Vec<T>) -> String {
	let mut literal = String::with_capacity((vec.len() * 5) + 20);

	let mut i = 0u32;	
	for ele in vec.iter() {
		if i != 0 {
			literal.push_str(", ");
		}

		literal.push_str(&ele.to_string());
		i += 1;
	}

	literal
}



/* AXN_IDX_2D(): Host side address resolution - concerned with start idx of a slc
	- OpenCL device side version below [outdated] (for reference) - concerned with individual indexes:

		static inline uint axn_idz_2d(uchar slc_id, uint slc_columns, uint col_id, char col_ofs) {
			uint axn_idx_spt = mad24((uint)slc_id, slc_columns, (uint)(col_id + col_ofs + AXON_MAR__GIN_SIZE));
			int hslc_id = slc_id - HORIZONTAL_AXON_ROW_DEMARCATION;
			int hcol_id = mad24(hslc_id, SYNAPSE_SPAN_RHOMBAL_AREA, col_ofs + AXON_MAR__GIN_SIZE);
			uint axn_idx_hrz = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, slc_columns, (uint)(hcol_id + AXON_MAR__GIN_SIZE));
			return mul24((uint)(hslc_id < 0), axn_idx_spt) + mul24((uint)(hslc_id >= 0), axn_idx_hrz);
		}
}*/
// pub fn axn_idz_2d(axn_slc: u8, columns: u32, hrz_demarc: u8) -> u32 {
// 	let mut axn_idx: u32 = if axn_slc < hrz_demarc {
// 		(axn_slc as u32 * columns)
// 	} else {
// 		(hrz_demarc as u32 * columns) + (cmn::SYNAPSE_SPAN_RHOMBAL_AREA * (axn_slc as u32 - hrz_demarc as u32))
// 	};

// 	axn_idx + cmn::AXON_MARGIN_SIZE as u32
// }


// fn rewrap_layer_name(layer_opt: Option<Protolayer>) -> Option<&'static str> {
// 	match layer_opt {
// 		Some(pl) => Some(pl.name),
// 		None => None,
// 	}
// }

// fn rewrap_layer_base_slc_id(layer_opt: Option<Protolayer>) -> Option<&'static str> {
// 	match layer_opt {
// 		Some(pl) => Some(pl.base_slc_id),
// 		None => None,
// 	}
// }
