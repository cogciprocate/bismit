// use num;
use rand::{ self, XorShiftRng };
// use std::mem;
use rand::distributions::{ /*Normal,*/ IndependentSample, Range };
// use rand::{ ThreadRng, Rng };
// use num::{ Integer };
// use std::default::{ Default };
// use std::fmt::{ Display };
use std::collections::{ BTreeSet };

use cmn::{ self, CorticalDimensions };
use map::{ AreaMap };
use ocl::{ self, ProQueue, WorkSize, Envoy, OclNum };
use proto::{ /*ProtoLayerMap, RegionKind, ProtoAreaMaps,*/ ProtocellKind, Protocell, 
	DendriteKind, /*Protolayer, ProtolayerKind*/ };
// use dendrites::{ Dendrites };
use axon_space::{ AxonSpace };
// use cortical_area:: { Aux };

#[cfg(test)]
pub use self::tests::{ SynCoords, SynapsesTest };

//	Synapses: Smallest and most numerous unit in the cortex - the soldier at the bottom
// 		- TODO:
// 		- [high priority] Testing: 
// 			- [INCOMPLETE] Check for uniqueness and correct distribution frequency among src_slcs and cols
// 		- [low priority] Optimization:
// 			- [Complete] Obviously grow() and it's ilk need a lot of work
/*
	Synapse index space (for each of the synapse property Envoys) is first divided by tuft, then slice, then cell, then synapse. This means that even though a cell may have three (or any number of) tufts, and that you would naturally tend to think that synapse space would be first divided by slice, then cell, then tuft, tufts are moved to the front of that list. The reason for this is nuanced but it basically boils down to performance. When a kernel is processing synapses it's best to process tuft-at-a-time as the first order iteration rather than slice or cell-at-a-time because the each tuft inherently shares synapses whos axon sources are going to tend to be similar, making cache performance consistently better. This makes indexing very confusing so there's a definite trade off in complexity (for us poor humans). 

	Calculating a particular synapse index is shown below in syn_idx(). This is (hopefully) the exact same method the kernel uses for addressing: tuft is most significant, followed by slice, then cell, then synapse. Dendrites are not necessary to calculate a synapses index unless you happen only to have a synapses id (address) within a dendrite. Mostly the id within a cell is used and the dendrite is irrelevant, especially on the host side. 

	Synapse space breakdown (m := n - 1, n being the number of elements in any particular segment):
		- Tuft[0]
			- Slice[0]
				- Cell[0]
					- Synapse[0]
					...
					- Synapse[m]
				...
				- Cell[m]
					...
			- Slice[1]
			 	...
			...
			- Slice[m]
			 	...
		...
		- Tuft[m]
			...

	So even though tufts are, conceptually, children (sub-components) of a cell:
	|-->
	|	- Slice
	|		- Cell
	|--------<	- Tuft
					- Dendrite
						-Synapse

	 ... for indexing purposes tufts are parent to slices, which are parent to cells (then dendrites, then synapses).

*/


const DEBUG_NEW: bool = true;
const DEBUG_GROW: bool = true;
const DEBUG_REGROW_DETAIL: bool = false;


pub struct Synapses {
	layer_name: &'static str,
	dims: CorticalDimensions,
	syns_per_den_l2: u8,
	protocell: Protocell,
	//protoregion: ProtoLayerMap,
	dst_src_slc_ids: Vec<Vec<u8>>,
	den_kind: DendriteKind,
	cell_kind: ProtocellKind,
	since_decay: usize,
	kernels: Vec<Box<ocl::Kernel>>,
	src_idx_cache: SrcIdxCache,
	hex_tile_offs: Vec<(i8, i8)>,
	//kern_cycle: ocl::Kernel,
	//kern_regrow: ocl::Kernel,
	rng: XorShiftRng,
	pub states: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_char>,
	pub src_slc_ids: Envoy<ocl::cl_uchar>,
	pub src_col_u_offs: Envoy<ocl::cl_char>,
	pub src_col_v_offs: Envoy<ocl::cl_char>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	//pub slc_pool: Envoy<ocl::cl_uchar>,  // BRING THIS BACK (OPTIMIZATION)
}

