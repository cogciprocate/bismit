use std::ops::{ Range };
// use std::slice::{ Iter };

use proto::{ Protolayer, ProtoareaMap, ProtoareaMaps, ProtolayerMaps, LayerKind, DendriteKind };
use cmn::{ self, CorticalDims };
use map::{ self, LayerTags, };
use input_source::{ InputSource };

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
				plmaps: &ProtolayerMaps, input_sources: &Vec<InputSource>, slc_total: &mut u8) -> LayerInfo 
	{
		let name = protolayer.name();
		let tags = protolayer.tags();
		// let slc_range = protolayer.base_slc_id()..(protolayer.base_slc_id() + protolayer.depth());
		let mut sources = Vec::with_capacity(8);

		let mut next_base_slc_id = *slc_total;
		let mut axn_count = 0;

		// println!("\n{mt}{mt}### LAYER: {:?}, next_base_slc_id: {}, slc_range: {:?}\n", 
		// 	tags, next_base_slc_id, slc_range, mt = cmn::MT);

		// If layer is an input layer, add sources:
		if tags.contains(map::INPUT) {
			let src_area_combos: Vec<(&'static str, LayerTags)> = 
				pamap.aff_areas().iter().map(|&an| (an, map::FEEDBACK | map::SPECIFIC))
					.chain(pamap.eff_areas().iter().map(|&an| (an, map::FEEDFORWARD | map::SPECIFIC)))
				.chain(pamap.aff_areas().iter().chain(pamap.eff_areas().iter())
					.map(|&an| (an, map::NONSPECIFIC)))
				.collect();				

			// println!("\n{mt}{mt}{mt}### SRC_AREAS: {:?}\n", src_area_combos, mt = cmn::MT);

			// For each potential source area (aff or eff):
			// - get that area's layers
			// - get the layers with a complimentary flag ('map::OUTPUT' in this case)
			//    - other tags identical
			// - filter out feedback from eff areas and feedforward from aff areas
			// - push what's left to sources
			for (src_area_name, src_area_tags) in src_area_combos {
				// Our layer must contain the flow direction flag corresponding with the source area.
				// if src_area_tags.meshes(tags) {
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

						/////////////
						//
						// NOTE: Finish finding input_source depth (scan for matching area name)
						// if input_source with matching area name is not found, use the protolayer depth
						//
						// NEXT TODO: Wire up SliceMap with these new goodies and remove plmaps completely.
						//
						////////////

						// [FIXME] Determine depths for 'input_source's
						let src_layer_depth = cmn::DEFAULT_OUTPUT_LAYER_DEPTH;

						let src_layer_axns = src_pamap.dims().columns()	* src_layer_depth as u32;

						sources.push(SourceLayerInfo::new(src_area_name, src_pamap.dims(), 
							src_layer.tags(), src_layer_depth, src_layer_axns, next_base_slc_id, 
							(*src_layer).clone()));						

						// println!("{mt}{mt}{mt}{mt}LAYERINFO::NEW(layer: '{}'): Adding source layer: \
						// 	src_area_name: '{}', src_area_tags: '{:?}', src_layer_map.name: '{}', \
						// 	src_layer.name: '{}', next_base_slc_id: '{}', depth: '{}', \
						// 	src_layer.tags: '{:?}'", name, src_area_name, src_area_tags, 
						// 	src_layer_map.name, src_layer.name(), next_base_slc_id, src_layer.depth(), 
						// 	src_layer.tags(), mt = cmn::MT);

						next_base_slc_id += src_layer_depth;
						axn_count += src_layer_axns;
					}
				} 
				// else if tags.contains(map::NONSPECIFIC) && tags.
			} 
		} else {
			// <<<<< DEPTH STUFF >>>>>
			// let output_layer_depth = if tags.contains(map::OUTPUT) { Some(1) } else { None };

			// let valid_depth = match self.kind {
			// 	Cellular(ref pc) => {

			// 	}
			// 	_ => depth,
			// }

			// [FIXME]: Get rid of the map::OUTPUT check and just default to 0.
			let layer_depth = match protolayer.depth() {
				Some(d) => d,
				None => if tags.contains(map::OUTPUT) { cmn::DEFAULT_OUTPUT_LAYER_DEPTH } else { 0 },
			};

			next_base_slc_id += layer_depth;
			axn_count += pamap.dims().columns() * layer_depth as u32;
		}

		let slc_range = *slc_total..next_base_slc_id;
		*slc_total = next_base_slc_id;		
		// assert_eq!(next_base_slc_id, slc_range.end);
		sources.shrink_to_fit();

		println!("{mt}Adding Layer <{}>: slc_range: {:?}", name, slc_range, mt = cmn::MT);

		LayerInfo {
			name: name,
			tags: tags,
			slc_range: slc_range,
			sources: sources,
			axn_count: axn_count,
			protolayer: protolayer.clone(),
		}
	}

	pub fn src_lyr_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
		self.protolayer.src_lyr_names(den_type)
	}

	pub fn dst_src_lyrs(&self) -> Vec<Vec<&'static str>> {
		let layers_by_tuft = match self.protolayer.kind() {
			&LayerKind::Cellular(ref protocell) => protocell.den_dst_src_lyrs.clone(),
			_ => None,
		};

		match layers_by_tuft {
			Some(v) => v,
			None => Vec::with_capacity(0),
		}
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn tags(&self) -> LayerTags {
		self.tags
	}

	pub fn kind(&self) -> &LayerKind {
		self.protolayer.kind()
	}

	pub fn sources(&self) -> &Vec<SourceLayerInfo>  {
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
	depth: u8,
	dst_slc_range: Range<u8>,
	protolayer: Protolayer,
}

impl SourceLayerInfo {
	pub fn new(area_name: &'static str, area_dims: &CorticalDims, tags: LayerTags, depth: u8,
				axn_count: u32, dst_slc_idz: u8, protolayer: Protolayer) -> SourceLayerInfo 
	{
		let dims = area_dims.clone_with_depth(depth);
		let dst_slc_range = dst_slc_idz..(dst_slc_idz + depth);
		debug_assert_eq!(dims.cells(), axn_count);

		SourceLayerInfo {
			area_name: area_name, 
			dims: dims,
			tags: tags, 
			depth: depth,
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
