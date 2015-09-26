use num::{ Num };
use std::fmt::{ Display };
use std::ops::{ Range };
use std::collections::{ HashMap };
//use std::num::ToString;

use ocl::{ BuildOptions, BuildOption };
use proto::{ layer, ProtoLayerMaps, ProtoLayerMap, Protolayer, ProtolayerFlags, ProtoAreaMaps, ProtoAreaMap };
use cmn::{ self, CorticalDimensions };

// 	AREAMAP { }:
// 		- Move in functionality from the 'execution phase' side of proto_area_map and plmap.
//		- Leave the 'init phase' stuff to the proto-*s.
#[derive(Clone)]
pub struct AreaMap {
	area_name: &'static str,
	dims: CorticalDimensions,
	ia_cache: InterAreaInfoCache,


	// aff_in_layer_name: &'static str,
	// eff_in_layer_name: &'static str,

	pub slices: SliceMap,

	hrz_demarc: u8,

	// Create maps for each aspect which have their own types and are queryable 
	// into sub-lists of the same type

	// layers: LayerMap
	// slices: SliceMap
	// etc...

	// other new types: TuftMap/CellMap
	proto_area_map: ProtoAreaMap,
	proto_layer_map: ProtoLayerMap,
}

impl AreaMap {
	pub fn new(proto_area_map: &ProtoAreaMap, plmaps: &ProtoLayerMaps, pamaps: &ProtoAreaMaps) -> AreaMap {
		let proto_area_map = proto_area_map.clone();			
		let mut proto_layer_map = plmaps[proto_area_map.region_name].clone();
		proto_layer_map.freeze(&proto_area_map);

		let ia_cache = InterAreaInfoCache::new(
			&proto_area_map.eff_areas, // EFF AREAS
			proto_layer_map.layer_with_flag(layer::AFFERENT_INPUT), // AFF INPUT LAYER
			&proto_area_map.aff_areas, // AFF AREAS
			proto_layer_map.layer_with_flag(layer::EFFERENT_INPUT), // EFF INPUT LAYER
			proto_layer_map.layer_with_flag(layer::AFFERENT_OUTPUT), // AFF & EFF OUTPUT LAYER
			pamaps,
		);

		let dims = proto_area_map.dims();
		let slices = SliceMap::new(dims, &proto_area_map, &proto_layer_map, &ia_cache);
		let hrz_demarc = proto_layer_map.hrz_demarc();
		let area_name = proto_area_map.name;
		

		AreaMap {
			area_name: area_name,
			dims: dims,
			ia_cache: ia_cache,
			slices: slices,
			hrz_demarc: hrz_demarc,
			proto_area_map: proto_area_map,
			proto_layer_map: proto_layer_map,
		}
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

	pub fn axn_range_by_flag(&self, layer_flags: layer::ProtolayerFlags) -> Range<u32> {
		let emsg = format!("\nAreaMap::axn_range(): '{:?}' flag not set for any layer in area: '{}'.", 
			layer_flags, self.area_name);

		let layer = match layer_flags {
			layer::AFFERENT_INPUT => match &self.ia_cache.aff_in_layer { 
				&Some(ref l) => l, 
				&None => panic!(emsg), 
			},

			layer::EFFERENT_INPUT => match &self.ia_cache.eff_in_layer { 
				&Some(ref l) => l, 
				&None => panic!(emsg), 
			},

			layer::AFFERENT_OUTPUT => match &self.ia_cache.out_layer { 
				&Some(ref l) => l, 
				&None => panic!(emsg), 
			},

			_ => self.proto_layer_map.layer_with_flag(layer_flags).expect(&emsg), // CHANGE TO LAYERS_WITH_FLAG()
		};	
		

		let layer_len = self.dims.columns() * layer.depth as u32;
		let layer_base_slc = layer.base_slc_id;
		//let buffer_offset = cmn::axn_idz_2d(base_slc, self.dims.columns(), self.area_map.plmap().hrz_demarc());
		let layer_idz = self.axn_idz(layer_base_slc);

		layer_idz..(layer_idz + layer_len)
	}

	pub fn input_src_area_names(&self, layer_flags: layer::ProtolayerFlags) -> Vec<&'static str> {
		if layer_flags == layer::EFFERENT_INPUT {
			//panic!("Fix me");
			self.proto_area_map.aff_areas.clone()
		} else if layer_flags == layer::AFFERENT_INPUT {
			//panic!("Fix me");
			self.proto_area_map.eff_areas.clone()
		} else {
			panic!("\nAreaMap::input_src_area_names(): Can only be called with an \
				input layer flag as argument");
		}		
	}

