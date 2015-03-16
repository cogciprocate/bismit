use common;
use ocl::{ self, Ocl, WorkSize };
use envoy::{ Envoy };
use cortical_areas::{ CorticalAreas, Width };
use cortical_regions::{ CorticalRegion, CorticalRegionType };
use protocell::{ CellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use dendrites::{ Dendrites };
use cells::{ Somata, Aux };
use aspiny::{ AspinyStellate };
use columns::{ Columns };


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
	kern_cycle: ocl::Kernel,
	pub states: Envoy<ocl::cl_char>,
}

impl Axons {
	pub fn new(width: u32, height_noncellular: u8, height_cellular: u8, region: &CorticalRegion, ocl: &Ocl) -> Axons {
		let padding: u32 = num::cast(common::AXONS_MARGIN * 2).expect("Axons::new()");
		let height = height_cellular + height_noncellular;

		//println!("New Axons with: height_ac: {}, height_c: {}, width: {}", height_noncellular, height_cellular, width);
		let states = Envoy::<ocl::cl_char>::with_padding(padding, width, height, 0i8, ocl);

		let mut kern_cycle = ocl.new_kernel("axns_cycle_unoptd", WorkSize::TwoDim(height_cellular as usize, width as usize))
			.lws(WorkSize::TwoDim(1 as usize, common::AXONS_WORKGROUP_SIZE as usize));

		Axons {
			height_noncellular: height_noncellular,
			height_cellular: height_cellular,
			width: width,
			padding: padding,
			kern_cycle: kern_cycle,
			states: states,
		}
	}

	pub fn init_kernels(&mut self, asps: &AspinyStellate, cols: &Columns, aux: &Aux) {
		
			self.kern_cycle.new_arg_envoy(&asps.ids);
			self.kern_cycle.new_arg_envoy(&asps.states);
			self.kern_cycle.new_arg_envoy(&cols.states);
			self.kern_cycle.new_arg_envoy(&self.states);
			self.kern_cycle.new_arg_envoy(&aux.ints_0);
			self.kern_cycle.new_arg_envoy(&aux.ints_1);
			//self.kern_cycle.arg_local(0u8, common::AXONS_WORKGROUP_SIZE / common::ASPINY_SPAN as usize);
			self.kern_cycle.new_arg_scalar(self.height_noncellular as u32);
	}

	pub fn cycle(&self) {

		
		self.kern_cycle.enqueue();
	} 

}





	/*pub fn cycle(&self, soma: &Somata, ocl: &Ocl) {
		let mut kern = ocl.new_kernel("axns_cycle", WorkSize::TwoDim(self.height_cellular as usize, self.width as usize));

		kern.arg(&soma.states);
		kern.arg(&soma.hcol_max_ids);
		kern.arg(&self.states);
		kern.arg_scalar(self.height_noncellular as u32);

		kern.enqueue();
	} */


	/*pub fn cycle_orig(&self, soma: &Somata, ocl: &Ocl) {

		let kern = ocl::new_kernel(ocl.program, "axns_cycle");

		ocl::set_kernel_arg(0, soma.states.buf, kern);
		//ocl::set_kernel_arg(1, soma.hcol_max_vals.buf, kern);
		ocl::set_kernel_arg(1, soma.hcol_max_ids.buf, kern);
		ocl::set_kernel_arg(2, self.states.buf, kern);
		ocl::set_kernel_arg(3, self.height_noncellular as u32, kern);

		let gws = (self.height_cellular as usize, self.width as usize);

		ocl::enqueue_2d_kernel(ocl.command_queue, kern, None, &gws, None);

	} */