impl Synapses {
	pub fn new(layer_name: &'static str, dims: CorticalDimensions, protocell: Protocell, 
					den_kind: DendriteKind, cell_kind: ProtocellKind, area_map: &AreaMap, 
					axons: &AxonSpace, /*aux: &Aux,*/ ocl: &ProQueue
	) -> Synapses {
		let syns_per_tft_l2: u8 = protocell.dens_per_tuft_l2 + protocell.syns_per_den_l2;
		assert!(dims.per_tft_l2() as u8 == syns_per_tft_l2);

		let wg_size = cmn::SYNAPSES_WORKGROUP_SIZE;
		let syn_reach = cmn::SYNAPSE_REACH as i8;

		let src_idx_cache = SrcIdxCache::new(protocell.syns_per_den_l2, protocell.dens_per_tuft_l2, dims.clone());

		//let slc_pool = Envoy::new(cmn::SYNAPSE_ROW_POOL_SIZE, 0, ocl); // BRING THIS BACK
		//let states = Envoy::<ocl::cl_uchar>::with_padding(32768, dims, 0, ocl);
		let states = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);
		let strengths = Envoy::<ocl::cl_char>::new(dims, 0, ocl);
		let src_slc_ids = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);


		//let src_col_u_offs = Envoy::<ocl::cl_char>::shuffled(dims, 0 - syn_reach, syn_reach + 1, ocl); // *****
		//let src_col_v_offs = Envoy::<ocl::cl_char>::shuffled(dims, 0 - syn_reach, syn_reach + 1, ocl); // *****
		let src_col_u_offs = Envoy::<ocl::cl_char>::new(dims, 0, ocl); // *****
		let src_col_v_offs = Envoy::<ocl::cl_char>::new(dims, 0, ocl); // *****


		let flag_sets = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);

		// KERNELS
		let dst_src_slc_ids = area_map.proto_layer_map().dst_src_slc_ids(layer_name);
		assert!(dst_src_slc_ids.len() == dims.tfts_per_cel() as usize,
			"Synapses::new(): Error creating synapses: layer '{}' has one or more invalid \
			source layers defined. If a source layer is an afferent or efferent input, please \
			ensure that the source area for that the layer exists.", layer_name);		

		let mut kernels = Vec::with_capacity(dst_src_slc_ids.len());

		if DEBUG_NEW { 
			println!("{mt}{mt}{mt}{mt}{mt}SYNAPSES::NEW(): kind: {:?}, len: {}, \
				dims: {:?}, phys_len: {}", 
				den_kind, states.len(), dims, dims.physical_len().unwrap(), mt = cmn::MT); 
		}

		let min_wg_sqrt = 8 as usize;
		assert_eq!((min_wg_sqrt * min_wg_sqrt), cmn::OPENCL_MINIMUM_WORKGROUP_SIZE as usize);

		// OBVIOUSLY THIS NAME IS CONFUSING: See above for explanation.
		let cel_tfts_per_syntuft = dims.cells();

		for tft_id in 0..dst_src_slc_ids.len() {
			kernels.push(Box::new(

				// ocl.new_kernel("syns_cycle_layer".to_string(),
				// ocl.new_kernel("syns_cycle_vec4_layer".to_string(),
				// ocl.new_kernel("syns_cycle_wow_layer".to_string(),
				ocl.new_kernel("syns_cycle_wow_vec4_layer".to_string(), 
					
					WorkSize::TwoDim(dims.v_size() as usize, (dims.u_size()) as usize))
					.lws(WorkSize::TwoDim(min_wg_sqrt, min_wg_sqrt))
					// WorkSize::ThreeDim(dims.depth() as usize, dims.v_size() as usize, (dims.u_size()) as usize))
					// .lws(WorkSize::ThreeDim(1, 8, 8 as usize)) // <<<<< TEMP UNTIL WE FIGURE OUT A WAY TO CALC THIS
					.arg_env(&axons.states)
					.arg_env(&src_col_u_offs)
					.arg_env(&src_col_v_offs)
					.arg_env(&src_slc_ids)
					//.arg_env(&strengths)
					.arg_scl(tft_id as u32 * cel_tfts_per_syntuft)
					.arg_scl(syns_per_tft_l2)
					.arg_scl(dims.depth() as u8)
					// .arg_env_named::<i32>("aux_ints_0", None)
					// .arg_env_named::<i32>("aux_ints_1", None)
					.arg_env(&states)
			))
		}


		// for tft_id in 0..dst_src_slc_ids.len() {
		// 	kernels.push(Box::new(

		// 		ocl.new_kernel("syns_cycle_simple".to_string(),
		// 		// ocl.new_kernel("syns_cycle_simple_vec4".to_string(),
		// 		// ocl.new_kernel("syns_cycle_wow".to_string(),
		// 		// ocl.new_kernel("syns_cycle_wow_vec4".to_string(), 
				
		// 			WorkSize::ThreeDim(dims.depth() as usize, dims.v_size() as usize, (dims.u_size()) as usize))
		// 			.lws(WorkSize::ThreeDim(1, 8, 8 as usize)) // <<<<< TEMP UNTIL WE FIGURE OUT A WAY TO CALC THIS
		// 			.arg_env(&axons.states)
		// 			.arg_env(&src_col_u_offs)
		// 			.arg_env(&src_col_v_offs)
		// 			.arg_env(&src_slc_ids)
		// 			//.arg_env(&strengths)
		// 			.arg_scl(tft_id as u32 * cel_tfts_per_syntuft)
		// 			.arg_scl(syns_per_tft_l2)
		// 			.arg_env(&aux.ints_0)
		// 			//.arg_env(&aux.ints_1)
		// 			.arg_env(&states)
		// 	))
		// }

		let mut syns = Synapses {
			layer_name: layer_name,
			dims: dims,
			syns_per_den_l2: protocell.syns_per_den_l2,
			protocell: protocell,
			//protoregion: protoregion.clone(),
			dst_src_slc_ids: dst_src_slc_ids,
			den_kind: den_kind,
			cell_kind: cell_kind,
			since_decay: 0,
			//kern_cycle: kern_cycle,
			kernels: kernels,
			src_idx_cache: src_idx_cache,
			hex_tile_offs: cmn::hex_tile_offs(cmn::SYNAPSE_REACH as i8),
			//kern_regrow: kern_regrow,
			rng: rand::weak_rng(),
			states: states,
			strengths: strengths,
			src_slc_ids: src_slc_ids,
			src_col_u_offs: src_col_u_offs,
			src_col_v_offs: src_col_v_offs,
			flag_sets: flag_sets,
			//slc_pool: slc_pool,  // BRING THIS BACK
		};

		//println!("\nHex tile offsets: \n{:?}", syns.hex_tile_offs);

		syns.grow(true);
		//syns.refresh_slc_pool();

		syns
	}


	fn grow(&mut self, init: bool) {
		if DEBUG_GROW && DEBUG_REGROW_DETAIL && !init {
			println!("RG:{:?}: [PRE:(SLICE)(OFFSET)(STRENGTH)=>($:UNIQUE, ^:DUPL)=>POST:(..)(..)(..)]\n", self.den_kind);
		}

		self.strengths.read();
		self.src_slc_ids.read();
		self.src_col_v_offs.read();

		let syns_per_layer_tft = self.dims.per_slc_per_tft() as usize * self.dims.depth() as usize;
		let dst_src_slc_ids = self.dst_src_slc_ids.clone();
		let mut src_tft_i = 0usize;

		for src_slc_ids in &dst_src_slc_ids {
			if src_slc_ids.len() == 0 { continue; }
			//assert!(src_slc_ids.len() > 0, "Synapses must have at least one source slice.");
			assert!(src_slc_ids.len() <= (self.dims.per_cel()) as usize, 
				"cortical_area::Synapses::init(): Number of source slcs must not exceed number of synapses per cell.");

			let syn_reach = cmn::SYNAPSE_REACH as i8;

			let src_slc_id_range: Range<usize> = Range::new(0, src_slc_ids.len());
			// let src_col_offs_range: Range<i8> = Range::new(0 - syn_reach, syn_reach + 1);
			let src_col_offs_range: Range<usize> = Range::new(0, self.hex_tile_offs.len());
			let strength_init_range: Range<i8> = Range::new(-3, 4);

			let syn_idz = syns_per_layer_tft * src_tft_i as usize;
			let syn_idn = syn_idz + syns_per_layer_tft as usize;

			if init && DEBUG_GROW {
				println!("{mt}{mt}{mt}{mt}{mt}\
					SYNAPSES::GROW()[INIT]: \"{}\" ({:?}): src_slc_ids: {:?}, \
					syns_per_layer_tft:{}, idz:{}, idn:{}", self.layer_name, self.den_kind, 
					src_slc_ids, syns_per_layer_tft, syn_idz, syn_idn, mt = cmn::MT);	
			}

			for syn_idx in syn_idz..syn_idn {
				if init || (self.strengths[syn_idx] <= cmn::SYNAPSE_STRENGTH_FLOOR) {
					//syn_idx = i + (src_slc_ids * 
					self.regrow_syn(syn_idx, &src_slc_id_range, &src_col_offs_range,
						&strength_init_range, &src_slc_ids, init);
				}
			}

			src_tft_i += 1;
		}

		self.strengths.write();
		self.src_slc_ids.write();
		self.src_col_v_offs.write();	
		self.src_col_u_offs.write();
	}

	fn regrow_syn(&mut self, 
				syn_idx: usize, 
				src_slc_idx_range: &Range<usize>, 
				src_col_offs_range: &Range<usize>,
				// src_col_offs_range: &Range<i8>,
				strength_init_range: &Range<i8>,
				src_slc_ids: &Vec<u8>,
				init: bool,
	) {

		// DEBUG
			//let mut print_str: String = String::with_capacity(10); 
			//let mut tmp_str = format!("[({})({})({})=>", self.src_slc_ids[syn_idx], self.src_col_v_offs[syn_idx],  self.strengths[syn_idx]);
			//print_str.push_str(&tmp_str);
		let syn_span = 2 * cmn::SYNAPSE_REACH as i8;

		loop {
			let old_ofs = AxnOfs { 
				slc: self.src_slc_ids[syn_idx], 
				v_ofs: self.src_col_v_offs[syn_idx],
				u_ofs: self.src_col_u_offs[syn_idx],
			};

			self.src_slc_ids[syn_idx] = src_slc_ids[src_slc_idx_range.ind_sample(&mut self.rng)];

			self.src_col_v_offs[syn_idx] = self.hex_tile_offs[src_col_offs_range.ind_sample(&mut self.rng)].0; 
			self.src_col_u_offs[syn_idx] = self.hex_tile_offs[src_col_offs_range.ind_sample(&mut self.rng)].1;			
			// self.src_col_v_offs[syn_idx] = src_col_offs_range.ind_sample(&mut self.rng);
			// self.src_col_u_offs[syn_idx] = src_col_offs_range.ind_sample(&mut self.rng);

			let intensity_reduction_l2 = 3;

			// <<<<< TODO: NEED SOMETHING SIMPLER/FASTER TO INIT STRENGTHS >>>>>
			let syn_str_intensity = (syn_span - 
					(self.src_col_v_offs[syn_idx].abs() + 
					self.src_col_u_offs[syn_idx].abs())
				) >> intensity_reduction_l2;

			self.strengths[syn_idx] = syn_str_intensity * strength_init_range.ind_sample(&mut self.rng);

			let new_ofs = AxnOfs { 
				slc: self.src_slc_ids[syn_idx], 
				v_ofs: self.src_col_v_offs[syn_idx],
				u_ofs: self.src_col_u_offs[syn_idx],
			};

			// <<<<< TODO: VERIFY AXON INDEX SAFETY >>>>>
			// 	- Will need to know u and v coords of host cell

			if self.src_idx_cache.insert(syn_idx, old_ofs, new_ofs) {
				//print_str.push_str("$"); // DEBUG
				break;
			} else {
				//print_str.push_str("^"); // DEBUG
			}
		}

		// DEBUG
			// tmp_str = format!("=>({})({})({})] ", self.src_slc_ids[syn_idx], self.src_col_v_offs[syn_idx],  self.strengths[syn_idx]);
			// print_str.push_str(&tmp_str);

			// if DEBUG_GROW && DEBUG_REGROW_DETAIL && !init {
			// 	print!("{}", print_str);
			// }
	}

	pub fn cycle(&self) {
		for kern in self.kernels.iter() {
			kern.enqueue();
		}
	}

	pub fn regrow(&mut self) {
		self.grow(false);
	}

	pub fn confab(&mut self) {
		self.states.read();
		self.strengths.read();
		self.src_slc_ids.read();
		self.src_col_v_offs.read();
	} 

	pub fn set_arg_env_named<T: OclNum>(&mut self, name: &'static str, env: &Envoy<T>) {
		let using_aux = false;

		if using_aux {
			for kernel in self.kernels.iter_mut() {
				kernel.set_arg_env_named(name, env);
			}
		}
	}

	pub fn den_kind(&self) -> DendriteKind {
		self.den_kind.clone()
	}

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

	pub fn syns_per_den_l2(&self) -> u8 {
		self.syns_per_den_l2
	}

	pub fn syns_per_tftsec(&self) -> u32 {
		let slcs_per_tftsec = self.dims.depth();
		let cels_per_slc = self.dims.columns();
		let syns_per_cel_tft = self.dims.per_tft();

		slcs_per_tftsec as u32 * cels_per_slc * syns_per_cel_tft
	}

	// [FIXME] TODO: Depricate me evenutally
	pub fn set_offs_to_zero_temp(&mut self) {
		self.src_col_v_offs.set_all_to(0);
		self.src_col_u_offs.set_all_to(0);
	}

	/* SRC_SLICE_IDS(): TODO: DEPRICATE */
	// pub fn src_slc_ids(&self, layer_name: &'static str, layer: &Protolayer) -> Vec<u8> {
		
	// 	//println!("\n##### SYNAPSES::SRC_SLICE_IDS({}): {:?}", layer_name, self.dst_src_slc_ids);

	// 	match layer.kind {
	// 		ProtolayerKind::Cellular(ref cell) => {
	// 			if cell.cell_kind == self.cell_kind {
	// 				self.protoregion.src_slc_ids(layer_name, self.den_kind)
	// 			} else {
	// 				panic!("Synapse::src_slc_ids(): cell_kind mismatch! ")
	// 			}
	// 		},

	// 		_ => panic!("Synapse::src_slc_ids(): ProtolayerKind not Cellular! "),
	// 	}
	// }
}


