// use num;
// use rand;
// use std::mem;
// use rand::{ ThreadRng };
// use num::{ Integer };
// use std::default::{ Default };
// use std::fmt::{ Display };
// use std::ops::{ Range };

use cmn::{ self, CorticalDims };
use map::{ AreaMap };
use ocl::{ self, ProQueue, WorkSize, Envoy };
use proto::{ /*ProtoLayerMap, RegionKind, ProtoAreaMaps,*/ ProtocellKind, Protocell, DendriteKind };
use synapses::{ Synapses };
use axon_space::{ AxonSpace };
// use cortical_area:: { Aux };

#[cfg(test)]
pub use self::tests::{ DenCoords, DendritesTest, den_idx };

pub struct Dendrites {
	layer_name: &'static str,
	dims: CorticalDims,
	//protocell: Protocell,
	//per_cell_l2: u32,
	den_kind: DendriteKind,
	cell_kind: ProtocellKind,
	kern_cycle: ocl::Kernel,
	pub thresholds: Envoy<ocl::cl_uchar>,
	pub states_raw: Envoy<ocl::cl_uchar>,
	pub states: Envoy<ocl::cl_uchar>,
	pub energies: Envoy<ocl::cl_uchar>,
	syns: Synapses,
}

impl Dendrites {
	pub fn new(
					layer_name: &'static str,
					dims: CorticalDims,
					//src_tfts: Vec<Vec<&'static str>>,
					protocell: Protocell,
					den_kind: DendriteKind, 
					cell_kind: ProtocellKind,
					area_map: &AreaMap,
					axons: &AxonSpace,
					// aux: &Aux,
					ocl: &ProQueue
	) -> Dendrites {
		//println!("\n### Test D1 ###");
		//let width_dens = dims.width << per_cell_l2;
		assert!(dims.per_tft_l2() as u8 == protocell.dens_per_tuft_l2);

		//let dims = cel_dims.clone_with_ptl2(per_cell_l2);

		let syns_per_den_l2 = protocell.syns_per_den_l2;
		let den_threshold = protocell.den_thresh_init.unwrap_or(1);

		/*let (den_threshold, den_kernel) = match den_kind {
			DendriteKind::Distal => (
				protocell.den_thresh_init.unwrap_or(1),
				//cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2, 
				"den_cycle"
			),
			DendriteKind::Proximal => (
				protocell.den_thresh_init.unwrap_or(1),
				//cmn::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2, 
				
			),
		};*/



		let states = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let states_raw = Envoy::<ocl::cl_uchar>::new(dims, cmn::STATE_ZERO, ocl);
		let energies = Envoy::<ocl::cl_uchar>::new(dims, 255, ocl);

		println!("            DENDRITES::NEW(): '{}': dendrites with: dims:{:?}, len:{}", layer_name, dims, states.len());

		let syns_dims = dims.clone_with_ptl2((dims.per_tft_l2() + syns_per_den_l2 as i8));
		let syns = Synapses::new(layer_name, syns_dims, protocell.clone(), den_kind, cell_kind, 
			area_map, axons, /*aux,*/ ocl);


		let kern_cycle = ocl.new_kernel("den_cycle".to_string(), WorkSize::OneDim(states.len()))
			.arg_env(&syns.states)
			.arg_env(&syns.strengths)
			.arg_scl(syns_per_den_l2)
			.arg_scl(den_threshold)
			.arg_env(&energies)
			.arg_env(&states_raw)
			// .arg_env_named("aux_ints_0", None)
			// .arg_env_named("aux_ints_1", None)
			.arg_env(&states)
		;

		/*let kern_cycle = ocl.new_kernel("den_cycle_old", WorkSize::TwoDims(dims.depth() as usize, dims.per_slc() as usize))
			.arg_env(&syns.states)
			.arg_env(&syns.strengths)
			.arg_scl(syns_per_den_l2)
			.arg_scl(den_threshold)
			.arg_env(&energies)
			.arg_env(&states_raw)
			//.arg_env(&aux.ints_0)
			.arg_env(&states)
		;*/
		
		Dendrites {
			layer_name: layer_name,
			dims: dims,
			//protocell: protocell,
			//per_cell_l2: per_cell_l2,
			den_kind: den_kind,
			cell_kind: cell_kind,
			kern_cycle: kern_cycle,
			thresholds: Envoy::<ocl::cl_uchar>::new(dims, 1, ocl),
			states_raw: states_raw,
			states: states,
			energies: energies,
			syns: syns,
		}
	}


