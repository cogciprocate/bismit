// use std::collections::{ HashMap };
use std::ops::{ Range };
// use std::iter;

use proto::{ Protolayer, ProtoareaMap, ProtoareaMaps, ProtolayerMap, ProtolayerMaps };
use cmn::{ self, CorticalDims };
use map::{ self, LayerFlags, /*SliceDims*/ };


#[derive(Clone)]
pub struct LayerMap {
	area_name: &'static str,
	index: Vec<LayerInfo>,
	// names: HashMap<&'static str, usize>,
	// slices: HashMap<u8, usize>,
	// flags: HashMap<LayerFlags, Vec<usize>>,
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

		// let names = HashMap::with_capacity(index.capacity());
		// let slices = HashMap::with_capacity(index.capacity());
		// let flags = HashMap::with_capacity(index.capacity());

		LayerMap { area_name: pamap.name, index: index, /*names: names, slices: slices, flags: flags*/ }
	}

	// [FIXME] TODO: Cache results.
	pub fn layer_info_by_flag(&self, flags: LayerFlags) -> Vec<&LayerInfo> {
		self.index.iter().filter(|li| li.flags.contains(flags)).map(|li| li).collect()
	}

	// [FIXME] TODO: Cache results. Use iterator mapping and filtering.
	pub fn layer_src_info_by_flag(&self, flags: LayerFlags) -> Vec<&SourceLayerInfo> {
		// let mut src_layers = Vec::with_capacity(8);

		// // let comp_flagset = flags; // (flags & !map::INPUT) | map::OUTPUT;

		// for layer in self.index.iter() {
		// 	// println!("\n*** LAYERMAP::SANBF('{}', '{}'):L1: comp_flagset: '{:?}', layer.flags: '{:?}', Match: {}", 
		// 	// 	self.area_name, layer.name, comp_flagset, layer.flags, layer.flags.contains(comp_flagset));

		// 	if layer.flags.contains(flags) {
		// 		// println!("*** LAYERMAP::SANBF('{}', '{}'):L2: layer.sources: '{:?}'", 
		// 		// 	self.area_name, layer.name, layer.sources);

		// 		for src_layer in layer.sources.iter() {
		// 			// println!("*** LAYERMAP::SANBF('{}', '{}'):L3: comp_flagset: '{:?}', source_layer.flags: '{:?}'", 
		// 			// 	self.area_name, layer.name, flags, source_layer.flags);

		// 			// if src_layer.flags.contains(flags.mirror_io()) {
		// 			// 	// names.push((source_layer.area_name, source_layer.flags));
		// 			// 	src_layers.push(src_layer);
		// 			// 	// println!("*** LAYERMAP::SANBF('{}', '{}'): Pushing: {}, {:?}", 
		// 			// 	// 	self.area_name, layer.name, source_layer.area_name, source_layer.flags);
		// 			// }
		// 			debug_assert!(src_layer.flags.contains(flags.mirror_io()));
		// 			src_layers.push(src_layer);
		// 		}				
		// 	}
		// }

		// src_layers

		let mut src_layers = Vec::with_capacity(8);

		for layer in self.layer_info_by_flag(flags).iter() {
			for src_layer in layer.sources.iter() {
				debug_assert!(src_layer.flags.contains(flags.mirror_io()));
				src_layers.push(src_layer);
			}
		}

		src_layers
	}

	pub fn layer_src_area_names_by_flag(&self, flags: LayerFlags) -> Vec<&'static str> {
		self.layer_src_info_by_flag(flags).iter().map(|sli| sli.area_name()).collect()
	}

	pub fn slc_src_layer_info(&self, slc_id: u8, layer_flags: LayerFlags) -> Option<&SourceLayerInfo> {
		let mut src_layer_info = Vec::with_capacity(8);
		let layer_info = self.layer_info_by_flag(layer_flags);

		for lyr in layer_info {			
			for src_lyr in lyr.src_info() {
				// println!("   ###### SRC LAYER_INFO: {:?}", src_lyr);
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

	// pub fn slc_src_layer_dims(&self, slc_id: u8, layer_flags: LayerFlags) -> Vec<&CorticalDims> {
	// 	let mut src_layer_dims = Vec::with_capacity(8);
	// 	let layer_info = self.layer_info_by_flag(layer_flags);

	// 	for lyr in layer_info {
	// 		if slc_id >= lyr.slc_idz() && slc_id <= lyr.slc_idm() {
	// 			for src_lyr in lyr.src_info() {
	// 				// println!("   ###### SRC LAYER_INFO: {:?}", src_lyr);
	// 				src_layer_dims.push(src_lyr.dims());
	// 			}
	// 		}
	// 	}

	// 	src_layer_dims
	// }
}


#[derive(Clone, Debug)]
pub struct LayerInfo {
	name: &'static str,	
	flags: LayerFlags,
	// slices: Vec<u8>,
	slc_range: Range<u8>,
	// slc_idz: u8,
	// depth: u8,
	sources: Vec<SourceLayerInfo>,
	axn_count: u32,
	proto: Protolayer,
}

impl LayerInfo {
	// [FIXME]: TODO: Clean up and optimize.
	pub fn new(protolayer: &Protolayer, pamap: &ProtoareaMap, pamaps: &ProtoareaMaps, 
				plmaps: &ProtolayerMaps) -> LayerInfo {
		let name = protolayer.name;
		let flags = protolayer.flags;
		let slc_range = protolayer.base_slc_id..(protolayer.base_slc_id + protolayer.depth);
		// let slc_idz = protolayer.base_slc_id;
		// let depth = protolayer.depth;
		let mut sources = Vec::with_capacity(8);

		let mut base_slc_id = protolayer.base_slc_id;
		let mut axn_count = 0;

		// If layer is an input layer, add sources:
		if flags.contains(map::INPUT) {
			// let mut src_areas = Vec::with_capacity(pamap.aff_areas.len() + pamap.eff_areas.len());

			// for area_name in pamap.aff_areas.iter() {
			// 	src_areas.push((area_name, map::FEEDBACK));
			// }

			// for area_name in pamap.eff_areas.iter() {
			// 	src_areas.push((area_name, map::FEEDFORWARD));
			// }

			let src_areas: Vec<(&'static str, LayerFlags)> = pamap.aff_areas.iter()
				.map(|&an| (an, map::FEEDBACK)).chain(pamap.eff_areas.iter()
					.map(|&an| (an, map::FEEDFORWARD))).collect();

			// For each potential source area (aff or eff):
			// - get that area's layers
			// - get the layers with a complimentary flag ('map::OUTPUT' in this case)
			//    - other flags identical
			// - filter out feedback from eff areas and feedforward from aff areas
			// - push what's left to sources
			for (src_area_name, src_area_flow) in src_areas {
				// Our layer must contain the flow direction flag corresponding with the source area.
				if flags.contains(src_area_flow) {
					let src_pamap = pamaps.maps().get(src_area_name).expect("LayerInfo::new()");
					let src_layer_map = &plmaps[src_pamap.layer_map_name];
					let src_layers = src_layer_map.layers_with_flags(flags.mirror_io() | src_area_flow);

					for src_layer in src_layers.iter() {
						let src_layer_axns = src_pamap.dims().columns()	* src_layer.depth() as u32;
						// let slc_dims = SliceDims::new(src_pamap.dims(), Some(src_area_dims))
						// 	.expect("LayerInfo::new(): Error calling 'SliceDims::new()'.");

						sources.push(SourceLayerInfo::new(src_area_name, src_pamap.dims(), 
							src_layer.flags, src_layer_axns, base_slc_id, (*src_layer).clone()));

						base_slc_id += src_layer.depth();
						axn_count += src_layer_axns;

						println!("{mt}{mt}LAYERINFO::NEW(layer: '{}'): Adding source layer: \
							src_area_name: '{}', src_area_flow: '{:?}', src_layer_map.name: '{}', \
							src_layer.name: '{}', src_layer.flags: '{:?}'", name, src_area_name, 
							src_area_flow, src_layer_map.name, src_layer.name, src_layer.flags, 
							mt = cmn::MT);
					}
				}

				// println!("{mt}{mt}{mt}##### src_layers: {:?} :", src_layers, mt = cmn::MT);
			} 

			// println!("{mt}{mt}{mt}###### LayerInfo::new(): area: {}, layer: {}, SOURCE LAYERS: {:?}", 
			// 	pamap.name, name, sources, mt = cmn::MT);
		} else {
			base_slc_id += protolayer.depth();
			axn_count += pamap.dims().columns() * protolayer.depth() as u32;
		}

		assert!(base_slc_id == slc_range.end);

		sources.shrink_to_fit();

		LayerInfo {
			name: name,
			flags: protolayer.flags,
			slc_range: slc_range,
			// slc_idz: slc_idz,
			// depth: depth,
			sources: sources,
			axn_count: axn_count,
			proto: protolayer.clone(),
		}
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

	// pub fn slc_idz(&self) -> u8 {
	// 	self.slc_range.start
	// }

	// pub fn slc_idn(&self) -> u8 {
	// 	// assert!(self.slc_range.len() > 0);
	// 	self.slc_range.end
	// }

	pub fn depth(&self) -> u8 {
		self.slc_range.len() as u8
	}
}


#[derive(Clone, Debug)]
pub struct SourceLayerInfo {
	area_name: &'static str,
	dims: CorticalDims,
	flags: LayerFlags,
	dst_slc_range: Range<u8>,
	// axn_count: u32,
	protolayer: Protolayer,
}

impl SourceLayerInfo {
	pub fn new(area_name: &'static str, area_dims: &CorticalDims, flags: LayerFlags, 
				axn_count: u32, dst_slc_idz: u8, protolayer: Protolayer) -> SourceLayerInfo 
	{
		let dims = area_dims.clone_with_depth(protolayer.depth());
		let dst_slc_range = dst_slc_idz..(dst_slc_idz + protolayer.depth());
		debug_assert_eq!(dims.cells(), axn_count);

		SourceLayerInfo {
			area_name: area_name, 
			dims: dims,
			flags: flags, 
			dst_slc_range: dst_slc_range,
			// axn_count: axn_count, 
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

	pub fn dst_slc_range(&self) -> &Range<u8> {
		&self.dst_slc_range
	}
}







// [FIXME] TODO: DEPRICATE
// #[derive(Clone)]
// pub struct LayerSourceAreas {
// 	index: Vec<&'static str>, 						// <-- store in index
// 	names: HashMap<&'static str, SourceAreaInfo>, 	// <-- reverse with ^
// 	axns_sum: u32,
// }

// impl LayerSourceAreas {
// 	pub fn new(area_dims: &CorticalDims, src_area_names: &Vec<&'static str>, pamaps: &ProtoareaMaps) -> LayerSourceAreas {
// 		let mut names = HashMap::with_capacity(src_area_names.len());

// 		let mut axns_sum = 0;

// 		for &src_area_name in src_area_names.iter() {
// 			let src_area_dims = pamaps.maps()[src_area_name].dims();
// 			axns_sum += src_area_dims.columns();
// 			names.insert(src_area_name, SourceAreaInfo::new(area_dims, src_area_name, src_area_dims));
// 		}

// 		LayerSourceAreas {
// 			names: names,
// 			index: src_area_names.clone(),
// 			axns_sum: axns_sum,
// 		}
// 	}

// 	pub fn len(&self) -> usize {
// 		debug_assert!(self.names.len() == self.index.len());		
// 		self.names.len()
// 	}

// 	pub fn axns_sum(&self) -> u32 {
// 		self.axns_sum
// 	}

// 	fn area_dims(&self, area_name: &'static str) -> &SliceDims {
// 		let area_info = &self.names[area_name];
// 		//(area_info.dims.v_size, area_info.dims.u_size)
// 		&area_info.dims
// 	}

// 	pub fn area_info_by_idx(&self, idx: u8) -> &SourceAreaInfo {
// 		assert!((idx as usize) < self.len());
// 		&self.names[self.index[idx as usize]]
// 	}
// }


// [FIXME] TODO: DEPRICATE
// #[derive(Clone)]
// pub struct SourceAreaInfo {
// 	name: &'static str,
// 	dims: SliceDims,
// }

// impl SourceAreaInfo {
// 	pub fn new(area_dims: &CorticalDims, src_area_name: &'static str, src_area_dims: &CorticalDims
// 		) -> SourceAreaInfo 
// 	{
// 		let slc_dims = SliceDims::new(area_dims, Some(src_area_dims)).expect("SourceAreaInfo::new()");
// 		SourceAreaInfo { name: src_area_name, dims: slc_dims }
// 	}

// 	pub fn dims(&self) -> &SliceDims {
// 		&self.dims
// 	}

// 	pub fn name(&self) -> &'static str {
// 		self.name
// 	}
// }


