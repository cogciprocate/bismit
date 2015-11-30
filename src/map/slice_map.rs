use ocl::{ self, EnvoyDims };
use cmn::{ self, CorticalDims, HexTilePlane };
use map::{ area_map, SliceDims, LayerMap, AxonKind };


#[derive(Debug, Clone)]
pub struct SliceMap {
	axn_idzs: Vec<u32>,
	layer_names: Vec<&'static str>,
	axn_kinds: Vec<AxonKind>,
	v_sizes: Vec<u32>,
	u_sizes: Vec<u32>,
	v_scales: Vec<u32>,
	u_scales: Vec<u32>,
	v_mids: Vec<u32>,
	u_mids: Vec<u32>,
	dims: Vec<SliceDims>,
	physical_len: u32
}

impl SliceMap {
	pub fn new(area_dims: &CorticalDims, layers: &LayerMap) -> SliceMap {		
		let slc_map = layers.slc_map();
		let depth = layers.depth() as usize;

		debug_assert_eq!(slc_map.len(), depth);

		let mut axn_idzs = Vec::with_capacity(depth);
		let mut layer_names = Vec::with_capacity(depth);
		let mut axn_kinds = Vec::with_capacity(depth);
		let mut v_scales = Vec::with_capacity(depth);
		let mut u_scales = Vec::with_capacity(depth);
		let mut v_sizes = Vec::with_capacity(depth);
		let mut u_sizes = Vec::with_capacity(depth);
		let mut v_mids = Vec::with_capacity(depth);
		let mut u_mids = Vec::with_capacity(depth);
		let mut dims = Vec::with_capacity(depth);

		let mut axn_idz_ttl = 0u32;
		

		for (&slc_id, &layer) in slc_map.iter() {
			let mut add_slice = |slc_dims: SliceDims| {
				debug_assert_eq!(slc_id as usize, axn_idzs.len());

				axn_idzs.push(axn_idz_ttl);
				axn_idz_ttl += slc_dims.columns();

				layer_names.push(layer.name());
				axn_kinds.push(layer.axn_kind());
				v_sizes.push(slc_dims.v_size());
				u_sizes.push(slc_dims.u_size());
				v_scales.push(slc_dims.v_scale());
				u_scales.push(slc_dims.u_scale());			
				v_mids.push(slc_dims.v_mid());
				u_mids.push(slc_dims.u_mid());	
				dims.push(slc_dims);
			};

			// let src_layer_info = layers.slc_src_layer_info(slc_id, layer.tags());

			// match src_layer_info {
			// 	Some(sli) => {
			// 		add_slice(SliceDims::new(area_dims, Some(sli.dims()), sli.axn_kind())
			// 			.expect("SliceMap::new(): Error creating SliceDims."));
			// 	},

			// 	None =>	add_slice(SliceDims::new(area_dims, None, layer.axn_kind())
			// 		.expect("SliceMap::new()")), // 100% scaling
			// };

			let src_layers = layer.sources();

			if src_layers.len() > 0 {
				for sl in src_layers {
					debug_assert_eq!(layer.axn_kind(), sl.axn_kind());

					add_slice(SliceDims::new(area_dims, Some(sl.dims()), sl.axn_kind())
						.expect("SliceMap::new(): Error creating SliceDims."));
				}
			} else {
				add_slice(SliceDims::new(area_dims, None, layer.axn_kind())
					.expect("SliceMap::new()"))
			}
		}

		debug_assert_eq!(axn_idzs.len(), layer_names.len());
		debug_assert_eq!(axn_idzs.len(), axn_kinds.len());
		debug_assert_eq!(axn_idzs.len(), dims.len());
		debug_assert_eq!(axn_idzs.len(), v_sizes.len());
		debug_assert_eq!(axn_idzs.len(), u_sizes.len());
		debug_assert_eq!(axn_idzs.len(), v_scales.len());
		debug_assert_eq!(axn_idzs.len(), u_scales.len());
		debug_assert_eq!(axn_idzs.len(), v_mids.len());
		debug_assert_eq!(axn_idzs.len(), u_mids.len());
		debug_assert_eq!(axn_idzs.len(), depth);		

		SliceMap {
			axn_idzs: axn_idzs,
			layer_names: layer_names,
			axn_kinds: axn_kinds,
			dims: dims,
			v_sizes: v_sizes,
			u_sizes: u_sizes,	
			v_scales: v_scales,
			u_scales: u_scales,	
			v_mids: v_mids,
			u_mids: u_mids,	
			physical_len: axn_idz_ttl,		
		}
	}

	pub fn print_debug(&self) {
		println!(
			"{mt}{mt}SLICEMAP::PRINT_DEBUG(): Area slices: \
			\n{mt}{mt}{mt}layer_names:  {:?}, \
			\n{mt}{mt}{mt}axn_idzs:     [{}], \
			\n{mt}{mt}{mt}v_sizes:      [{}], \
			\n{mt}{mt}{mt}u_sizes:      [{}], \
			\n{mt}{mt}{mt}v_scales:     [{}], \
			\n{mt}{mt}{mt}u_scales:     [{}], \
			\n{mt}{mt}{mt}v_mids:       [{}], \
			\n{mt}{mt}{mt}u_mids:       [{}]", 
			self.layer_names, 
			area_map::literal_list(&self.axn_idzs), 
			area_map::literal_list(&self.v_sizes), 
			area_map::literal_list(&self.u_sizes), 
			area_map::literal_list(&self.v_scales), 
			area_map::literal_list(&self.u_scales), 
			area_map::literal_list(&self.v_mids), 
			area_map::literal_list(&self.u_mids), 
			mt = cmn::MT
		);

		println!("");
	}

	pub fn idz(&self, slc_id: u8) -> u32 {
		self.axn_idzs[slc_id as usize]
	}

	pub fn layer_name(&self, slc_id: u8) -> &'static str {
		self.layer_names[slc_id as usize]
	}

	pub fn slc_axn_count(&self, slc_id: u8) -> u32 {
		self.v_sizes[slc_id as usize] * self.u_sizes[slc_id as usize]
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

	pub fn axn_kinds(&self) -> &Vec<AxonKind> {
		&self.axn_kinds
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

	pub fn v_mids(&self) -> &Vec<u32> {
		&self.v_mids
	}

	pub fn u_mids(&self) -> &Vec<u32> {
		&self.u_mids
	}

	pub fn dims(&self) -> &Vec<SliceDims> {
		&self.dims
	}
}

impl EnvoyDims for SliceMap {
	fn padded_envoy_len(&self, incr: usize) -> usize {
		ocl::padded_len(self.axn_count() as usize, incr)
	}
}


#[cfg(test)]
pub mod tests {
	use std::fmt::{ Display, Formatter, Result as FmtResult };
	use super::{ SliceMap };

	pub trait SliceMapTest {
		fn print(&self);
	}

	impl SliceMapTest for SliceMap {
		fn print(&self) {
			unimplemented!();
		}
	}

	impl Display for SliceMap {
		fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
			let mut output = String::with_capacity(30 * self.slc_count());

			for i in 0..self.slc_count() {
				output.push_str(&format!("[{}: '{}', {}]", i, self.layer_names()[i], 
					self.axn_idzs()[i]));
			}

			fmtr.write_str(&output)
		}
	}
}



			// println!("{mt}{mt}SLICEMAP::NEW(): Adding inter-area slice '{}': slc_id: {}, src_area_name: {}, \
			// 	v_size: {}, u_size: {}.", layer.name(), slc_id, sli.area_name(),
			// 	slc_dims.v_size(), slc_dims.u_size(), mt = cmn::MT);
