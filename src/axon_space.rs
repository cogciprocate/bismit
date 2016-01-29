//use std;
//use num;
//use rand;
//use std::mem;
//use rand::distributions::{ Normal, IndependentSample, Range };
//use rand::{ ThreadRng };
//use num::{ Integer };
//use std::default::{ Default };
//use std::fmt::{ Display };

use cmn::{ self };
use map::{ AreaMap };
use ocl::{ self, ProQue, Envoy, EnvoyDims };
//use proto::{ ProtolayerMap, LayerMapKind, ProtoareaMaps, CellKind, Protocell, DendriteKind };
//use synapses::{ Synapses };
//use dendrites::{ Dendrites };
//use cortical_area:: { Aux };
//use iinn::{ InhibitoryInterneuronNetwork };
//use minicolumns::{ Minicolumns };

#[cfg(test)]
pub use self::tests::{ AxonSpaceTest, AxnCoords };


pub struct AxonSpace {
	//dims: CorticalDims,
	//depth_axn_sptl: u8,
	//depth_cellular: u8,
	//depth_axn_hrz: u8,
	//padding: u32,
	//kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_uchar>,
}

impl AxonSpace {
	pub fn new(area_map: &AreaMap, ocl_pq: &ProQue) -> AxonSpace {
		//let depth_axn_sptl = region.depth_axonal_spatial();
		//let depth_cellular = region.depth_cellular();
		//let depth_axn_hrz = region.depth_axonal_horizontal();
		//let depth_total = region.depth_total(); // NOT THE TRUE AXON DEPTH
		
		//let mut hrz_axn_slcs = 0u8;		

		// <<<<< REDO THIS TO FIT INTO: MIN(V_SIZE, U_SIZE) * MIN(V_SIZE, U_SIZE)
		// if depth_axn_hrz > 0 { 
		// 	let syn_span_lin_l2 = (cmn::SYNAPSE_REACH_GEO_LOG2 + 1) << 1;
		// 	let hrz_frames_per_slc: u32 = (area_dims.columns() >> syn_span_lin_l2) as u8; 

		// 	assert!(hrz_frames_per_slc > 0, 
		// 		"Synapse span must be equal or less than cortical area width");

		// 	hrz_axn_slcs += depth_axn_hrz as u32 / hrz_frames_per_slc;

		// 	if (depth_axn_hrz % hrz_frames_per_slc) != 0 {
		// 		hrz_axn_slcs += 1;
		// 	}

		// 	//println!("      AXONS::NEW(): columns: {}, syn_span: {}, depth_axn_hrz: {}, hrz_frames_per_slc: {}, hrz_axon_slcs: {}", area_dims.columns(), 1 << syn_span_lin_l2, depth_axn_hrz, hrz_frames_per_slc, hrz_axn_slcs);
		// }

		//hrz_axn_slcs = 1; // TEMPORARY (until above is fixed)
		//let physical_depth = depth_cellular + depth_axn_sptl + hrz_axn_slcs;

		//let dims = area_dims.clone_with_depth(physical_depth);

		//let padding: u32 = cmn::AXON_MARGIN_SIZE * 2;
		
		println!("{mt}{mt}AXONS::NEW(): new axons with: total axons: {}", 
			area_map.slices().padded_envoy_len(ocl_pq.get_max_work_group_size()), mt = cmn::MT);

		let states = Envoy::<ocl::cl_uchar>::with_vec(area_map.slices(), ocl_pq.queue());

		AxonSpace {
			//dims: dims,
			//depth_axn_sptl: depth_axn_sptl,
			//depth_cellular: depth_cellular,
			//depth_axn_hrz: depth_axn_hrz,
			//padding: padding,
			//kern_cycle: kern_cycle,
			states: states,
		}
	}
}



#[cfg(test)]
pub mod tests {
	use super::{ AxonSpace };
	use map::{ AreaMap, AreaMapTest };
	use cmn::{ CelCoords };
	use ocl::{ EnvoyTest };

	pub trait AxonSpaceTest {
		fn axn_state(&self, idx: usize) -> u8;
		fn write_to_axon(&mut self, val: u8, idx: u32);
	}

	impl AxonSpaceTest for AxonSpace {
		fn axn_state(&self, idx: usize) -> u8 {
			// let mut sdr = vec![0u8];
			// self.states.read(&mut sdr, idx);
			// sdr[0]

			self.states.read_idx_direct(idx)
		}

		fn write_to_axon(&mut self, val: u8, idx: u32) {
			let sdr = vec![val];
			self.states.write(&sdr, idx as usize, None, None);
		}
	}

	pub struct AxnCoords {
		idx: u32,
		slc_id: u8, 
		v_id: u32, 
		u_id: u32,
	}

	impl AxnCoords {
		pub fn new(slc_id: u8, v_id: u32, u_id: u32, area_map: &AreaMap
			) -> Result<AxnCoords, &'static str> 
		{
			match area_map.axn_idx(slc_id, v_id, 0, u_id, 0) {
				Ok(idx) => Ok(AxnCoords { idx: idx, slc_id: slc_id, v_id: v_id, u_id: u_id }),
				Err(e) => Err(e),
			}
		}

		pub fn from_cel_coords(cel_base_axn_slc: u8, cel_coords: &CelCoords, area_map: &AreaMap
			) -> Result<AxnCoords, &'static str>
		{
			AxnCoords::new(cel_base_axn_slc, cel_coords.v_id,
				cel_coords.u_id, area_map)
		}

		pub fn idx(&self) -> u32 {
			self.idx
		}
	}	
}
