// use std::collections::{ HashMap };
// use std::ops::{ Range };
use std::slice::{ Iter };

use proto::{ ProtoareaMap, ProtoareaMaps, ProtolayerMap, ProtolayerMaps, RegionKind };
use cmn::{ self };
use map::{ LayerTags, LayerInfo, SourceLayerInfo};
use input_source::{ InputSource };


#[derive(Clone)]
// [FIXME]: TODO: Add caches.
pub struct LayerMap {
	area_name: &'static str,
	index: Vec<LayerInfo>,
	// TEMP: Remove or make private.
	depth: u8,
	pub plmap: ProtolayerMap,
}

impl LayerMap {
	pub fn new(pamap: &ProtoareaMap, plmaps: &ProtolayerMaps, pamaps: &ProtoareaMaps, 
			input_sources: &Vec<InputSource>) -> LayerMap 
	{
		println!("\nLAYERMAP::NEW(): Assembling layer map for area '{}'...{mt}", 
			pamap.name, mt = cmn::MT);

		let mut plmap = plmaps[pamap.layer_map_name].clone();
		plmap.freeze(&pamap);

		let mut index = Vec::with_capacity(plmap.layers().len());
		let mut slc_total = 0u8;

		for (pl_name, pl) in plmap.layers().iter() {
			index.push(LayerInfo::new(pl, pamap, pamaps, plmaps, input_sources, &mut slc_total));
		}

		assert_eq!(slc_total as usize, plmap.slc_map().len());

		// println!("{mt}{mt}LAYERMAP::NEW(): index: {:?}, plmap.slc_map(): {:?}", 
		// 	index, plmap.slc_map(), mt = cmn::MT);
		LayerMap { area_name: pamap.name, index: index, plmap: plmap, depth: slc_total }
	}

	// [FIXME] TODO: Cache results.
	pub fn layer_info(&self, tags: LayerTags) -> Vec<&LayerInfo> {
		self.index.iter().filter(|li| li.tags().meshes(tags)).map(|li| li).collect()
	}

	// [FIXME] TODO: Create HashMap to index layer names.
	pub fn layer_info_by_name(&self, name: &'static str) -> &LayerInfo {
		let layers: Vec<&LayerInfo> = self.index.iter().filter(|li| li.name() == name)
			.map(|li| li).collect();
		debug_assert_eq!(layers.len(), 1);
		layers[0]
	}

	// [FIXME] TODO: Cache results. Use iterator mapping and filtering.
	pub fn layer_src_info(&self, tags: LayerTags) -> Vec<&SourceLayerInfo> {
		let mut src_layers = Vec::with_capacity(8);

		for layer in self.layer_info(tags).iter() {
			for src_layer in layer.sources().iter() {
				debug_assert!(src_layer.tags().meshes(tags.mirror_io()));
				src_layers.push(src_layer);
			}
		}

		src_layers
	}

	pub fn layer_src_area_names_by_tags(&self, tags: LayerTags) -> Vec<&'static str> {
		self.layer_src_info(tags).iter().map(|sli| sli.area_name()).collect()
	}

	pub fn slc_src_layer_info(&self, slc_id: u8, layer_tags: LayerTags) -> Option<&SourceLayerInfo> {
		let mut src_layer_info = Vec::with_capacity(8);
		let layer_info = self.layer_info(layer_tags);

		for lyr in layer_info {			
			for src_lyr in lyr.sources() {
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

	pub fn iter(&self) -> Iter<LayerInfo> {
		self.index.iter()
	}

	pub fn region_kind(&self) -> &RegionKind {
		&self.plmap.kind
	}

	pub fn depth(&self) -> u8 {
		self.depth
	}
}