struct SrcIdxCache {
	syns_per_den_l2: u8,
	dens_per_tft_l2: u8,
	dims: CorticalDimensions,
	dens: Vec<Box<BTreeSet<i32>>>,
}

impl SrcIdxCache {
	fn new(syns_per_den_l2: u8, dens_per_tft_l2: u8, dims: CorticalDimensions) -> SrcIdxCache {
		let dens_per_tft = 1 << dens_per_tft_l2 as u32;
		let area_dens = (dens_per_tft * dims.cel_tfts()) as usize;
		let mut dens = Vec::with_capacity(dens_per_tft as usize);

		for i in 0..area_dens {	dens.push(Box::new(BTreeSet::new())); }

		//println!("##### CREATING SRCIDXCACHE WITH: dens: {}", dens.len());

		SrcIdxCache {
			syns_per_den_l2: syns_per_den_l2,
			dens_per_tft_l2: dens_per_tft_l2,
			dims: dims,
			dens: dens,
		}
	}

	pub fn insert(&mut self, syn_idx: usize, new_ofs: AxnOfs, old_ofs: AxnOfs) -> bool {
		let den_idx = syn_idx >> self.syns_per_den_l2;

		let new_ofs_key: i32 = self.axn_ofs(&new_ofs);
		let is_unique: bool = self.dens[den_idx].insert(new_ofs_key);

		if is_unique {
			let old_ofs_key: i32 = self.axn_ofs(&old_ofs);
			self.dens[den_idx].remove(&old_ofs_key);
		}

		is_unique
	}

