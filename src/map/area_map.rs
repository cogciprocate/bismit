// use num::{ Num };
use std::fmt::{ Display };
use std::ops::{ Range };
// use std::collections::{ HashMap };
//use std::num::ToString;

use ocl::{ BuildConfig, BuildOpt };
use proto::{ ProtolayerMaps, ProtolayerMap, Protolayer, ProtoareaMaps, ProtoareaMap };
use cmn::{ self, CorticalDims };
use map::{ self, SliceMap, SliceDims, LayerFlags, LayerMap, LayerSourceAreas, SourceAreaInfo };
// use map::slice_map;

// 	AREAMAP { }:
// 		- Move in functionality from the 'execution phase' side of ProtoareaMap and ProtolayerMap.
//		- Leave the 'init phase' stuff to the proto-*s.
#[derive(Clone)]
pub struct AreaMap {
	area_name: &'static str,
	dims: CorticalDims,
	ia_cache: InterAreaInfoCache,
	slices: SliceMap,
	layers: LayerMap,
	hrz_demarc: u8,

	// TODO: Create maps for each aspect which have their own types and are queryable 
	// into sub-lists of the same type
	// layers: LayerMap
	// slices: SliceMap
	// etc...
	// other new types: TuftMap/CellMap

	pamap: ProtoareaMap,
	plmap: ProtolayerMap,
}

impl AreaMap {
	pub fn new(pamap: &ProtoareaMap, plmaps: &ProtolayerMaps, pamaps: &ProtoareaMaps) -> AreaMap {
		let pamap = pamap.clone();			
		let mut plmap = plmaps[pamap.layer_map_name].clone();
		plmap.freeze(&pamap);

		println!("{mt}AREAMAP::NEW(): area name: {}, eff areas: {:?}, aff areas: {:?}", pamap.name, 
			pamap.eff_areas, pamap.aff_areas, mt = cmn::MT);

		let dims = pamap.dims().clone_with_depth(plmap.depth_total());		
		let hrz_demarc = plmap.hrz_demarc();
		// let area_name = pamap.name;

		let ia_cache = InterAreaInfoCache::new(
			&dims,
			&pamap.eff_areas, // EFF AREAS
			&pamap.aff_areas, // AFF AREAS
			plmap.layer_with_flag(map::FF_IN), // AFF INPUT LAYER			
			plmap.layer_with_flag(map::FB_IN), // EFF INPUT LAYER
			plmap.layer_with_flag(map::FF_OUT), // AFF & EFF OUTPUT LAYER
			pamaps,
		);

		let slices = SliceMap::new(&dims, &pamap, &plmap, &ia_cache);

		let layers = LayerMap::new(&pamap, &plmap, pamaps, plmaps);

		AreaMap {
			area_name: pamap.name,
			dims: dims,
			ia_cache: ia_cache,
			slices: slices,
			layers: layers,
			hrz_demarc: hrz_demarc,
			pamap: pamap,
			plmap: plmap,
			//emsg: emsg,
		}
	}	

	pub fn slc_src_area_dims(&self, slc_id: u8, layer_flags: LayerFlags) -> &SliceDims {
		//self.plmap.layer_with_flag(layer_flags).expect("Cannot find layer").layer_base_slc()

		// GET SOURCE AREA DIMS!
		// 		- get layer name
		// 		- get layer slice list
		// 		- return 
		match self.ia_cache.src_area_for_slc(slc_id, layer_flags) {
			Some(ref area) => area.dims(),
			None => panic!("AreaMap:: Cannot find a slice with id: `{}` and flags: `{:?}`.", 
				slc_id, layer_flags),
		}
	}

	// pub fn layer_src_info_by_flag(&self, layer_flags: LayerFlags) -> Vec<(&'static str, LayerFlags)> {
	// 	self.layers.layer_src_info_by_flag(layer_flags)
	// }

