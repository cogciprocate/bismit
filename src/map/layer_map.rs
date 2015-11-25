// use std::collections::{ HashMap };
use std::ops::{ Range };
use std::slice::{ Iter };

use proto::{ Protolayer, ProtoareaMap, ProtoareaMaps, ProtolayerMap, ProtolayerMaps, ProtolayerKind, DendriteKind };
use cmn::{ self, CorticalDims };
use map::{ self, LayerTags, };


#[derive(Clone)]
// [FIXME]: TODO: Add caches.
pub struct LayerMap {
	area_name: &'static str,
	index: Vec<LayerInfo>,
}

impl LayerMap {
	pub fn new(pamap: &ProtoareaMap, plmap: &ProtolayerMap, pamaps: &ProtoareaMaps, 
				plmaps: &ProtolayerMaps) -> LayerMap 
	{
		println!("{mt}LAYERMAP::NEW()...", mt = cmn::MT);

		let mut index = Vec::with_capacity(plmap.layers().len());

		for (pl_name, pl) in plmap.layers().iter() {
			index.push(LayerInfo::new(pl, pamap, pamaps, plmaps));
		}

		// println!("{mt}{mt}LAYERMAP::NEW(): index: {:?}", index, mt = cmn::MT);
		LayerMap { area_name: pamap.name, index: index }
	}

	// [FIXME] TODO: Cache results.
	pub fn layer_info_by_tags(&self, tags: LayerTags) -> Vec<&LayerInfo> {
		self.index.iter().filter(|li| li.tags.contains(tags)).map(|li| li).collect()
	}

	// [FIXME] TODO: Cache results. Use iterator mapping and filtering.
	pub fn layer_src_info_by_tags(&self, tags: LayerTags) -> Vec<&SourceLayerInfo> {
		let mut src_layers = Vec::with_capacity(8);

		for layer in self.layer_info_by_tags(tags).iter() {
			for src_layer in layer.sources.iter() {
				debug_assert!(src_layer.tags().contains(tags.mirror_io()));
				src_layers.push(src_layer);
			}
		}

		src_layers
	}

	pub fn layer_src_area_names_by_tags(&self, tags: LayerTags) -> Vec<&'static str> {
		self.layer_src_info_by_tags(tags).iter().map(|sli| sli.area_name()).collect()
	}

	pub fn slc_src_layer_info(&self, slc_id: u8, layer_tags: LayerTags) -> Option<&SourceLayerInfo> {
		let mut src_layer_info = Vec::with_capacity(8);
		let layer_info = self.layer_info_by_tags(layer_tags);

		for lyr in layer_info {			
			for src_lyr in lyr.src_info() {
				if slc_id >= src_lyr.dst_slc_range().start 
					&& slc_id < src_lyr.dst_slc_range().end
				{
					src_layer_info.push(src_lyr);
				}
			}
		}

		if src_layer_info.len() == 1 {
			Some(src_layer_info[0])
		} else {
			None
		}
	}

	pub fn iter(&self) -> Iter<map::layer_map::LayerInfo>{
		self.index.iter()
	}
}


// [FIXME]: Consolidate terminology and usage between source-layer layers (cellular)
// and source-area layers (axonal).

#[derive(Clone, Debug)]
pub struct LayerInfo {
	name: &'static str,	
	tags: LayerTags,
	slc_range: Range<u8>,
	sources: Vec<SourceLayerInfo>,
	axn_count: u32,
	protolayer: Protolayer,
}

