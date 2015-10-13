// use num::{ Num };
// use std::fmt::{ Display };
// use std::ops::{ Range };
// use std::collections::{ HashMap };
//use std::num::ToString;

use ocl::{ /*BuildOptions, BuildOption,*/ EnvoyDimensions };
use proto::{ /*layer, ProtoLayerMaps,*/ ProtoLayerMap, /*Protolayer, ProtolayerFlags, ProtoAreaMaps,*/ ProtoAreaMap };
use cmn::{ self, CorticalDimensions, SliceDimensions, HexTilePlane };
use map::{ InterAreaInfoCache };
use map::area_map;


// pub fn axn_idz_2d(axn_slc: u8, columns: u32, hrz_demarc: u8) -> u32 {
// 	let mut axn_idx: u32 = if axn_slc < hrz_demarc {
// 		(axn_slc as u32 * columns)
// 	} else {
// 		(hrz_demarc as u32 * columns) + (cmn::SYNAPSE_SPAN_RHOMBAL_AREA * (axn_slc as u32 - hrz_demarc as u32))
// 	};

// 	axn_idx + cmn::AXON_MARGIN_SIZE as u32
// }



#[derive(Debug, Clone)]
pub struct SliceMap {
	axn_idzs: Vec<u32>,
	layer_names: Vec<&'static str>,
	v_sizes: Vec<u32>,
	u_sizes: Vec<u32>,
	v_scales: Vec<u32>,
	u_scales: Vec<u32>,
	dims: Vec<SliceDimensions>,
	physical_len: u32
}

impl SliceMap {
	pub fn new(area_dims: &CorticalDimensions, pamap: &ProtoAreaMap, plmap: &ProtoLayerMap, 
					ia_cache: &InterAreaInfoCache,
	) -> SliceMap {		
		let proto_slc_map = plmap.slc_map();

		let mut axn_idzs = Vec::with_capacity(proto_slc_map.len());
		let mut layer_names = Vec::with_capacity(proto_slc_map.len());
		let mut v_scales = Vec::with_capacity(proto_slc_map.len());
		let mut u_scales = Vec::with_capacity(proto_slc_map.len());
		let mut v_sizes = Vec::with_capacity(proto_slc_map.len());
		let mut u_sizes = Vec::with_capacity(proto_slc_map.len());
		let mut dims = Vec::with_capacity(proto_slc_map.len());

		/*=============================================================================
		=================================  ================================
		=============================================================================*/

		//eff_in_layer_base_slc_id = ia_cache.eff_in_layer_name
		let mut axn_idz_ttl = 0u32;

		for (&slc_id, &layer_name) in proto_slc_map.iter() {
			// CALCULATE SCALE FOR V AND U
			let layer = &plmap.layers()[layer_name];
			let src_area_opt = ia_cache.src_area_for_slc(slc_id, layer.flags);

			let slc_dims = match src_area_opt {
				Some(src_area) => {
					//let src_area = src_area.unwrap();					

					//let src_area_name = src_area.name

					//let src_v_size = src_v_size * 4;
					//let src_u_size = src_u_size * 4;

					// let v_scl = calc_scale(src_dims.v_size(), area_dims.v_size()).expect(
					// 	&format!("\nSliceMap::new(): Error processing {} for area: '{}', layer: '{}' \
					// 	source area: {}", "v_size", pamap.name, layer_name, src_area_name));
					// let u_scl = calc_scale(src_dims.u_size(), area_dims.u_size()).expect(
					// 	&format!("\nSliceMap::new(): Error processing {} for area: '{}', layer: '{}' \
					// 	source area: {}", "u_size", pamap.name, layer_name, src_area_name));

					// let (v_scl, u_scl) = slc_dims.calc_scale(area_dims).expect(
					// 	&format!("\nSliceMap::new(): Error calculating scale for area: '{}', layer: '{}' \
					// 	source area: {}", pamap.name, layer_name, src_area.name));

					let slc_dims = src_area.dims.clone();

					println!("{}SLICEMAP::NEW(): Processing inter-area slice '{}': slc_id: {}, src_area_name: {}, \
						v_size: {}, u_size: {}.", cmn::MT, layer_name, slc_id, src_area.name,
						slc_dims.v_size(), slc_dims.u_size());

					slc_dims
				},

				None =>	SliceDimensions::new(area_dims, None).unwrap(), // 100%
			};

			//axn_idzs.push(axn_idz_2d(slc_id, area_dims.columns(), plmap.hrz_demarc()));
			axn_idzs.push(axn_idz_ttl);
			axn_idz_ttl += slc_dims.len();

			layer_names.push(layer_name);
			v_sizes.push(slc_dims.v_size());
			u_sizes.push(slc_dims.u_size());
			v_scales.push(slc_dims.v_scale());
			u_scales.push(slc_dims.u_scale());			
			dims.push(slc_dims);
		}

		assert_eq!(axn_idzs.len(), layer_names.len());
		assert_eq!(axn_idzs.len(), dims.len());
		assert_eq!(axn_idzs.len(), v_sizes.len());
		assert_eq!(axn_idzs.len(), u_sizes.len());
		assert_eq!(axn_idzs.len(), v_scales.len());
		assert_eq!(axn_idzs.len(), u_scales.len());

		SliceMap {
			axn_idzs: axn_idzs,
			layer_names: layer_names,
			dims: dims,
			v_sizes: v_sizes,
			u_sizes: u_sizes,	
			v_scales: v_scales,
			u_scales: u_scales,	
			physical_len: axn_idz_ttl,		
		}
	}

