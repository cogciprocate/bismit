use common;
use ocl::{ self, Ocl, WorkSize };
use ocl::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionKind };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cells::{ Aux };
use peak_column::{ PeakColumn };
use columns::{ Columns };


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
	depth_noncellular: u8,
	depth_cellular: u8,
	pub width: u32,
	padding: u32,
	//kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_uchar>,
}

impl Axons {
	pub fn new(width: u32, depth_noncellular: u8, depth_cellular: u8, region: &CorticalRegion, ocl: &Ocl) -> Axons {
		let padding: u32 = (common::AXONS_MARGIN * 2) as u32;
		//let padding: u32 = std::num::cast(common::AXONS_MARGIN * 2).expect("Axons::new()");
		let depth = depth_cellular + depth_noncellular;

		//println!("New Axons with: depth_ac: {}, depth_c: {}, width: {}", depth_noncellular, depth_cellular, width);
		let states = Envoy::<ocl::cl_uchar>::with_padding(padding, width, depth, common::STATE_ZERO, ocl);

		Axons {
			depth_noncellular: depth_noncellular,
			depth_cellular: depth_cellular,
			width: width,
			padding: padding,
			//kern_cycle: kern_cycle,
			states: states,
		}
	}


}


	/*pub fn cycle(&self, soma: &Somata, ocl: &Ocl) {
		let mut kern = ocl.new_kernel("axns_cycle", WorkSize::TwoDim(self.depth_cellular as usize, self.width as usize));

		kern.arg(&soma.states);
		kern.arg(&soma.hcol_max_ids);
		kern.arg(&self.states);
		kern.arg_scalar(self.depth_noncellular as u32);

		kern.enqueue();
	} */


	/*pub fn cycle_orig(&self, soma: &Somata, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "axns_cycle");

		ocl::set_kernel_arg(0, soma.states.buf, kern);
		//ocl::set_kernel_arg(1, soma.hcol_max_vals.buf, kern);
		ocl::set_kernel_arg(1, soma.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);
		ocl::set_kernel_arg(3, self.depth_noncellular as u32, kern);

		let gws = (self.depth_cellular as usize, self.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	} */
