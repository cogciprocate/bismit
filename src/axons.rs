use std;
use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };

use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ Protoareas };
use proto::regions::{ Protoregion, ProtoregionKind };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cortical_area:: { Aux };
use iinn::{ InhibitoryInterneuronNetwork };
use minicolumns::{ Minicolumns };


pub struct Axons {
	dims: CorticalDimensions,
	depth_axn_sptl: u8,
	depth_cellular: u8,
	depth_axn_hrz: u8,
	padding: u32,
	//kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_uchar>,
}

impl Axons {
	pub fn new(area_dims: CorticalDimensions, region: &Protoregion, ocl: &Ocl) -> Axons {
		let depth_axn_sptl = region.depth_axonal_spatial();
		let depth_cellular = region.depth_cellular();
		let depth_axn_hrz = region.depth_axonal_horizontal();
		//let depth_total = region.depth_total(); // NOT THE TRUE AXON DEPTH
		
		let mut hrz_axn_slcs = 0u8;

		if depth_axn_hrz > 0 {
			let syn_span_lin_l2 = (cmn::SYNAPSE_REACH_GEO_LOG2 + 1) << 1;
			let hrz_frames_per_slc: u8 = (area_dims.columns() >> syn_span_lin_l2) as u8; 

			assert!(hrz_frames_per_slc > 0, 
				"Synapse span must be equal or less than cortical area width");

			hrz_axn_slcs += depth_axn_hrz / hrz_frames_per_slc;

			if (depth_axn_hrz % hrz_frames_per_slc) != 0 {
				hrz_axn_slcs += 1;
			}

			//print!("\n      AXONS::NEW(): columns: {}, syn_span: {}, depth_axn_hrz: {}, hrz_frames_per_slc: {}, hrz_axon_slcs: {}", area_dims.columns(), 1 << syn_span_lin_l2, depth_axn_hrz, hrz_frames_per_slc, hrz_axn_slcs);
		}

		let physical_depth = depth_cellular + depth_axn_sptl + hrz_axn_slcs;

		let dims = area_dims.clone_with_depth(physical_depth);

		let padding: u32 = (cmn::SYNAPSE_SPAN_LIN) as u32;
		
		print!("\n      AXONS::NEW(): new axons with: depth_axn_s: {}, depth_cel: {}, depth_axn_h: {}, physical_depth: {}, dims: {:?}", depth_axn_sptl, depth_cellular, depth_axn_hrz, physical_depth, dims);

		let states = Envoy::<ocl::cl_uchar>::with_padding(padding, dims, cmn::STATE_ZERO, ocl);

		Axons {
			dims: dims,
			depth_axn_sptl: depth_axn_sptl,
			depth_cellular: depth_cellular,
			depth_axn_hrz: depth_axn_hrz,
			padding: padding,
			//kern_cycle: kern_cycle,
			states: states,
		}
	}
}
