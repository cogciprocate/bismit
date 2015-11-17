// use num::{ Num };
// use std::fmt::{ Display };
// use std::ops::{ Range };
use std::collections::{ HashMap };
//use std::num::ToString;

// use ocl::{ BuildConfig, BuildOption };
use proto::{ Protolayer, ProtoAreaMaps };
use cmn::{ /*self,*/ CorticalDims, SliceDims };
use map::{ self, LayerFlags };

pub struct LayerMap {
	map: HashMap<&'static str, LayerInfo>,
}

impl LayerMap {

}

pub struct LayerInfo {
	name: &'static str,
	slices: Vec<u8>,
}

#[derive(Clone)]
// NEEDS RENAME & INTEGRATION WITH / CONVERSION TO LAYERMAP
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
				pamaps: &ProtoAreaMaps,
			) -> InterAreaInfoCache 
	{
		let eff_areas = LayerSourceAreas::new(area_dims, eff_area_names, pamaps);
		let aff_areas = LayerSourceAreas::new(area_dims, aff_area_names, pamaps);

		InterAreaInfoCache { 
			eff_areas: eff_areas, 			
			aff_areas: aff_areas, 
			aff_in_layer: clone_rewrap_layer(aff_in_layer), 
			eff_in_layer: clone_rewrap_layer(eff_in_layer),
			out_layer: clone_rewrap_layer(out_layer),
		}
	}

	pub fn src_area_for_slc(&self, slc_id: u8, flags: LayerFlags) -> Option<&SourceAreaInfo> {
		let (layer_src_areas, layer_opt) = if flags.contains(map::AFFERENT_INPUT) {
			// println!("##### AFF -> slc_id: {}, flags: {:?}", slc_id, flags);
			(&self.eff_areas, &self.aff_in_layer)			
		} else if flags.contains(map::EFFERENT_INPUT) {			
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
pub struct LayerSourceAreas {
	map: HashMap<&'static str, SourceAreaInfo>,
	index: Vec<&'static str>,
	axns_sum: u32,
}

impl LayerSourceAreas {
	fn new(area_dims: &CorticalDims, src_area_names: &Vec<&'static str>, pamaps: &ProtoAreaMaps) -> LayerSourceAreas {
		let mut map = HashMap::with_capacity(src_area_names.len());

		let mut axns_sum = 0;

		for &src_area_name in src_area_names.iter() {
			let src_area_dims = pamaps.maps()[src_area_name].dims();
			axns_sum += src_area_dims.columns();
			map.insert(src_area_name, SourceAreaInfo::new(area_dims, src_area_name, src_area_dims));
		}

		LayerSourceAreas {
			map: map,
			index: src_area_names.clone(),
			axns_sum: axns_sum,
		}
	}

	fn len(&self) -> usize {
		assert!(self.map.len() == self.index.len());		
		self.map.len()
	}

	pub fn axns_sum(&self) -> u32 {
		self.axns_sum
	}

	fn area_dims(&self, area_name: &'static str) -> &SliceDims {
		let area_info = &self.map[area_name];
		//(area_info.dims.v_size, area_info.dims.u_size)
		&area_info.dims
	}

	fn area_info_by_idx(&self, idx: u8) -> &SourceAreaInfo {
		assert!((idx as usize) < self.len());
		&self.map[self.index[idx as usize]]
	}
}


#[derive(Clone)]
// TODO: DEPRICATE IN FAVOR OF SLICE MAP
struct SourceAreaInfo {
	pub name: &'static str,
	pub dims: SliceDims,
	// pub v_size: u32,
	// pub u_size: u32,
}

impl SourceAreaInfo {
	fn new(area_dims: &CorticalDims, src_area_name: &'static str, src_area_dims: &CorticalDims
		) -> SourceAreaInfo 
	{
		let slc_dims = SliceDims::new(area_dims, Some(src_area_dims)).unwrap();
		SourceAreaInfo { name: src_area_name, dims: slc_dims /*v_size: v_size, u_size: u_size*/ }
	}

	pub fn dims(&self) -> &SliceDims {
		&self.dims
	}
}


fn clone_rewrap_layer(pl_ref_opt: Option<&Protolayer>) -> Option<Protolayer> {
	match pl_ref_opt {
		Some(pl_ref) => Some(pl_ref.clone()),
		None => None,
	}
}