	// LAYER_SOURCE_AREA_INFO(): DEPRICATE THIS UGLY BASTARD
	pub fn layer_source_area_info(&self, layer_flags: LayerFlags) -> (&Protolayer, u32) {
		let emsg = format!("\nAreaMap:: `{:?}` flag not set for any layer in area: `{}`.", 
			layer_flags, self.area_name);

		if layer_flags.contains(map::FF_IN) {
			//println!("##### AFF IN CONTAINED");
			match &self.ia_cache.aff_in_layer { 
				&Some(ref l) => (l, self.ia_cache.eff_areas.axns_sum()), 
				&None => panic!(emsg), 
			}
		} else if layer_flags.contains(map::FB_IN) {
			//println!("##### EFF IN CONTAINED");
			match &self.ia_cache.eff_in_layer { 
				&Some(ref l) => (l, self.ia_cache.aff_areas.axns_sum()), 
				&None => panic!(emsg), 
			}
		} else if layer_flags.contains(map::FF_OUT) {
			//println!("##### AFF OUT CONTAINED");
			match &self.ia_cache.out_layer { 
				&Some(ref l) => (l, self.dims.columns()), 
				&None => panic!(emsg), 
			} 
		} else if layer_flags.contains(map::FB_OUT) { // REDUNDANT (MERGE WITH ABOVE)
			//println!("##### EFF OUT CONTAINED");
			match &self.ia_cache.out_layer { 
				&Some(ref l) => (l, self.dims.columns()), 
				&None => panic!(emsg), 
			} 
		} else { 
			//println!("##### CALCULATING LAYER LENGTH OLD SCHOOL");
			let l = self.plmap.layer_with_flag(layer_flags).expect(&emsg);
			//let layer_len = axn_idz_2d(l.base_slc_id, self.dims.columns(), self.hrz_demarc);
			let layer_len = self.dims.columns();
			(l, layer_len)
		}
	}

	// ADD OPTION FOR MORE CUSTOM KERNEL FILES OR KERNEL LINES
	pub fn gen_build_options(&self) -> BuildConfig {
		let mut build_options = cmn::base_build_options()
			.cmplr_def("HORIZONTAL_AXON_ROW_DEMARCATION", self.hrz_demarc as i32)
			.cmplr_def("AXN_SLC_COUNT", self.slices.depth() as i32)
			.bo(BuildOpt::include_def("AXN_SLC_IDZS", literal_list(self.slices.axn_idzs())))
			.bo(BuildOpt::include_def("AXN_SLC_V_SIZES", literal_list(self.slices.v_sizes())))
			.bo(BuildOpt::include_def("AXN_SLC_U_SIZES", literal_list(self.slices.u_sizes())))
			.bo(BuildOpt::include_def("AXN_SLC_V_SCALES", literal_list(self.slices.v_scales())))
			.bo(BuildOpt::include_def("AXN_SLC_U_SCALES", literal_list(self.slices.u_scales())))
		;

		// CUSTOM KERNELS
		match self.pamap.filters {
			Some(ref protofilters) => {
				for pf in protofilters.iter() {
					match pf.cl_file_name() {
						Some(ref clfn)  => {							
							build_options.add_kern_file(format!("{}/{}", cmn::cl_root_path(), clfn.clone()))
							// build_options.add_kern_file(format!("{}/{}", "cl", clfn.clone()))
						},

						None => (),
					}
				}
			},
			None => (),
		};

		cmn::load_builtin_kernel_files(&mut build_options);

		build_options
	}

	pub fn axn_base_slc_ids_by_flag(&self, layer_flags: LayerFlags) -> Vec<u8> {
		// self.plmap.layer_with_flag(layer_flags).expect("Cannot find layer").base_slc()
		let layers = self.plmap.layers_with_flags(layer_flags);
		let mut slc_ids = Vec::with_capacity(layers.len());

		for &layer in layers.iter() {
			slc_ids.push(layer.base_slc());
		}

		slc_ids
	}

	pub fn axn_range_by_flag(&self, layer_flags: LayerFlags) -> Range<u32> {				
		let (layer, layer_len) = self.layer_source_area_info(layer_flags);
		// let layers = self.layers.layer_info_by_flag(layer_flags);
		// assert!(layers.len() == 1, "AreaMap::axn_range_by_flag(): [FIXME]: Axon range \
		// 	can not yet be calculated for more than one source layer / area. Flags: {:?}",
		// 	layer_flags);

		let layer_idz = self.axn_idz(layer.base_slc_id);

		layer_idz..(layer_idz + layer_len)
	}

	// [TODO] Layer source area system needs rework.
	pub fn output_layer_info_by_flag(&self) -> Vec<(LayerFlags, u32)> {
		let layers = self.plmap.layers_with_flags(map::OUTPUT);
		let mut layer_info = Vec::with_capacity(layers.len());
		
		for &layer in layers.iter() {
			layer_info.push((layer.flags, self.dims.columns()));
		}

		layer_info
	}