	pub fn gen_build_options(&self) -> BuildOptions {
		let mut build_options = cmn::base_build_options()
			.add_opt(BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", self.hrz_demarc as i32))
			.opt("AXN_SLC_COUNT", self.slices.len() as i32)
			.add_opt(BuildOption::with_str_val("AXN_SLC_IDZS", literal_list(&self.slices.axn_idzs)))
			.add_opt(BuildOption::with_str_val("AXN_SLC_V_SCALES", literal_list(&self.slices.v_scales)))
			.add_opt(BuildOption::with_str_val("AXN_SLC_U_SCALES", literal_list(&self.slices.u_scales)))

			// AXN_SLC_IDZS
			// AXN_SLC_V_SCALES
			// AXN_SLC_U_SCALES
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

		build_options.add_kern_file(cmn::BUILTIN_FILTERS_CL_FILE_NAME.to_string());
		build_options.add_kern_file(cmn::BUILTIN_CL_FILE_NAME.to_string()); // ** MUST BE ADDED LAST **

		build_options
	}

}


#[derive(Debug, Clone)]
pub struct SliceMap {
	axn_idzs: Vec<u32>,
	layer_names: Vec<&'static str>,
	v_scales: Vec<u8>,
	u_scales: Vec<u8>,	
}

impl SliceMap {
	pub fn new(area_dims: CorticalDimensions, pamap: &ProtoAreaMap, plmap: &ProtoLayerMap, 
					ia_cache: &InterAreaInfoCache,
	) -> SliceMap {		
		let proto_slc_map = plmap.slc_map();

		let mut axn_idzs = Vec::with_capacity(proto_slc_map.len());
		let mut layer_names = Vec::with_capacity(proto_slc_map.len());
		let mut v_scales = Vec::with_capacity(proto_slc_map.len());
		let mut u_scales = Vec::with_capacity(proto_slc_map.len());

		/*=============================================================================
		=================================  ================================
		=============================================================================*/

		//eff_in_layer_base_slc_id = ia_cache.eff_in_layer_name


		for (&slc_id, &layer_name) in proto_slc_map.iter() {
			// CALCULATE SCALE FOR V AND U
			let layer = &plmap.layers()[layer_name];
			let src_area_opt = ia_cache.src_area_for_slc(slc_id, layer.flags);

			let (v_scale, u_scale) = if src_area_opt.is_some() {
				// 	NEED A LISTING OF SOURCE AREA NAMES FOR EACH SLICE OF A LAYER!!
				// 	MUST BE SHARED WITH THALAMUS SO MAKE ONE THEN PASS IT TO THAL
				// 	THALAMUS SIMPLY RUNS THROUGH THE LIST OF input_src_area_names()
				// 		- which are just 'proto_area_map.eff_areas' and 'proto_area_map.aff_areas'
				// 
				//  We can build our list from that
				// 	BUILD A NEW SLC_MAP BEFORE ENTERING THIS LOOP!!!
				// 		- Iterate through the layers, peel out each slice with that layer name (preserving slc_id)
				//		- Rebuild the list with source area names
				//			- list of: struct { slc_id, src_area_name, layer_name }
				//		- That list will become the master list
				// 		- Then enter this loop from that list

				//let (v_size, u_size) = ia_cache.eff_areas.get_dim_sizes(source_area_name);

				let (src_area_name, src_v_size, src_u_size) = match src_area_opt {
					Some (sa) => (sa.name, sa.v_size, sa.u_size),
					None => panic!("area_map::SliceMap::new(): Unknown problem with slice."),
				};

				//let src_v_size = src_v_size * 4;
				//let src_u_size = src_u_size * 4;

				let v_scl = calc_scale(src_v_size, area_dims.v_size()).expect(
					&format!("\nSliceMap::new(): Error processing {} for area: '{}', layer: '{}' \
					source area: {}", "v_size", pamap.name, layer_name, src_area_name));
				let u_scl = calc_scale(src_u_size, area_dims.u_size()).expect(
					&format!("\nSliceMap::new(): Error processing {} for area: '{}', layer: '{}' \
					source area: {}", "u_size", pamap.name, layer_name, src_area_name));

				println!("{}SLICEMAP::NEW(): Processing inter-area layer '{}': slc_id: {}, src_area_name: {}, \
					src_v_size: {}, src_u_size: {}.", cmn::MT, layer_name, slc_id, src_area_name,
					src_v_size, src_u_size);

				(v_scl, u_scl)
			} else {
				(16, 16) // 100%
			};

			axn_idzs.push(axn_idz_2d(slc_id, area_dims.columns(), plmap.hrz_demarc()));
			layer_names.push(layer_name);
			v_scales.push(v_scale);
			u_scales.push(u_scale);
		}

		SliceMap {
			axn_idzs: axn_idzs,
			layer_names: layer_names,
			v_scales: v_scales,
			u_scales: u_scales,			
		}
	}