	pub fn print_debug(&self) {
		//let mini_tab = "   "; // 3 spaces

		println!(
			"\n{mt}SLICEMAP::PRINT_DEBUG(): Area slices: \
			\n{mt}{mt}layer_names: {:?}, \
			\n{mt}{mt}axn_idzs: {:?}(literal: '{}'), \
			\n{mt}{mt}v_sizes: {:?}(literal: '{}'), \
			\n{mt}{mt}u_sizes: {:?}(literal: '{}'), \
			\n{mt}{mt}v_scales: {:?}(literal: '{}'), \
			\n{mt}{mt}u_scales: {:?}(literal: '{}')", 
			self.layer_names, 
			self.axn_idzs, area_map::literal_list(&self.axn_idzs), 
			self.v_sizes, area_map::literal_list(&self.v_sizes), 
			self.u_sizes, area_map::literal_list(&self.u_sizes), 
			self.v_scales, area_map::literal_list(&self.v_scales), 
			self.u_scales, area_map::literal_list(&self.u_scales), 
			mt = cmn::MT
		);

		println!("");
		// for i in 0..self.axn_idzs.len() {

		// }
	}

	pub fn idz(&self, slc_id: u8) -> u32 {
		self.axn_idzs[slc_id as usize]
	}

	pub fn layer_name(&self, slc_id: u8) -> &'static str {
		self.layer_names[slc_id as usize]
	}

	pub fn slc_count(&self) -> usize {
		self.axn_idzs.len() 
	}

	pub fn depth(&self) -> u8 {
		self.axn_idzs.len() as u8
	}

	pub fn axn_count(&self) -> u32 {
		self.physical_len
	}

	pub fn axn_idzs(&self) -> &Vec<u32> {
		&self.axn_idzs
	}

	pub fn layer_names(&self) -> &Vec<&'static str> {
		&self.layer_names
	}

	pub fn v_sizes(&self) -> &Vec<u32> {
		&self.v_sizes
	}

	pub fn u_sizes(&self) -> &Vec<u32> {
		&self.u_sizes
	}

	pub fn v_scales(&self) -> &Vec<u32> {
		&self.v_scales
	}

	pub fn u_scales(&self) -> &Vec<u32> {
		&self.u_scales
	}

	pub fn dims(&self) -> &Vec<SliceDimensions> {
		&self.dims
	}
}

impl EnvoyDimensions for SliceMap {
	/* PHYSICAL_LEN(): ROUND CORTICAL_LEN() UP TO THE NEXT PHYSICAL_INCREMENT */
	fn len(&self) -> u32 {
		self.axn_count()
	}
}


