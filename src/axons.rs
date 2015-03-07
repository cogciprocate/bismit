use common;
use ocl;
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cells::{ Somata };


use std::num;
use std::rand;
use std::mem;
use std::rand::distributions::{ Normal, IndependentSample, Range };
use std::rand::{ ThreadRng };
use std::num::{ NumCast, Int, FromPrimitive };
use std::default::{ Default };
use std::fmt::{ Display };

pub struct Axons {
	height_noncellular: u8,
	height_cellular: u8,
	pub width: u32,
	padding: u32,
	pub states: Envoy<ocl::cl_char>,
	//inhib_tmp_row: u8,
	//inhib_tmp_2_row: u8,
}

impl Axons {
	pub fn new(width: u32, height_noncellular: u8, height_cellular: u8, region: &CorticalRegion, ocl: &ocl::Ocl) -> Axons {
		let padding: u32 = num::cast(common::AXONS_MARGIN * 2).expect("Axons::new()");
		let height = height_cellular + height_noncellular;

		/* BULLSHIT BELOW */
		//let ref region = region[CorticalRegionType::Sensory];
		//let inhib_tmp_row = region.row_ids(vec!["inhib_tmp"])[0];
		//let inhib_tmp_2_row = region.row_ids(vec!["inhib_tmp_2"])[0];
		/* END BULLSHIT (remember to remove inhib_tmp_row) */

		//println!("New Axons with: height_ac: {}, height_c: {}, width: {}", height_noncellular, height_cellular, width);

		Axons {
			height_noncellular: height_noncellular,
			height_cellular: height_cellular,
			width: width,
			padding: padding,
			states: Envoy::<ocl::cl_char>::with_padding(padding, width, height, 0i8, ocl),
			//inhib_tmp_row: inhib_tmp_row,
			//inhib_tmp_2_row: inhib_tmp_2_row,
		}
	}

	pub fn cycle(&self, soma: &Somata, ocl: &ocl::Ocl) {

			let kern = ocl::new_kernel(ocl.program, "axns_cycle");

			ocl::set_kernel_arg(0, soma.states.buf, kern);
			//ocl::set_kernel_arg(1, soma.hcol_max_vals.buf, kern);
			ocl::set_kernel_arg(1, soma.hcol_max_ids.buf, kern);
			ocl::set_kernel_arg(2, self.states.buf, kern);
			ocl::set_kernel_arg(3, self.height_noncellular as u32, kern);

			let gws = (self.height_cellular as usize, self.width as usize);

			ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	} 
}