	pub fn print_debug(&self) {
		//let mini_tab = "   "; // 3 spaces

		println!("\n{mt}SLICEMAP::PRINT_DEBUG(): Area slices: \
			\n{mt}{mt}layer_names: {:?}, \
			\n{mt}{mt}axn_idzs: {:?}(literal: '{}'), \
			\n{mt}{mt}v_scales: {:?}(literal: '{}'), \
			\n{mt}{mt}u_scales: {:?}(literal: '{}')", self.layer_names, self.axn_idzs, 
			literal_list(&self.axn_idzs), self.v_scales, literal_list(&self.v_scales), 
			self.u_scales, literal_list(&self.u_scales), mt = cmn::MT);
		println!("");
		// for i in 0..self.axn_idzs.len() {

		// }
	}

	pub fn idz(&self, slc_id: u8) -> u32 {
		self.axn_idzs[slc_id as usize]
	}

	pub fn len(&self) -> u32 {
		self.axn_idzs.len() as u32
	}
}



#[derive(Clone)]
pub struct InterAreaInfoCache {
	eff_areas: LayerSourceAreas, // eff. areas -> aff. input layer
	aff_in_layer: Option<Protolayer>,
	aff_areas: LayerSourceAreas, // aff. areas -> eff. input layer
	eff_in_layer: Option<Protolayer>,
	out_layer: Option<Protolayer>,
}

impl InterAreaInfoCache {
	fn new(
				eff_area_names: &Vec<&'static str>, aff_in_layer: Option<&Protolayer>, 
				aff_area_names: &Vec<&'static str>, eff_in_layer: Option<&Protolayer>,
				out_layer: Option<&Protolayer>,
				pamaps: &ProtoAreaMaps,
	) -> InterAreaInfoCache {
		let eff_areas = LayerSourceAreas::new(eff_area_names, pamaps);
		let aff_areas = LayerSourceAreas::new(aff_area_names, pamaps);

		InterAreaInfoCache { 
			eff_areas: eff_areas, 
			aff_in_layer: clone_rewrap_layer(aff_in_layer), 
			aff_areas: aff_areas, 
			eff_in_layer: clone_rewrap_layer(eff_in_layer),
			out_layer: clone_rewrap_layer(out_layer),
		}
	}

