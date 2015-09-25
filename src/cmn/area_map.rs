use num::{ Num };
use std::fmt::{ Display };
//use std::num::ToString;

use ocl::{ BuildOptions, BuildOption };
use proto::{ ProtolayerMaps, ProtolayerMap, Protoarea };
use cmn::{ self, CorticalDimensions };

// 	AREAMAP { }:
// 		- Move in functionality from the 'execution phase' side of protoarea and protolayer_map.
//		- Leave the 'init phase' stuff to the proto-*s.
pub struct AreaMap {
	protoarea: Protoarea,
	protolayer_map: ProtolayerMap,	

	pub slices: SliceMap,

	hrz_demarc: u8,

	// Create maps for each aspect which have their own types and are queryable 
	// into sub-lists of the same type

	// layers: LayerMap
	// slices: SliceMap
	// etc...

	// other new types: TuftMap/CellMap
}

impl AreaMap {
	pub fn new(protolayer_maps: &ProtolayerMaps, protoarea: &Protoarea) -> AreaMap {
		let protoarea = protoarea.clone();			
		let mut protolayer_map = protolayer_maps[protoarea.region_name].clone();
		protolayer_map.freeze(&protoarea);

		let slices = SliceMap::new(&protolayer_map, protoarea.dims());

		let hrz_demarc = protolayer_map.hrz_demarc();

		AreaMap {
			protoarea: protoarea,
			protolayer_map: protolayer_map,
			slices: slices,
			hrz_demarc: hrz_demarc,
		}
	}

	pub fn protoarea(&self) -> &Protoarea {
		&self.protoarea
	}

	pub fn protolayer_map(&self) -> &ProtolayerMap {
		&self.protolayer_map
	}

	pub fn axn_idz(&self, slc_id: u8) -> u32 {
		self.slices.idz(slc_id)
	}

	pub fn gen_build_options(&self) -> BuildOptions {
		let mut build_options = cmn::base_build_options()
			.add_opt(BuildOption::new("HORIZONTAL_AXON_ROW_DEMARCATION", self.protolayer_map.hrz_demarc() as i32))
			.opt("AXN_SLC_COUNT", self.slices.len() as i32)
			.add_opt(BuildOption::with_str_val("AXN_SLC_IDZS", literal_list(&self.slices.idzs)))
			.add_opt(BuildOption::with_str_val("AXN_SLC_V_SCALES", literal_list(&self.slices.v_scales)))
			.add_opt(BuildOption::with_str_val("AXN_SLC_U_SCALES", literal_list(&self.slices.u_scales)))

			// AXN_SLC_IDZS
			// AXN_SLC_V_SCALES
			// AXN_SLC_U_SCALES
		;

		// CUSTOM KERNELS
		match self.protoarea.filters {
			Some(ref protofilters) => {
				for pf in protofilters.iter() {
					match pf.cl_file_name() {
						Some(ref clfn)  => build_options.add_kern_file(clfn.clone()),
						None => (),
					}
				}
			},
			None => (),
		};

		build_options.add_kern_file(cmn::BUILTIN_FILTERS_CL_FILE_NAME.to_string());
		build_options.add_kern_file(cmn::BUILTIN_CL_FILE_NAME.to_string()); // ** MUST BE ADDED LAST **

		build_options
	}

}


#[derive(Debug, Clone)]
pub struct SliceMap {
	idzs: Vec<u32>,
	layer_names: Vec<&'static str>,
	v_scales: Vec<u8>,
	u_scales: Vec<u8>,	
}

impl SliceMap {
	pub fn new(layer_map: &ProtolayerMap, area_dims: CorticalDimensions) -> SliceMap {
		let slc_map = layer_map.slc_map();

		let mut idzs = Vec::with_capacity(slc_map.len());
		let mut layer_names = Vec::with_capacity(slc_map.len());
		let mut v_scales = Vec::with_capacity(slc_map.len());
		let mut u_scales = Vec::with_capacity(slc_map.len());

		for (&slc_id, &layer_name) in slc_map.iter() {
			idzs.push(axn_idz_2d(slc_id, area_dims.columns(), layer_map.hrz_demarc()));
			layer_names.push(layer_name);
			v_scales.push(16);
			u_scales.push(16);
		}

		SliceMap {
			idzs: idzs,
			layer_names: layer_names,
			v_scales: v_scales,
			u_scales: u_scales,			
		}
	}

	pub fn print_debug(&self) {
		let mini_tab = "   "; // 3 spaces

		println!("\nSLICEMAP::PRINT_DEBUG(): \n{mt}layer_names: {:?}, \n{mt}idzs: {:?}(literal: '{}'), \n{mt}v_scales: {:?}(literal: '{}'), \n{mt}u_scales: {:?}(literal: '{}')",
				self.layer_names, self.idzs, literal_list(&self.idzs), self.v_scales, literal_list(&self.v_scales), self.u_scales, literal_list(&self.u_scales), mt = mini_tab);
		println!("");
		// for i in 0..self.idzs.len() {

		// }
	}

	pub fn idz(&self, slc_id: u8) -> u32 {
		self.idzs[slc_id as usize]
	}

	pub fn len(&self) -> u32 {
		self.idzs.len() as u32
	}
}


fn literal_list<T: Display>(vec: &Vec<T>) -> String {
	let mut literal = String::with_capacity((vec.len() * 5) + 20);

	let mut i = 0u32;	
	for ele in vec.iter() {
		if i != 0 {
			literal.push_str(", ");
		}

		literal.push_str(&ele.to_string());
		i += 1;
	}

	literal
}


/* AXN_IDX_2D(): Host side address resolution - concerned with start idx of a slc
	- OpenCL device side version below [outdated] (for reference) - concerned with individual indexes:

		static inline uint axn_idz_2d(uchar slc_id, uint slc_columns, uint col_id, char col_ofs) {
			uint axn_idx_spt = mad24((uint)slc_id, slc_columns, (uint)(col_id + col_ofs + AXON_MAR__GIN_SIZE));
			int hslc_id = slc_id - HORIZONTAL_AXON_ROW_DEMARCATION;
			int hcol_id = mad24(hslc_id, SYNAPSE_SPAN_RHOMBAL_AREA, col_ofs + AXON_MAR__GIN_SIZE);
			uint axn_idx_hrz = mad24((uint)HORIZONTAL_AXON_ROW_DEMARCATION, slc_columns, (uint)(hcol_id + AXON_MAR__GIN_SIZE));
			return mul24((uint)(hslc_id < 0), axn_idx_spt) + mul24((uint)(hslc_id >= 0), axn_idx_hrz);
		}
}*/
pub fn axn_idz_2d(axn_slc: u8, columns: u32, hrz_demarc: u8) -> u32 {
	let mut axn_idx: u32 = if axn_slc < hrz_demarc {
		(axn_slc as u32 * columns)
	} else {
		(hrz_demarc as u32 * columns) + (cmn::SYNAPSE_SPAN_RHOMBAL_AREA * (axn_slc as u32 - hrz_demarc as u32))
	};

	axn_idx + cmn::AXON_MARGIN_SIZE as u32
}
