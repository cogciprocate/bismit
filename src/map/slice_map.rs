
use ocl::{ self, EnvoyDims };
use proto::{ ProtolayerMap, ProtoareaMap };
use cmn::{ self, CorticalDims, HexTilePlane };
use map::{ area_map, SliceDims, LayerMap };


#[derive(Debug, Clone)]
pub struct SliceMap {
	axn_idzs: Vec<u32>,
	layer_names: Vec<&'static str>,
	v_sizes: Vec<u32>,
	u_sizes: Vec<u32>,
	v_scales: Vec<u32>,
	u_scales: Vec<u32>,
	dims: Vec<SliceDims>,
	physical_len: u32
}

impl SliceMap {
	pub fn new(area_dims: &CorticalDims, pamap: &ProtoareaMap, plmap: &ProtolayerMap, 
				layers: &LayerMap,
	) -> SliceMap {		
		let proto_slc_map = plmap.slc_map();

		let mut axn_idzs = Vec::with_capacity(proto_slc_map.len());
		let mut layer_names = Vec::with_capacity(proto_slc_map.len());
		let mut v_scales = Vec::with_capacity(proto_slc_map.len());
		let mut u_scales = Vec::with_capacity(proto_slc_map.len());
		let mut v_sizes = Vec::with_capacity(proto_slc_map.len());
		let mut u_sizes = Vec::with_capacity(proto_slc_map.len());
		let mut dims = Vec::with_capacity(proto_slc_map.len());

		let mut axn_idz_ttl = 0u32;

		for (&slc_id, &layer_name) in proto_slc_map.iter() {
			let layer = &plmap.layers()[layer_name];
			let src_layer_info = layers.slc_src_layer_info(slc_id, layer.tags());

			let slc_dims = match src_layer_info {
				Some(sli) => {
					let slc_dims = SliceDims::new(area_dims, Some(sli.dims()))
						.expect("SliceMap::new(): Error creating SliceDims.");

					println!("{}SLICEMAP::NEW(): Adding inter-area slice '{}': slc_id: {}, src_area_name: {}, \
						v_size: {}, u_size: {}.", cmn::MT, layer_name, slc_id, sli.area_name(),
						slc_dims.v_size(), slc_dims.u_size());

					slc_dims
				},

				None =>	SliceDims::new(area_dims, None).expect("SliceMap::new()"), // 100% scaling
			};

			axn_idzs.push(axn_idz_ttl);
			axn_idz_ttl += slc_dims.columns();

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