	fn axn_ofs(&self, axn_ofs: &AxnOfs) -> i32 {
		(axn_ofs.slc as i32 * self.dims.columns() as i32) 
		+ (axn_ofs.v_ofs as i32 * self.dims.u_size() as i32)
		+ axn_ofs.u_ofs as i32
	}
}

struct AxnOfs {
	slc: u8,
	v_ofs: i8,
	u_ofs: i8,
}



#[cfg(test)]
pub mod tests {
	#![allow(non_snake_case)]
	use std::ops::{ Range };
	use std::fmt::{ Display, Formatter, Result as FmtResult };
	use rand::{ XorShiftRng };
	use rand::distributions::{ IndependentSample, Range as RandRange };

	use cmn::{ CelCoords };
	use cmn::{ CorticalDimensions };
	use dendrites::{ self };
	use super::{ Synapses };

	const PRINT_DEBUG_INFO: bool = false;

	pub trait SynapsesTest {
		fn set_offs_to_zero(&mut self);
		fn set_all_to_zero(&mut self);
		fn set_src_offs(&self, v_ofs: i8, u_ofs: i8, idx: usize);
		fn set_src_slc(&self, src_slc_id: u8, idx: usize);
		fn syn_state(&self, idx: u32) -> u8;
		fn rand_syn_coords(&mut self, cel_coords: &CelCoords) -> SynCoords;
		fn print_range(&mut self, range: Range<usize>);
		fn print_all(&mut self);
		fn rng(&mut self) -> &mut XorShiftRng;	
	}

