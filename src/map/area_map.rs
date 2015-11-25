use std::fmt::{ Display };
use std::ops::{ Range }; 
use std::collections::{ BTreeMap };

use ocl::{ BuildConfig, BuildOpt };
use proto::{ ProtolayerMaps, ProtolayerMap, ProtoareaMaps, ProtoareaMap, RegionKind, Protofilter,
	DendriteKind };
use cmn::{ self, CorticalDims };
use map::{ self, SliceMap, LayerTags, LayerMap };

// 	AREAMAP { }:
// 		- Move in functionality from the 'execution phase' side of ProtoareaMap and ProtolayerMap.
//		- Leave the 'init phase' stuff to the proto-*s.
#[derive(Clone)]
pub struct AreaMap {
	area_name: &'static str,
	dims: CorticalDims,
	slices: SliceMap,
	layers: LayerMap,
	hrz_demarc: u8,
	eff_areas: Vec<&'static str>,
	aff_areas: Vec<&'static str>,

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
			pamap.eff_areas(), pamap.aff_areas(), mt = cmn::MT);

		let dims = pamap.dims().clone_with_depth(plmap.depth_total());		
		let hrz_demarc = plmap.hrz_demarc();

		let layers = LayerMap::new(&pamap, &plmap, pamaps, plmaps);

		let slices = SliceMap::new(&dims, &pamap, &plmap, &layers);

		AreaMap {
			area_name: pamap.name,
			dims: dims,
			slices: slices,
			layers: layers,
			hrz_demarc: hrz_demarc,
			eff_areas: pamap.eff_areas().clone(),
			aff_areas: pamap.aff_areas().clone(),
			pamap: pamap,
			plmap: plmap,
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

	// NEW
	pub fn layer_name_by_tags(&self, layer_tags: LayerTags) -> &'static str {
		let layer_info = self.layers.layer_info_by_tags(layer_tags);
		assert_eq!(layer_info.len(), 1);
		layer_info[0].name()
	}

	// NEW - REVAMP
	pub fn layer_slc_ids(&self, layer_names: Vec<&'static str>) -> Vec<u8> {
		// self.plmap.slc_ids(layer_names)
		let mut slc_ids = Vec::new();

		for layer_name in layer_names.iter() {
			let l = &self.plmap.layers()[layer_name];
				for i in l.base_slc_id()..(l.base_slc_id() + l.depth()) {
					slc_ids.push(i);
				}
		}

		slc_ids
	}

	// UPDATE
	pub fn layer_src_slc_ids(&self, layer_name: &'static str, den_type: DendriteKind) -> Vec<u8> {
		let src_lyr_names = self.plmap.layers()[&layer_name].src_lyr_names(den_type);
		
		self.layer_slc_ids(src_lyr_names)
 	}

	// UPDATE
	pub fn aff_out_slcs(&self) -> Vec<u8> {
		let mut output_slcs: Vec<u8> = Vec::with_capacity(8);
 		
 		for (layer_name, layer) in self.plmap.layers().iter() {
 			if (layer.tags() & map::FF_OUT) == map::FF_OUT {
 				let v = self.plmap.slc_ids(vec![layer.name()]);
 				output_slcs.push_all(&v);
 			}
 		}

		output_slcs	
	}

	// UPDATE - DEPRICATE
	pub fn axn_base_slc_ids_by_tags(&self, layer_tags: LayerTags) -> Vec<u8> {
		let layers = self.plmap.layers_with_tags(layer_tags);
		let mut slc_ids = Vec::with_capacity(layers.len());

		for &layer in layers.iter() {
			slc_ids.push(layer.base_slc());
		}

		slc_ids
	}

	pub fn slc_map(&self) -> BTreeMap<u8, &'static str> {
		self.plmap.slc_map()
	}

	// NEW
	pub fn axn_range_by_tags(&self, layer_tags: LayerTags) -> Range<u32> {				
		let layers = self.layers.layer_info_by_tags(layer_tags);
		assert!(layers.len() == 1, "AreaMap::axn_range_by_tags(): Axon range \
			can not be calculated for more than one layer at a time. Flags: {:?}",
			layer_tags);

		let layer_idz = self.axn_idz(layers[0].slc_range().start);
		let layer_len = layers[0].axn_count();

		debug_assert!({
				let slc_idm = layers[0].slc_range().start + layers[0].depth() - 1;
				let slc_len = self.slices.slc_axn_count(slc_idm);
				let axn_idz = self.axn_idz(slc_idm);
				let axn_idn = axn_idz + slc_len;
				// println!("\n\n# (layer_idz, layer_len) = ({}, {}), axn_idn = {}, \
				// 	slc_len = {}, axn_idz = {}, \n# layer: {:?}\n", 
				// 	layer_idz, layer_len, axn_idn, slc_len, axn_idz, layers[0]);
				(layer_idz + layer_len) == axn_idn
			}, "AreaMap::axn_range(): Axon index mismatch.");

		layer_idz..(layer_idz + layer_len)
	}

	// [TODO] Layer source area system needs rework.
	pub fn output_layer_info_by_tags(&self) -> Vec<(LayerTags, u32)> {
		let layers = self.plmap.layers_with_tags(map::OUTPUT);
		let mut layer_info = Vec::with_capacity(layers.len());
		
		for &layer in layers.iter() {
			layer_info.push((layer.tags(), self.dims.columns()));
		}

		layer_info
	}

	// NEW
	pub fn slc_src_layer_dims(&self, slc_id: u8, layer_tags: LayerTags) -> Option<&CorticalDims> {
		self.layers.slc_src_layer_info(slc_id, layer_tags).map(|sli| sli.dims())
	}

	// DEPRICATE
	pub fn aff_areas(&self) -> &Vec<&'static str> {
		&self.aff_areas
	}

	// DEPRICATE
	pub fn eff_areas(&self) -> &Vec<&'static str> {
		&self.eff_areas
	}

	pub fn area_name(&self) -> &'static str {
		self.area_name
	}

	// DEPRICATE
	pub fn proto_area_map(&self) -> &ProtoareaMap {
		&self.pamap
	}

	// DEPRICATE
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

	// UPDATE - DEPRICATE
	pub fn filters(&self) -> &Option<Vec<Protofilter>> {
		&self.pamap.filters
	}

	pub fn dims(&self) -> &CorticalDims {
		&self.dims
	}

	pub fn hrz_demarc(&self) -> u8 {
		self.hrz_demarc
	}

	// pub fn lm_name_tmp(&self) -> &'static str {
	// 	self.plmap.name
	// }

	// UPDATE - DEPRICATE
	pub fn lm_kind_tmp(&self) -> &RegionKind {
		&self.plmap.kind
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


#[cfg(test)]
pub mod tests {
	use std::fmt::{ Display, Formatter, Result as FmtResult };
	use super::{ AreaMap };

	pub trait AreaMapTest {
		fn axn_idx(&self, slc_id: u8, v_id: u32, v_ofs: i8, u_id: u32, u_ofs: i8) 
				-> Result<u32, &'static str>;
		fn axn_col_id(&self, slc_id: u8, v_id_unscaled: u32, v_ofs: i8, u_id_unscaled: u32, u_ofs: i8)
				-> Result<u32, &'static str>;
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