	pub fn aff_areas(&self) -> &Vec<&'static str> {
		&self.pamap.aff_areas
	}

	pub fn eff_areas(&self) -> &Vec<&'static str> {
		&self.pamap.eff_areas
	}

	pub fn area_name(&self) -> &'static str {
		self.area_name
	}

	pub fn proto_area_map(&self) -> &ProtoareaMap {
		&self.pamap
	}

	pub fn proto_layer_map(&self) -> &ProtolayerMap {
		&self.plmap
	}

	pub fn axn_idz(&self, slc_id: u8) -> u32 {
		self.slices.idz(slc_id)
	}

	pub fn slices(&self) -> &SliceMap {
		&self.slices
	}

	pub fn layers(&self) -> &LayerMap {
		&self.layers
	}

	pub fn dims(&self) -> &CorticalDims {
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



#[derive(Clone)]
// NEEDS RENAME & INTEGRATION WITH / CONVERSION TO LAYERMAP
// [FIXME] TODO: DEPRICATE
pub struct InterAreaInfoCache {
	pub eff_areas: LayerSourceAreas, // eff. areas -> aff. input layer	
	pub aff_areas: LayerSourceAreas, // aff. areas -> eff. input layer
	pub aff_in_layer: Option<Protolayer>,
	pub eff_in_layer: Option<Protolayer>,
	pub out_layer: Option<Protolayer>,
}

impl InterAreaInfoCache {
	pub fn new(
				area_dims: &CorticalDims,
				eff_area_names: &Vec<&'static str>, 
				aff_area_names: &Vec<&'static str>, 
				aff_in_layer: Option<&Protolayer>, 				
				eff_in_layer: Option<&Protolayer>,
				out_layer: Option<&Protolayer>,
				pamaps: &ProtoareaMaps,
			) -> InterAreaInfoCache 
	{
		let eff_areas = LayerSourceAreas::new(area_dims, eff_area_names, pamaps);
		let aff_areas = LayerSourceAreas::new(area_dims, aff_area_names, pamaps);

		InterAreaInfoCache { 
			eff_areas: eff_areas, 			
			aff_areas: aff_areas, 
			aff_in_layer: aff_in_layer.map(|l| l.clone()),
			eff_in_layer: eff_in_layer.map(|l| l.clone()),
			out_layer: out_layer.map(|l| l.clone()),
		}
	}

	pub fn src_area_for_slc(&self, slc_id: u8, flags: LayerFlags) -> Option<&SourceAreaInfo> {
		let (layer_src_areas, layer_opt) = if flags.contains(map::FF_IN) {
			// println!("##### AFF -> slc_id: {}, flags: {:?}", slc_id, flags);
			(&self.eff_areas, &self.aff_in_layer)			
		} else if flags.contains(map::FB_IN) {			
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

// fn clone_rewrap_layer(pl_ref_opt: Option<&Protolayer>) -> Option<Protolayer> {
// 	match pl_ref_opt {
// 		Some(pl_ref) => Some(pl_ref.clone()),
// 		None => None,
// 	}
// }



#[cfg(test)]
pub mod tests {
	use std::fmt::{ Display, Formatter, Result as FmtResult };
	use super::{ AreaMap };

	pub trait AreaMapTest {
		fn axn_idx(&self, slc_id: u8, v_id: u32, v_ofs: i8, u_id: u32, u_ofs: i8) 
				-> Result<u32, &'static str>;
		fn axn_col_id(&self, slc_id: u8, v_id_unscaled: u32, v_ofs: i8, u_id_unscaled: u32, u_ofs: i8)
				-> Result<u32, &'static str>;
		// fn print_slc_map(&self);
	}

	impl AreaMapTest for AreaMap {
		/* AXN_IDX(): Some documentation for this can be found in bismit.cl
		 		Basically all we're doing is scaling up or down the v and u coordinates based on a predetermined scaling factor. The scaling factor only applies when a foreign cortical area is a source for the axon's slice AND is a different size than the local cortical area. The scale factor is based on the relative size of the two areas. Most of the time the scaling factor is 1:1 (scale factor of 16). The algorithm below for calculating an axon index is the same as the one in the kernel and gives precisely the same results.
		*/
		fn axn_idx(&self, slc_id: u8, v_id_unscaled: u32, v_ofs: i8, u_id_unscaled: u32, u_ofs: i8)
				-> Result<u32, &'static str> 
		{
			let v_scale = self.slices.v_scales()[slc_id as usize];
			let u_scale = self.slices.u_scales()[slc_id as usize];

			let v_id_scaled = (v_id_unscaled * v_scale) / 16;
			let u_id_scaled = (u_id_unscaled * u_scale) / 16;

			let slc_count = self.slices().depth();
			let v_size = self.slices.v_sizes()[slc_id as usize];
			let u_size = self.slices.u_sizes()[slc_id as usize];

			// println!("AreaMapTest::axn_idx(): \
			// axn_idz: {}, slc_count: {}, slc_id: {}, v_scale: {}, v_size: {}, \
			// v_id_unscaled: {}, v_id_scaled: {}, v_ofs: {}, u_scale: {}, u_size: {}, \
			// u_id_unscaled: {}, u_id_scaled: {}, u_ofs: {}", self.axn_idz(slc_id), 
			// slc_count, slc_id, v_scale, v_size, v_id_unscaled, v_id_scaled, v_ofs, 
			// u_scale, u_size, u_id_unscaled, u_id_scaled, u_ofs);

			if coords_are_safe(slc_count, slc_id, v_size, v_id_scaled, v_ofs, u_size, u_id_scaled, u_ofs) {
				Ok(axn_idx_unsafe(self.axn_idz(slc_id), v_id_scaled, v_ofs, u_size, u_id_scaled, u_ofs))
			} else {
				Err("Axon coordinates invalid.")
			}
		}

		fn axn_col_id(&self, slc_id: u8, v_id_unscaled: u32, v_ofs: i8, u_id_unscaled: u32, u_ofs: i8)
				-> Result<u32, &'static str> 
		{
			let v_scale = self.slices.v_scales()[slc_id as usize];
			let u_scale = self.slices.u_scales()[slc_id as usize];

			let v_id_scaled = (v_id_unscaled * v_scale) / 16;
			let u_id_scaled = (u_id_unscaled * u_scale) / 16;

			let v_size = self.slices.v_sizes()[slc_id as usize];
			let u_size = self.slices.u_sizes()[slc_id as usize];

			// Make sure v and u are safe (give fake slice info to coords_are_safe()):
			if coords_are_safe(1, 0, v_size, v_id_scaled, v_ofs, u_size, u_id_scaled, u_ofs) {
				// Give a fake, zero idz (since this is a column id we're returning):
				Ok(axn_idx_unsafe(0, v_id_scaled, v_ofs, u_size, u_id_scaled, u_ofs))
			} else {
				Err("Axon coordinates invalid.")
			}
		}
		// fn print_slc_map(&self) {
		// 	print!("\nslice map: ");

		// 	for i in 0..self.slices.slc_count() {
		// 		print!("[{}: '{}', {}]", i, self.slices.layer_names()[i], self.slices.axn_idzs()[i]);
		// 	}

		// 	print!("\n");
		// }

	}

	impl Display for AreaMap {
	    fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
	        write!(fmtr, "area slices: {}", self.slices)
	    }
	}

	pub fn coords_are_safe(slc_count: u8, slc_id: u8, v_size: u32, v_id: u32, v_ofs: i8, 
			u_size: u32, u_id: u32, u_ofs: i8
		) -> bool 
	{
		(slc_id < slc_count) && coord_is_safe(v_size, v_id, v_ofs) 
			&& coord_is_safe(u_size, u_id, u_ofs)
	}

	pub fn coord_is_safe(dim_size: u32, coord_id: u32, coord_ofs: i8) -> bool {
		let coord_ttl = coord_id as i64 + coord_ofs as i64;
		(coord_ttl >= 0) && (coord_ttl < dim_size as i64)
	}

	pub fn axn_idx_unsafe(idz: u32, v_id: u32, v_ofs: i8, u_size: u32, u_id: u32, u_ofs: i8) -> u32 {
		let v = v_id as i64 + v_ofs as i64;
		let u = u_id as i64 + u_ofs as i64;
		(idz as i64 + (v * u_size as i64) + u) as u32
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