	impl SynapsesTest for Synapses {
		fn set_offs_to_zero(&mut self) {
			self.src_col_v_offs.set_all_to(0);
			self.src_col_u_offs.set_all_to(0);
		}

		fn set_all_to_zero(&mut self) {
			self.states.set_all_to(0);
			self.strengths.set_all_to(0);
			self.src_slc_ids.set_all_to(0);
			self.src_col_u_offs.set_all_to(0);
			self.src_col_v_offs.set_all_to(0);
			self.flag_sets.set_all_to(0);
		}

		fn set_src_offs(&self, v_ofs: i8, u_ofs: i8, idx: usize) {
			let sdr_v = vec![v_ofs];
			let sdr_u = vec![u_ofs];
			self.src_col_v_offs.write_direct(&sdr_v[..], idx);
			self.src_col_u_offs.write_direct(&sdr_u[..], idx);
		}

		fn set_src_slc(&self, src_slc_id: u8, idx: usize) {
			let sdr = vec![src_slc_id];
			self.src_slc_ids.write_direct(&sdr[..], idx);
		}

		fn syn_state(&self, idx: u32) -> u8 {
			let mut sdr = vec![0u8];
			self.states.read_direct(&mut sdr[..], idx as usize);
			sdr[0]
		}

		fn rand_syn_coords(&mut self, cel_coords: &CelCoords) -> SynCoords {
			let tft_id_range = RandRange::new(0, cel_coords.tfts_per_cel);
			let den_id_tft_range = RandRange::new(0, 1 << (cel_coords.dens_per_tft_l2 as u32));
			let syn_id_den_range = RandRange::new(0, 1 << (cel_coords.syns_per_den_l2 as u32));

			let tft_id = tft_id_range.ind_sample(&mut self.rng); 
			let den_id_tft = den_id_tft_range.ind_sample(&mut self.rng);
			let syn_id_den = syn_id_den_range.ind_sample(&mut self.rng);

			SynCoords::new(tft_id, den_id_tft, syn_id_den, cel_coords)
		}