	fn src_area_for_slc(&self, slc_id: u8, flags: ProtolayerFlags) -> Option<SourceAreaInfo> {
		let (layer_src_areas, layer_opt) = if flags.contains(layer::AFFERENT_INPUT) {
			// println!("##### AFF -> slc_id: {}, flags: {:?}", slc_id, flags);
			(&self.eff_areas, &self.aff_in_layer)			
		} else if flags.contains(layer::EFFERENT_INPUT) {			
			// println!("##### EFF -> slc_id: {}, flags: {:?}", slc_id, flags);
			(&self.aff_areas, &self.eff_in_layer)
		} else {
			// println!("##### NONE -> slc_id: {}, flags: {:?}", slc_id, flags);
			return None
		};

		match layer_opt {
			&Some(ref layer) => {
				if slc_id >= layer.base_slc_id && slc_id < layer.base_slc_id + layer.depth {
					assert!(layer.depth as usize == layer_src_areas.len());
					let layer_sub_idx = slc_id - layer.base_slc_id;					
					let src_area_info = layer_src_areas.area_info_by_idx(layer_sub_idx);
					Some(src_area_info)
				} else {
					return None;
				}
			},
			&None => None,
		}
	}
}


#[derive(Clone)]
struct LayerSourceAreas {
	map: HashMap<&'static str, SourceAreaInfo>,
	index: Vec<&'static str>,
}

impl LayerSourceAreas {
	fn new(area_names: &Vec<&'static str>, pamaps: &ProtoAreaMaps) -> LayerSourceAreas {
		let mut map = HashMap::with_capacity(area_names.len());

		for &area_name in area_names.iter() {
			let dims = pamaps.maps()[area_name].dims();
			map.insert(area_name, SourceAreaInfo::new(area_name, dims.v_size(), dims.u_size()));
		}

		LayerSourceAreas {
			map: map,
			index: area_names.clone(),	
		}
	}

	fn len(&self) -> usize {
		assert!(self.map.len() == self.index.len());		
		self.map.len()
	}

	fn get_dim_sizes(&self, area_name: &'static str) -> (u32, u32) {
		let area_info = &self.map[area_name];
		(area_info.v_size, area_info.u_size)
	}

	fn area_info_by_idx(&self, idx: u8) -> SourceAreaInfo {
		assert!((idx as usize) < self.len());
		self.map[self.index[idx as usize]].clone()
	}
}


#[derive(Clone)]
struct SourceAreaInfo {
	name: &'static str,
	v_size: u32,
	u_size: u32,
}

impl SourceAreaInfo {
	fn new(name: &'static str, v_size: u32, u_size: u32) -> SourceAreaInfo {
		SourceAreaInfo { name: name, v_size: v_size, u_size: u_size }
	}
}


fn literal_list<T: Display>(vec: &Vec<T>) -> String {
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

fn calc_scale(src_dim: u32, tar_dim: u32) -> Result<u8, &'static str> {
	// let scale_incr = if src_dim >= 16 { src_dim / 16 } 
	// 	else if src_dim > 0 { 1 }
	// 	else { panic!("area_map::calc_scale(): Source dimension cannot be zero.") };

	let src_dim = (src_dim as usize) * 16;
	let tar_dim = (tar_dim as usize) * 16;

	let scale_incr = match tar_dim {
		0 => return Err("Target area dimension cannot be zero."),
		1...15 => 1,
		_ => tar_dim / 16,
	};

	return match src_dim / scale_incr {
		0 => return Err("Source area dimension cannot be zero."),
		s @ 1...255 => Ok(s as u8),
		_ => return Err("Source area cannot have a dimension more than 16 times target area dimension."),
	}
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
pub fn axn_idz_2d(axn_slc: u8, columns: u32, hrz_demarc: u8) -> u32 {
	let mut axn_idx: u32 = if axn_slc < hrz_demarc {
		(axn_slc as u32 * columns)
	} else {
		(hrz_demarc as u32 * columns) + (cmn::SYNAPSE_SPAN_RHOMBAL_AREA * (axn_slc as u32 - hrz_demarc as u32))
	};

	axn_idx + cmn::AXON_MARGIN_SIZE as u32
}

fn clone_rewrap_layer(pl_ref_opt: Option<&Protolayer>) -> Option<Protolayer> {
	match pl_ref_opt {
		Some(pl_ref) => Some(pl_ref.clone()),
		None => None,
	}
}

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