	pub fn cycle(&self) {
		self.syns.cycle();

		self.kern_cycle.enqueue();
	}

	// FOR TESTING PURPOSES
	pub fn cycle_self_only(&self) {
		self.kern_cycle.enqueue();
	}

	pub fn regrow(&mut self) {
		self.syns.regrow();
	}

	pub fn confab(&mut self) {
		self.thresholds.read();
		self.states_raw.read();
		self.states.read();
		self.syns.confab();
	}

	pub fn dims(&self) -> &CorticalDims {
		&self.dims
	}

	pub fn syns(&self) -> &Synapses {
		&self.syns
	}

	pub fn syns_mut(&mut self) -> &mut Synapses {
		&mut self.syns
	}

}



#[cfg(test)]
pub mod tests {
	#![allow(non_snake_case)]
	use std::ops::{ Range };
	use std::fmt::{ Display, Formatter, Result };
	use rand::distributions::{ IndependentSample, Range as RandRange };

	use super::{ Dendrites };
	use cmn::{ CelCoords };
	use cmn::{ CorticalDims };
	use synapses::{ SynapsesTest };

	pub trait DendritesTest {
		fn set_all_to_zero(&mut self, set_syns_zero: bool);
		fn den_state_direct(&self, idx: u32) -> u8;
		fn rand_den_coords(&mut self, cel_coords: &CelCoords) -> DenCoords;
		fn den_idx(&self, tft_id: u32, cel_idx: u32, den_id_tft: u32) -> u32;
		fn tft_id_range(&self) -> Range<u32>;
		fn den_id_range(&self) -> Range<u32>;
	}

	impl DendritesTest for Dendrites {
		fn set_all_to_zero(&mut self, set_syns_zero: bool) {
			self.thresholds.set_all_to(0);
			self.states_raw.set_all_to(0);
			self.states.set_all_to(0);
			self.energies.set_all_to(0);

			if set_syns_zero { self.syns.set_all_to_zero() };
		}

		fn den_state_direct(&self, idx: u32) -> u8 {
			let mut sdr = vec![0u8];
			self.states.read_direct(&mut sdr[..], idx as usize);
			sdr[0]
		}

		fn rand_den_coords(&mut self, cel_coords: &CelCoords) -> DenCoords {
			let tft_id_range = RandRange::new(0, self.dims.tfts_per_cel());
			let den_id_cel_range = RandRange::new(0, self.dims.per_tft());

			let tft_id = tft_id_range.ind_sample(self.syns.rng()); 
			let den_id_cel = den_id_cel_range.ind_sample(self.syns.rng());

			DenCoords::new(tft_id, den_id_cel, cel_coords, &self.dims)
		}

		fn den_idx(&self, tft_id: u32, cel_idx: u32, den_id_tft: u32) -> u32 {
			den_idx(&self.dims, tft_id, cel_idx, den_id_tft)
		}

		fn tft_id_range(&self) -> Range<u32> {
			0..self.dims.tfts_per_cel()
		}

		fn den_id_range(&self) -> Range<u32> {
			0..self.dims.per_tft()
		}
		
	}

	// <<<<< TODO: NEEDS UPDATING TO MATCH / INTEGRATE WITH SYN_COORDS >>>>>
	#[derive(Debug, Clone)]
	pub struct DenCoords {
		pub idx: u32,	
		pub tft_id: u32,
		pub den_id_tft: u32,	
		pub cel_coords: CelCoords,
		pub layer_dims: CorticalDims, // Potentially move / remove dims
	}