		fn print_range(&mut self, range: Range<usize>) {
			print!("syns.states: ");
			self.states.print(1 << 0, Some((0, 255)), 
				Some(range.clone()), false);

			print!("syns.flag_sets: ");
			self.flag_sets.print(1 << 0, Some((0, 255)), 
				Some(range.clone()), false);

			print!("syns.strengths: ");
			self.strengths.print(1 << 0, Some((-128, 127)), 
				Some(range.clone()), false);

			// print!("syns.src_slc_ids: ");
			// self.src_slc_ids.print(1 << 0, Some((0, 255)), 
			// 	Some(range.clone()), false);

			// print!("syns.src_col_v_offs: ");
			// self.src_col_v_offs.print(1 << 0, Some((-128, 127)), 
			// 	Some(range.clone()), false);
			
			// print!("syns.src_col_u_offs: ");
			// self.src_col_v_offs.print(1 << 0, Some((-128, 127)), 
			// 	Some(range.clone()), false);
		}

		fn print_all(&mut self) {
			let range = 0..self.states.len();
			self.print_range(range);
		}

		fn rng(&mut self) -> &mut XorShiftRng {
			&mut self.rng
		}		
	}

	// <<<<< TODO: NEEDS UPDATING TO MATCH / INTEGRATE WITH DEN_COORDS >>>>>
	#[derive(Debug, Clone)]
	pub struct SynCoords {
		pub idx: u32,	
		pub tft_id: u32,
		pub den_id_tft: u32,
		pub syn_id_den: u32,		
		pub cel_coords: CelCoords,
		// pub layer_dims: CorticalDimensions,
	}