impl LayerInfo {
	// [FIXME]: TODO: Clean up and optimize.
	pub fn new(protolayer: &Protolayer, pamap: &ProtoareaMap, pamaps: &ProtoareaMaps, 
				plmaps: &ProtolayerMaps) -> LayerInfo {
		let name = protolayer.name();
		let tags = protolayer.tags();
		// let slc_range = protolayer.base_slc_id()..(protolayer.base_slc_id() + protolayer.depth());
		let mut sources = Vec::with_capacity(8);

		let mut next_base_slc_id = protolayer.base_slc_id();
		let mut axn_count = 0;

		// println!("\n{mt}{mt}### LAYER: {:?}, next_base_slc_id: {}, slc_range: {:?}\n", 
		// 	tags, next_base_slc_id, slc_range, mt = cmn::MT);

		// If layer is an input layer, add sources:
		if tags.contains(map::INPUT) {
			let src_areas: Vec<(&'static str, LayerTags)> = 
				pamap.aff_areas().iter().map(|&an| (an, map::FEEDBACK | map::SPECIFIC))
					.chain(pamap.eff_areas().iter().map(|&an| (an, map::FEEDFORWARD | map::SPECIFIC)))
				.chain(pamap.aff_areas().iter().chain(pamap.eff_areas().iter())
					.map(|&an| (an, map::NONSPECIFIC)))
				.collect();				

			// println!("\n{mt}{mt}{mt}### SRC_AREAS: {:?}\n", src_areas, mt = cmn::MT);

			// For each potential source area (aff or eff):
			// - get that area's layers
			// - get the layers with a complimentary flag ('map::OUTPUT' in this case)
			//    - other tags identical
			// - filter out feedback from eff areas and feedforward from aff areas
			// - push what's left to sources
			for (src_area_name, src_area_tags) in src_areas {
				// Our layer must contain the flow direction flag corresponding with the source area.
				if tags.contains(src_area_tags) {
					let src_pamap = pamaps.maps().get(src_area_name).expect("LayerInfo::new()");
					// let src_pamap = ;
					// let src_pamap = match pamaps.maps().get(src_area_name) {
					// 	Some(pm) => pm,
					// 	None => continue,
					// };

					let src_layer_map = &plmaps[src_pamap.layer_map_name];
					let src_layers = src_layer_map.layers_with_tags(tags.mirror_io());

						// println!("\n{mt}{mt}{mt}{mt}### SRC_LAYERS: {:?}\n", src_layers, mt = cmn::MT);

					for src_layer in src_layers.iter() {
						let src_layer_axns = src_pamap.dims().columns()	* src_layer.depth() as u32;

						sources.push(SourceLayerInfo::new(src_area_name, src_pamap.dims(), 
							src_layer.tags(), src_layer_axns, next_base_slc_id, (*src_layer).clone()));						

						// println!("{mt}{mt}LAYERINFO::NEW(layer: '{}'): Adding source layer: \
						// 	src_area_name: '{}', src_area_tags: '{:?}', src_layer_map.name: '{}', \
						// 	src_layer.name: '{}', next_base_slc_id: '{}', depth: '{}', \
						// 	src_layer.tags: '{:?}'", name, src_area_name, src_area_tags, 
						// 	src_layer_map.name, src_layer.name, next_base_slc_id, src_layer.depth(), 
						// 	src_layer.tags, mt = cmn::MT);

						next_base_slc_id += src_layer.depth();
						axn_count += src_layer_axns;
					}
				} 
				// else if tags.contains(map::NONSPECIFIC) && tags.
			} 

		} else {
			next_base_slc_id += protolayer.depth();
			axn_count += pamap.dims().columns() * protolayer.depth() as u32;
		}

		let slc_range = protolayer.base_slc_id()..next_base_slc_id;

		// assert_eq!(next_base_slc_id, slc_range.end);

		sources.shrink_to_fit();

		LayerInfo {
			name: name,
			tags: protolayer.tags(),
			slc_range: slc_range,
			sources: sources,
			axn_count: axn_count,
			protolayer: protolayer.clone(),
		}
	}

	pub fn src_lyr_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
		self.protolayer.src_lyr_names(den_type)
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn kind(&self) -> ProtolayerKind {
		self.protolayer.kind()
	}

	pub fn src_info(&self) -> &Vec<SourceLayerInfo>  {
		&self.sources
	}

	pub fn axn_count(&self) -> u32 {
		self.axn_count
	}

	pub fn slc_range(&self) -> &Range<u8> {
		&self.slc_range
	}

	pub fn depth(&self) -> u8 {
		self.slc_range.len() as u8
	}
}


#[derive(Clone, Debug)]
pub struct SourceLayerInfo {
	area_name: &'static str,
	dims: CorticalDims,
	tags: LayerTags,
	dst_slc_range: Range<u8>,
	protolayer: Protolayer,
}

impl SourceLayerInfo {
	pub fn new(area_name: &'static str, area_dims: &CorticalDims, tags: LayerTags, 
				axn_count: u32, dst_slc_idz: u8, protolayer: Protolayer) -> SourceLayerInfo 
	{
		let dims = area_dims.clone_with_depth(protolayer.depth());
		let dst_slc_range = dst_slc_idz..(dst_slc_idz + protolayer.depth());
		debug_assert_eq!(dims.cells(), axn_count);

		SourceLayerInfo {
			area_name: area_name, 
			dims: dims,
			tags: tags, 
			dst_slc_range: dst_slc_range,
			protolayer: protolayer,
		}
	}

	pub fn area_name(&self) -> &'static str {
		self.area_name
	}

	pub fn dims(&self) -> &CorticalDims {
		&self.dims
	}

	pub fn axn_count(&self) -> u32 {
		self.dims().cells()
	}

	pub fn tags(&self) -> LayerTags {
		self.tags
	}

	pub fn dst_slc_range(&self) -> &Range<u8> {
		&self.dst_slc_range
	}
}
