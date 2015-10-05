//use num::{ Num };
//use std::fmt::{ Display };
//use std::ops::{ Range };
//use std::collections::{ HashMap };
//use std::num::ToString;

//use ocl::{ BuildOptions, BuildOption, EnvoyDimensions };
use ocl::{ EnvoyDimensions };
//use proto::{ layer, ProtoLayerMaps, ProtoLayerMap, Protolayer, ProtolayerFlags, ProtoAreaMaps, ProtoAreaMap };
//use cmn::{ self, CorticalDimensions, HexTilePlane };
use cmn::{ CorticalDimensions, HexTilePlane };
//use map::{ InterAreaInfoCache };



#[derive(Clone, Debug)]
pub struct SliceDimensions {
	v_size: u32,
	u_size: u32,
	v_scale: u32,
	u_scale: u32,
}

impl SliceDimensions {
	pub fn new(area_dims: &CorticalDimensions, src_area_dims_opt: Option<&CorticalDimensions>) 
		-> Result<SliceDimensions, String> 
	{
		match src_area_dims_opt {
			Some(src_area_dims) => {
				let src_scales_res = get_src_scales(src_area_dims, area_dims);

				if src_scales_res.is_ok() {
					let (v_scale, u_scale) = src_scales_res.unwrap();

					Ok(SliceDimensions { 
						v_size: src_area_dims.v_size(),
						u_size: src_area_dims.u_size(),
						v_scale: v_scale,
						u_scale: u_scale,
					})
				} else {
					Err(src_scales_res.err().unwrap())
				}
			},

			None => {
				Ok(SliceDimensions { 
					v_size: area_dims.v_size(),
					u_size: area_dims.u_size(),
					v_scale: 16,
					u_scale: 16,
				})
			},
		}	
	}	

	pub fn v_scale(&self) -> u32 {
		self.v_scale
	}

	pub fn u_scale(&self) -> u32 {
		self.u_scale
	}

	pub fn columns(&self) -> u32 {
		self.v_size * self.u_size
	}

	pub fn depth(&self) -> u8 {
		1u8
	}
}

impl HexTilePlane for SliceDimensions {
	fn v_size(&self) -> u32 {
		self.v_size
	}

	fn u_size(&self) -> u32 {
		self.u_size
	}

	fn count(&self) -> u32 {
		self.columns()
	}
}

impl EnvoyDimensions for SliceDimensions {
	/* PHYSICAL_LEN(): ROUND CORTICAL_LEN() UP TO THE NEXT PHYSICAL_INCREMENT */
	fn physical_len(&self) -> u32 {
		self.columns()
	}
}


pub fn get_src_scales(src_area_dims: &CorticalDimensions, tar_area_dims: &CorticalDimensions) 
		-> Result<(u32, u32), String> 
	{
	let v_res = calc_scale(src_area_dims.v_size(), tar_area_dims.v_size());
	let u_res = calc_scale(src_area_dims.u_size(), tar_area_dims.u_size());

	if v_res.is_err() || u_res.is_err() {
		let mut err_str = String::new();

		match &v_res {
			&Err(e) => err_str.push_str(&format!("v_size: {}. ", e)),
			_ => (),
		}

		match &u_res {
			&Err(e) => err_str.push_str(&format!("u_size: {}. ", e)),
			_ => (),
		}

		Err(err_str)
	} else {
		Ok((v_res.unwrap(), u_res.unwrap()))
	}
}

pub fn calc_scale(src_dim: u32, tar_dim: u32) -> Result<u32, &'static str> {
	// let scale_incr = if src_dim >= 16 { src_dim / 16 } 
	// 	else if src_dim > 0 { 1 }
	// 	else { panic!("area_map::calc_scale(): Source dimension cannot be zero.") };

	let src_dim = (src_dim as usize) * 1024;
	let tar_dim = (tar_dim as usize) * 1024;

	let scale_incr = match tar_dim {
		0 => return Err("Target area dimension cannot be zero."),
		1...15 => 1,
		_ => tar_dim / 16,
	};

	return match src_dim / scale_incr {
		0 => return Err("Source area dimension cannot be zero."),
		s @ 1...255 => Ok(s as u32),
		_ => return Err("Source area cannot have a dimension more than 16 times target area dimension."),
	}
}