	impl SynCoords {
		pub fn new(tft_id: u32, den_id_tft: u32, syn_id_den: u32, cel_coords: &CelCoords, 
					// layer_dims: &CorticalDimensions
			) -> SynCoords 
		{
			// let syns_per_tft = 1 << (cel_coords.dens_per_tft_l2 as u32 
			// 	+ cel_coords.syns_per_den_l2 as u32);

			// 'tft_count' is synonymous with 'tfts_per_cel':
			let tft_count = cel_coords.tfts_per_cel;
			let syns_per_den = 1 << (cel_coords.syns_per_den_l2 as u32);
			let dens_per_tft = 1 << (cel_coords.dens_per_tft_l2 as u32);

			let syn_idx = syn_idx(&cel_coords.layer_dims, tft_count, dens_per_tft, 
				syns_per_den, tft_id, cel_coords.idx, den_id_tft, syn_id_den);

			SynCoords { 
				idx: syn_idx, 
				tft_id: tft_id,
				den_id_tft: den_id_tft,
				syn_id_den: syn_id_den, 				
				cel_coords: cel_coords.clone(),
				// layer_dims: layer_dims.clone(),
			}
		}

		pub fn syn_idx_range_tft(&self) -> Range<usize> {
			let tft_count = self.cel_coords.tfts_per_cel;
			let syns_per_den = 1 << (self.cel_coords.syns_per_den_l2 as u32);
			let dens_per_tft = 1 << (self.cel_coords.dens_per_tft_l2 as u32);			

			// Get the idz for the synapse on this tuft with: den_id_tft = 0, syn_id_den = 0:
			let syn_idz_cel_tft = syn_idx(&self.cel_coords.layer_dims, tft_count, dens_per_tft, 
				syns_per_den, self.tft_id, self.cel_coords.idx, 0, 0) as usize;

			let syns_per_tft = syns_per_den * dens_per_tft;

			syn_idz_cel_tft..(syn_idz_cel_tft + syns_per_tft as usize)
		}

		pub fn syn_idx_range_den(&self) -> Range<usize> {
			let tft_count = self.cel_coords.tfts_per_cel;
			let syns_per_den = 1 << (self.cel_coords.syns_per_den_l2 as u32);
			let dens_per_tft = 1 << (self.cel_coords.dens_per_tft_l2 as u32);			

			// Get the idz for the synapse on this dendrite with: syn_id_den = 0:
			let syn_idz_den = syn_idx(&self.cel_coords.layer_dims, tft_count, dens_per_tft, 
				syns_per_den, self.tft_id, self.cel_coords.idx, self.den_id_tft, 0) as usize;

			syn_idz_den..(syn_idz_den + syns_per_den as usize)
		}

