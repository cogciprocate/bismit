use std::ops::{ Range };
// use std::slice::{ Iter };

use proto::{ Protolayer, ProtoareaMap, ProtoareaMaps, ProtolayerMaps, LayerKind, DendriteKind, 
	LayerMapKind, AxonKind };
use cmn::{ self, CorticalDims };
use map::{ self, LayerTags, };
use input_source::{ InputSources };

// const CELLULAR_AXON_KIND: AxonKind = AxonKind::Spatial;

// [FIXME]: Consolidate terminology and usage between source-layer layers (cellular)
// and source-area layers (axonal).
#[derive(Clone, Debug)]
pub struct LayerInfo {
	name: &'static str,	
	tags: LayerTags,
	slc_range: Range<u8>,
	sources: Vec<SourceLayerInfo>,
	axn_count: u32,
	axn_kind: AxonKind,
	protolayer: Protolayer,
}

impl LayerInfo {
	// [FIXME]: TODO: Clean up and optimize.
	// [FIXME]: TODO: Return result and get rid of panics, et al.
	pub fn new(protolayer: &Protolayer, pamap: &ProtoareaMap, pamaps: &ProtoareaMaps, 
				plmaps: &ProtolayerMaps, input_sources: &InputSources, slc_total: &mut u8) -> LayerInfo 
	{
		let protolayer = protolayer.clone();
		let name = protolayer.name();
		let tags = protolayer.tags();
		let axn_kind = protolayer.axn_kind().expect("LayerInfo::new()");
		// let slc_range = protolayer.slc_idz()..(protolayer.slc_idz() + protolayer.depth());
		let mut sources = Vec::with_capacity(8);

		let mut next_slc_idz = *slc_total;
		let mut axn_count = 0;

		let mut src_layer_debug: Vec<String> = Vec::new();

		// println!("\n{mt}{mt}### LAYER: {:?}, next_slc_idz: {}, slc_range: {:?}\n", 
		// 	tags, next_slc_idz, slc_range, mt = cmn::MT);

		// If layer is an input layer, add sources:
		if tags.contains(map::INPUT) {
			match protolayer.kind() {
				&LayerKind::Axonal(_) => (),
				_ => panic!("Error assembling LayerInfo for '{}'. Layers containing \
					'map::INPUT' must be 'AxonKind::Axonal'.", name),
			}

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
			// Our layer must contain the flow direction flag corresponding with the source area.
			for (src_area_name, src_area_tags) in src_area_combos.into_iter()
					.filter(|&(_, sat)|  tags.contains(sat))
			{				
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

					////////////
					//
					// NOTE: Finish finding input_source depth (scan for matching area name)
					// if input_source with matching area name is not found, use the protolayer depth
					// 
					// ALSO:						
					//
					// - [FIXME] Determine depths for input sources!
					//
					//
					////////////

					// [FIXME] Determine depths for input sources
					// let src_layer_depth = cmn::DEFAULT_OUTPUT_LAYER_DEPTH;
					// let is_area = input_source_with_area(input_sources, src_area_name);
					// let src_layer_depth = 					

					let (src_layer_dims, src_layer_axn_kind) = match src_layer_map.kind() {
						&LayerMapKind::Thalamic => {
							let is = input_sources.get(&(src_area_name, src_layer.tags()))
								.expect(&format!("LayerInfo::new(): Invalid input source key: \
									('{}', '{:?}')", src_area_name, src_layer.tags()));
							(is.dims().clone(), is.axn_kind())
						},

						_ => {
							let depth = src_layer.depth().unwrap_or(cmn::DEFAULT_OUTPUT_LAYER_DEPTH);

							let src_axn_kind = match src_layer.kind() {
								&LayerKind::Axonal(ref ak) => {
									// [FIXME]: Make this a Result:
									assert!(ak.matches_tags(src_layer.tags()), "Incompatable layer \
										tags for layer: {:?}", src_layer);

									ak.clone()
								},

								&LayerKind::Cellular(_) => AxonKind::from_tags(src_layer.tags())
									.expect("LayerInfo::new(): Error determining axon kind"),
								// _ => panic!("LayerInfo::new(): Unknown LayerKind."),
							};

							(src_pamap.dims().clone_with_depth(depth), src_axn_kind)
						},
					};

					let tar_slc_range = next_slc_idz..(next_slc_idz + src_layer_dims.depth());
					src_layer_debug.push(format!("{mt}{mt}{mt}{mt}<{}>[\"{}\"]: {:?} | {:?}", src_layer.name(), 
						src_area_name, tar_slc_range, src_layer.tags(), mt = cmn::MT));

					sources.push(SourceLayerInfo::new(src_area_name, &src_layer_dims, 
						src_layer.tags(), src_layer_axn_kind, next_slc_idz));						

					// println!("{mt}{mt}{mt}{mt}LAYERINFO::NEW(layer: '{}'): Adding source layer: \
					// 	src_area_name: '{}', src_area_tags: '{:?}', src_layer_map.name: '{}', \
					// 	src_layer.name: '{}', next_slc_idz: '{}', depth: '{}', \
					// 	src_layer.tags: '{:?}'", name, src_area_name, src_area_tags, 
					// 	src_layer_map.name, src_layer.name(), next_slc_idz, src_layer.depth(), 
					// 	src_layer.tags(), mt = cmn::MT);

					// For (legacy) comparison purposes:
					// protolayer.set_depth(src_layer_depth);

					next_slc_idz += src_layer_dims.depth();
					axn_count += src_layer_dims.cells();
				}
			} 
		} else {
			// <<<<< DEPTH STUFF >>>>>
			// let output_layer_depth = if tags.contains(map::OUTPUT) { Some(1) } else { None };

			// [FIXME]: Get rid of the map::OUTPUT check and just default to 0.
			let layer_depth = match protolayer.depth() {
				Some(d) => d,
				None => if tags.contains(map::OUTPUT) { cmn::DEFAULT_OUTPUT_LAYER_DEPTH } else { 0 },
			};


			// if protolayer.axn_kind().unwrap() == AxonKind::None {
			// 	assert!(layer_depth == 0);
			// }

			next_slc_idz += layer_depth;
			axn_count += pamap.dims().columns() * layer_depth as u32;
		}

