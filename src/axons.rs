use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ ProtoAreas };
use proto::regions::{ ProtoRegion, ProtoRegionKind };
use proto::cell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cortical_area:: { Aux };
use peak_column::{ PeakColumns };
use minicolumns::{ MiniColumns };


use std;
use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };

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
	pub fn new(area_dims: CorticalDimensions, region: &ProtoRegion, ocl: &Ocl) -> Axons {
		let depth_axn_sptl = region.depth_axonal_spatial();
		let depth_cellular = region.depth_cellular();
		let depth_axn_hrz = region.depth_axonal_horizontal();
		let depth_total = region.depth_total();
		
		let mut hrz_axn_slices = 0u8;


		if depth_axn_hrz > 0 {
			let syn_span_lin_l2 = (cmn::SYNAPSE_REACH_GEO_LOG2 + 1) << 1;
			let hrz_frames_per_slice: u8 = (area_dims.columns() >> (syn_span_lin_l2)) as u8; // dims.width / (aspiny_span * 2)

			assert!(hrz_frames_per_slice > 0, 
				"Synapse span must be equal or less than cortical area width");

			hrz_axn_slices += depth_axn_hrz / hrz_frames_per_slice;

			if (depth_axn_hrz % hrz_frames_per_slice) != 0 {
				hrz_axn_slices += 1;
			}

			/*println!("\nAxons::new(): width: {}, syn_span: {}, depth_axn_hrz: {}, hrz_frames_per_slice: {}, hrz_axon_slices: {}", 
				width, 1 << syn_span_lin_l2, depth_axn_hrz, hrz_frames_per_slice, hrz_axn_slices);*/
		}

		println!("##### Axon depth_total: {}", depth_total);

		let dims = area_dims.clone_with_depth(depth_total);

		println!("##### Axon dims: {:?}", dims);

		let padding: u32 = (cmn::SYNAPSE_SPAN_LIN) as u32;
		
		//println!("####### padding: {}", padding);
		//println!("New Axons with: depth_ac: {}, depth_c: {}, width: {}", depth_axn_sptl, depth_cellular, width);
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


	/*pub fn cycle(&self, soma: &Somata, ocl: &Ocl) {
		let mut kern = ocl.new_kernel("axns_cycle", WorkSize::TwoDim(self.depth_cellular as usize, self.dims.width as usize));

		kern.arg(&soma.states);
		kern.arg(&soma.hcol_max_ids);
		kern.arg(&self.states);
		kern.arg_scalar(self.depth_axn_sptl as u32);

		kern.enqueue();
	} */


	/*pub fn cycle_orig(&self, soma: &Somata, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "axns_cycle");

		ocl::set_kernel_arg(0, soma.states.buf, kern);
		//ocl::set_kernel_arg(1, soma.hcol_max_vals.buf, kern);
		ocl::set_kernel_arg(1, soma.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);
		ocl::set_kernel_arg(3, self.depth_axn_sptl as u32, kern);

		let gws = (self.depth_cellular as usize, self.dims.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	} */