	impl DenCoords {
		pub fn new(tft_id: u32, den_id_tft: u32, cel_coords: &CelCoords, 
					layer_dims: &CorticalDims
			) -> DenCoords 
		{
			let den_idx = den_idx(&layer_dims, tft_id, cel_coords.idx, den_id_tft);

			DenCoords { 
				idx: den_idx, 
				tft_id: tft_id,
				den_id_tft: den_id_tft, 				
				cel_coords: cel_coords.clone(),
				layer_dims: layer_dims.clone(),
			}
		}

		pub fn cel_den_range_tftsec(&self) -> Range<usize> {
			let den_idz_cel_tft = den_idx(&self.layer_dims, self.tft_id, 
				self.cel_coords.idx, 0) as usize;
			let dens_per_cel_tft = self.layer_dims.per_tft() as usize;			

			den_idz_cel_tft..(den_idz_cel_tft + dens_per_cel_tft)
		}

		pub fn syn_idx_range_tft(&self, syns_per_den_l2: u8) -> Range<usize> {			
			let syn_idz_cel_tft = (den_idx(&self.layer_dims, self.tft_id, 
				self.cel_coords.idx, 0) as usize) << syns_per_den_l2 as usize;
			let syns_per_cel_tft = (self.layer_dims.per_tft() as usize) << syns_per_den_l2 as usize;

			syn_idz_cel_tft..(syn_idz_cel_tft + syns_per_cel_tft)
		}

		pub fn syn_range(&self, syns_per_den_l2: u8) -> Range<usize> {
			let syn_idz_den = (self.idx << syns_per_den_l2) as usize;
			let syns_per_den = (1 << syns_per_den_l2) as usize;

			syn_idz_den..(syn_idz_den + syns_per_den)
		}

		pub fn dims(&self) -> &CorticalDims {
			&self.layer_dims
		}
	}

	impl Display for DenCoords {
	    fn fmt(&self, fmtr: &mut Formatter) -> Result {
	        write!(fmtr, "DenCoords {{ idx: {}, tft_id: {}, den_id_tft: {} }}", 
				self.idx, self.tft_id, self.den_id_tft)
	    }
	}



	// den_idx(): FOR TESTING/DEBUGGING AND A LITTLE DOCUMENTATION
	// 		- Synapse index space heirarchy:  | Tuft - Slice - Cell - Dendrite - Synapse |
	// 		- 'cel_idx' already has slice built in to its value
	pub fn den_idx(layer_dims: &CorticalDims, tft_id: u32, cel_idx: u32, den_id_tft: u32) -> u32 {
		//  NOTE: 'layer_dims' expresses dimensions from the perspective of the 
		//  | Slice - Cell - Tuft - Dendrite - Synapse | heirarchy which is why the function
		//  names seem confusing (see explanation at top of synapses.rs).

		let tft_count = layer_dims.tfts_per_cel();
		let slcs_per_tft = layer_dims.depth();
		let cels_per_slc = layer_dims.columns();
		let dens_per_cel_tft = layer_dims.per_tft();

		// assert!((tft_count * slcs_per_tft as u32 * cels_per_slc * dens_per_cel_tft) == layer_dims.padded_envoy_len());
		assert!(tft_id < tft_count);
		assert!(cel_idx < slcs_per_tft as u32 * cels_per_slc);
		assert!(den_id_tft < dens_per_cel_tft);

		let dens_per_tft = slcs_per_tft as u32 * cels_per_slc * dens_per_cel_tft;

		let den_idz_tft = tft_id * dens_per_tft;
		// 'cel_idx' includes slc_id, v_id, and u_id
		let den_idz_slc_cel_tft = cel_idx * dens_per_cel_tft;
		let den_idx = den_idz_tft + den_idz_slc_cel_tft + den_id_tft;

		den_idx
	}
}