		let slc_range = *slc_total..next_slc_idz;
		*slc_total = next_slc_idz;		
		// assert_eq!(next_slc_idz, slc_range.end);
		sources.shrink_to_fit();

		println!("{mt}{mt}{mt}<{}>: {:?} | {:?}", name, slc_range, tags, mt = cmn::MT);

		for dbg_string in src_layer_debug {
			println!("{}", &dbg_string);
		}

		LayerInfo {
			name: name,
			tags: tags,
			slc_range: slc_range,
			sources: sources,
			axn_count: axn_count,
			axn_kind: axn_kind,
			protolayer: protolayer,
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

	pub fn axn_kind(&self) -> AxonKind {
		self.axn_kind.clone()
	}

	pub fn slc_range(&self) -> &Range<u8> {
		&self.slc_range
	}

	pub fn depth(&self) -> u8 {
		self.slc_range.len() as u8
	}
}

// fn input_source_from_area(input_sources: &Vec<InputSource>, area_name: &'static str) {
// 	let matching_sources = input_sources.iter().filter
// }


#[derive(Clone, Debug)]
pub struct SourceLayerInfo {
	area_name: &'static str,
	dims: CorticalDims,
	tags: LayerTags,
	axn_kind: AxonKind,
	// depth: u8,
	tar_slc_range: Range<u8>,
}

impl SourceLayerInfo {
	pub fn new(src_area_name: &'static str, src_layer_dims: &CorticalDims, src_layer_tags: LayerTags, 
				src_axn_kind: AxonKind, tar_slc_idz: u8) -> SourceLayerInfo 
	{
		// let dims = area_dims.clone_with_depth(depth);
		let tar_slc_range = tar_slc_idz..(tar_slc_idz + src_layer_dims.depth());
		// debug_assert_eq!(src_layer_dims.cells(), axn_count);

		SourceLayerInfo {
			area_name: src_area_name, 
			dims: src_layer_dims.clone(),
			tags: src_layer_tags, 
			axn_kind: src_axn_kind,
			// depth: depth,
			tar_slc_range: tar_slc_range,
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

	pub fn axn_kind(&self) -> AxonKind {
		self.axn_kind.clone()
	}

	pub fn tar_slc_range(&self) -> &Range<u8> {
		&self.tar_slc_range
	}
}