		// [FIXME] TODO: MOVE THIS TO DEN_COORDS & INTEGRATE
		pub fn tft_idx(&self) -> u32 {
			(self.tft_id * self.cel_coords.layer_dims.cells()) + self.cel_coords.idx
		}

		pub fn den_idx(&self) -> u32 {
			let den_dims = self.cel_coords.layer_dims
				.clone_with_ptl2(self.cel_coords.dens_per_tft_l2 as i8)
				.with_tfts(self.cel_coords.tfts_per_cel);

			dendrites::den_idx(&den_dims, self.tft_id, self.cel_coords.idx, self.den_id_tft)
		}

		pub fn idx(&self) -> u32 {
			self.idx
		}
	}

	impl Display for SynCoords {
	    fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
	        write!(fmtr, "SynCoords {{ idx: {}, tft_id: {}, den_id_tft: {} syn_id_den: {}, parent_cel: {} }}", 
				self.idx, self.tft_id, self.den_id_tft, self.syn_id_den, self.cel_coords)
	    }
	}

	#[test]
	fn test_source_uniqueness_UNIMPLEMENTED() {
		// UNIMPLEMENTED
	}



	// SYN_IDX(): FOR TESTING/DEBUGGING AND A LITTLE DOCUMENTATION
	// 		- Synapse index space heirarchy:  | Tuft - Slice - Cell - Dendrite - Synapse |
	// 		- 'cel_idx' already has slice built in to its value
	// 		- 'tft_count' is synonymous with 'tfts_per_cel'
	// 		- X_cel_tft is synonymous with X_tft but is verbosely described for clarity
	pub fn syn_idx(cel_layer_dims: &CorticalDimensions, tft_count: u32, dens_per_cel_tft: u32, syns_per_den: u32, 
					tft_id: u32, cel_idx: u32, den_id_cel_tft: u32, syn_id_den: u32) -> u32 
	{
		//  NOTE: 'cel_layer_dims' expresses dimensions from the perspective of the 
		//  | Slice - Cell - Tuft - Dendrite - Synapse | heirarchy which is why the function
		//  names (and other variable names) seem confusing (see explanation at top of file).

		let slcs_per_tftsec = cel_layer_dims.depth() as u32;
		let cels_per_slc = cel_layer_dims.columns();

		assert!(tft_id < tft_count);
		assert!(cel_idx < slcs_per_tftsec * cels_per_slc);
		assert!(den_id_cel_tft < dens_per_cel_tft);
		assert!(syn_id_den < syns_per_den);


		let syns_per_tftsec = slcs_per_tftsec * cels_per_slc * dens_per_cel_tft * syns_per_den;
		let syn_idz_tftsec = tft_id * syns_per_tftsec;
		// 'cel_idx' includes slc_id, v_id, and u_id:
		let syn_idz_tft_slc_cel = cel_idx * dens_per_cel_tft * syns_per_den;
		let syn_id_cel_tft = (den_id_cel_tft * syns_per_den) + syn_id_den;

		let syn_idx = syn_idz_tftsec + syn_idz_tft_slc_cel + syn_id_cel_tft;

		if PRINT_DEBUG_INFO {
			println!("\n#####\n\n\
				tft_count: {} \n\
				slcs_per_tftsec: {} \n\
				cels_per_slc: {}\n\
				syns_per_den: {}\n\
				\n\
				cel_idx: {},\n\
				tft_id: {},\n\
				den_id_tft: {}, \n\
				syn_id_den: {}, \n\
				\n\
				syn_idz_tftsec: {},\n\
				syn_idz_tft_slc_cel: {},\n\
				syn_idx: {},\n\
				\n\
				#####", 
				tft_count, slcs_per_tftsec, cels_per_slc, syns_per_den, 
				cel_idx, tft_id, den_id_cel_tft, syn_id_den, 
				syn_idz_tftsec, syn_idz_tft_slc_cel, syn_idx,
			);
		}

		syn_idx
	}
}

