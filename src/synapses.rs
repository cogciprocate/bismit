use rand::{ self, XorShiftRng };

use cmn::{ self, CorticalDims };
use map::{ AreaMap, SrcSlices, SrcIdxCache, SynSrc };
use ocl::{ cl_uchar, cl_char, ProQue, WorkSize, Envoy, OclNum, EventList, Kernel };
use proto::{ CellKind, Protocell, DendriteKind };
use axon_space::{ AxonSpace };

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

	So even though tufts are, conceptually, children (sub-components) of a cell...
	+-->
	|	- Slice
	|		- Cell
	+--------<	- Tuft
					- Dendrite
						-Synapse

	 ... **for indexing purposes** tufts are parent to slices, which are parent to cells (then dendrites, then synapses).

*/


const DEBUG_NEW: bool = true;
const DEBUG_GROW: bool = true;
const DEBUG_REGROW_DETAIL: bool = false;


pub struct Synapses {
	layer_name: &'static str,
	dims: CorticalDims,
	syns_per_den_l2: u8,
	protocell: Protocell,
	src_slc_ids_by_tft: Vec<Vec<u8>>,
	den_kind: DendriteKind,
	cell_kind: CellKind,
	since_decay: usize,
	kernels: Vec<Box<Kernel>>,
	src_idx_cache: SrcIdxCache,
	src_slcs: SrcSlices,
	rng: XorShiftRng,
	pub states: Envoy<cl_uchar>,
	pub strengths: Envoy<cl_char>,
	pub src_slc_ids: Envoy<cl_uchar>,
	pub src_col_u_offs: Envoy<cl_char>,
	pub src_col_v_offs: Envoy<cl_char>,
	pub flag_sets: Envoy<cl_uchar>,
	// pub slc_pool: Envoy<cl_uchar>,  // BRING THIS BACK (OPTIMIZATION)
}

impl Synapses {
	pub fn new(layer_name: &'static str, dims: CorticalDims, protocell: Protocell, 
				den_kind: DendriteKind, cell_kind: CellKind, area_map: &AreaMap, 
				axons: &AxonSpace, ocl_pq: &ProQue
			) -> Synapses 
	{
		let syns_per_tft_l2: u8 = protocell.dens_per_tuft_l2 + protocell.syns_per_den_l2;
		assert!(dims.per_tft_l2() as u8 == syns_per_tft_l2);

		let src_idx_cache = SrcIdxCache::new(protocell.syns_per_den_l2, 
			protocell.dens_per_tuft_l2, dims.clone());

		// let slc_pool = Envoy::with_vec(cmn::SYNAPSE_ROW_POOL_SIZE, 0, ocl_pq); // BRING THIS BACK
		let states = Envoy::<cl_uchar>::with_vec(dims, ocl_pq.queue());
		let strengths = Envoy::<cl_char>::with_vec(dims, ocl_pq.queue());
		let src_slc_ids = Envoy::<cl_uchar>::with_vec(dims, ocl_pq.queue());

		let src_col_u_offs = Envoy::<cl_char>::with_vec(dims, ocl_pq.queue());
		let src_col_v_offs = Envoy::<cl_char>::with_vec(dims, ocl_pq.queue()); 
		let flag_sets = Envoy::<cl_uchar>::with_vec(dims, ocl_pq.queue());

		// [FIXME]: TODO: Integrate src_slc_ids for any type of dendrite.
		let (src_slc_ids_by_tft, syn_reaches_by_tft) = match den_kind {
			DendriteKind::Proximal => {
				(vec![area_map.layer_src_slc_ids(layer_name, den_kind)],
					vec![protocell.den_prx_syn_reach])
			},
			DendriteKind::Distal => {
				(area_map.layer_dst_src_slc_ids(layer_name),
					protocell.den_dst_syn_reaches.clone())
			},
		};

		assert!(src_slc_ids_by_tft.len() == dims.tfts_per_cel() as usize,
			"Synapses::new(): Error creating synapses: layer '{}' has one or more invalid \
			source layers defined. If a source layer is an afferent or efferent input, please \
			ensure that the source area for that the layer exists. (src_slc_ids_by_tft: {:?})", 
			layer_name, src_slc_ids_by_tft);

		// [FIXME]: Implement src_ranges on a per-tuft basis.
		// let syn_reaches_by_tft: Vec<u8> = src_slc_ids_by_tft.iter().map(|_| syn_reach).collect();
		let src_slcs = SrcSlices::new(&src_slc_ids_by_tft, syn_reaches_by_tft, area_map);		

		if DEBUG_NEW { 
			println!("{mt}{mt}{mt}{mt}SYNAPSES::NEW(): kind: {:?}, len: {}, \
				dims: {:?}, phys_len: {},", 
				den_kind, states.len(), dims, dims.padded_envoy_len(
					ocl_pq.get_max_work_group_size()), mt = cmn::MT); 
		}

		let min_wg_sqrt = 8 as usize;
		assert_eq!((min_wg_sqrt * min_wg_sqrt), cmn::OPENCL_MINIMUM_WORKGROUP_SIZE as usize);

		// OBVIOUSLY THIS NAME IS CONFUSING: See above for explanation.
		let cel_tfts_per_syntuft = dims.cells();

		let mut kernels = Vec::with_capacity(src_slc_ids_by_tft.len());

		for tft_id in 0..src_slc_ids_by_tft.len() {
			kernels.push(Box::new(

				// ocl_pq.create_kernel("syns_cycle_layer",
				// ocl_pq.create_kernel("syns_cycle_vec4_layer",
				// ocl_pq.create_kernel("syns_cycle_wow_layer",
				ocl_pq.create_kernel("syns_cycle_wow_vec4_layer", 
					
					WorkSize::TwoDims(dims.v_size() as usize, (dims.u_size()) as usize))
					.lws(WorkSize::TwoDims(min_wg_sqrt, min_wg_sqrt))
					.arg_env(&axons.states)
					.arg_env(&src_col_u_offs)
					.arg_env(&src_col_v_offs)
					.arg_env(&src_slc_ids)
					.arg_scl(tft_id as u32 * cel_tfts_per_syntuft)
					.arg_scl(syns_per_tft_l2)
					.arg_scl(dims.depth() as u8)
					// .arg_env_named::<i32>("aux_ints_0", None)
					// .arg_env_named::<i32>("aux_ints_1", None)
					.arg_env(&states)
			))
		}


		let mut syns = Synapses {
			layer_name: layer_name,
			dims: dims,
			syns_per_den_l2: protocell.syns_per_den_l2,
			protocell: protocell,
			src_slc_ids_by_tft: src_slc_ids_by_tft,
			den_kind: den_kind,
			cell_kind: cell_kind,
			since_decay: 0,
			kernels: kernels,
			src_idx_cache: src_idx_cache,
			src_slcs: src_slcs,
			rng: rand::weak_rng(),
			states: states,
			strengths: strengths,
			src_slc_ids: src_slc_ids,
			src_col_u_offs: src_col_u_offs,
			src_col_v_offs: src_col_v_offs,
			flag_sets: flag_sets,
			// slc_pool: slc_pool,  // BRING THIS BACK
		};

		syns.grow(true);
		// syns.refresh_slc_pool(); // BRING THIS BACK

		syns
	}


	fn grow(&mut self, init: bool) {
		if DEBUG_GROW && DEBUG_REGROW_DETAIL && !init {
			println!("REGROW:{:?}: [PRE:(SLICE)(OFFSET)(STRENGTH)=>($:UNIQUE, ^:DUPL)=>POST:\
				(SLICE)(OFFSET)(STRENGTH)]\n", self.den_kind);
		}

		self.strengths.fill_vec_wait();
		self.src_slc_ids.fill_vec_wait();
		self.src_col_v_offs.fill_vec_wait();

		let syns_per_layer_tft = self.dims.per_slc_per_tft() as usize * self.dims.depth() as usize;
		let src_slc_ids_by_tft = self.src_slc_ids_by_tft.clone();
		let mut src_tft_id = 0usize;

		for src_slc_id_list in &src_slc_ids_by_tft {
			if src_slc_id_list.len() == 0 { continue; }

			let syn_idz = syns_per_layer_tft * src_tft_id as usize;
			let syn_idn = syn_idz + syns_per_layer_tft as usize;

			if DEBUG_GROW && init {
				println!("{mt}{mt}{mt}{mt}{mt}\
					SYNAPSES::GROW()[INIT]: '{}' ({:?}): src_slc_ids: {:?}, \
					syns_per_layer_tft:{}, idz:{}, idn:{}", self.layer_name, self.den_kind, 
					src_slc_id_list, syns_per_layer_tft, syn_idz, syn_idn, mt = cmn::MT);	
			}

			for syn_idx in syn_idz..syn_idn {
				debug_assert!(syn_idx < self.strengths.len());

				if init || (unsafe { *self.strengths
					.get_unchecked(syn_idx) } <= cmn::SYNAPSE_STRENGTH_FLOOR) 
				{
					self.regrow_syn(syn_idx, src_tft_id, init);
				}
			}

			src_tft_id += 1;
		}

		self.strengths.flush_vec_wait();
		self.src_slc_ids.flush_vec_wait();
		self.src_col_v_offs.flush_vec_wait();	
		self.src_col_u_offs.flush_vec_wait();
	}

	// [FIXME] TODO: VERIFY AXON INDEX SAFETY (notes below and in syn_src_map.rs).
	// - Will need to know u and v coords of host cell or deconstruct from syn_idx.
	// [FIXME] TODO: Remove synapse index bounds checks (.get_unchecked()...).
	// [FIXME][COMPLETE]: Implement per-slice syn_ranges.
	fn regrow_syn(&mut self, syn_idx: usize, tft_id: usize, init: bool) {		
		debug_assert!(syn_idx < self.src_slc_ids.len());
		debug_assert!(syn_idx < self.src_col_v_offs.len());
		debug_assert!(syn_idx < self.src_col_u_offs.len());

		loop {
			let old_src = unsafe { SynSrc { 
				slc_id: *self.src_slc_ids.get_unchecked(syn_idx), 
				v_ofs: *self.src_col_v_offs.get_unchecked(syn_idx),
				u_ofs: *self.src_col_u_offs.get_unchecked(syn_idx),
				strength: 0
			} };

			let new_src = self.src_slcs.gen_src(tft_id, &mut self.rng);

			if self.src_idx_cache.insert(syn_idx, &old_src, &new_src) {
				unsafe {
					*self.src_slc_ids.get_unchecked_mut(syn_idx) = new_src.slc_id;
					*self.src_col_v_offs.get_unchecked_mut(syn_idx) = new_src.v_ofs; 
					*self.src_col_u_offs.get_unchecked_mut(syn_idx) = new_src.u_ofs;
					*self.strengths.get_unchecked_mut(syn_idx) = new_src.strength;
				}

				if DEBUG_GROW && DEBUG_REGROW_DETAIL { print!("$"); }
				break;
			} else {
				if DEBUG_GROW && DEBUG_REGROW_DETAIL { print!("^"); }
			}
		}
	}

	pub fn cycle(&self, wait_events: Option<&EventList>) {
		for kern in self.kernels.iter() {
			kern.enqueue(wait_events, None);
		}
	}

	pub fn regrow(&mut self) {
		self.grow(false);
	}

	pub fn confab(&mut self) {
		self.states.fill_vec_wait();
		self.strengths.fill_vec_wait();
		self.src_slc_ids.fill_vec_wait();
		self.src_col_v_offs.fill_vec_wait();
	} 

	// Debugging purposes
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

	pub fn dims(&self) -> &CorticalDims {
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

}



#[cfg(test)]
pub mod tests {
	#![allow(non_snake_case)]
	use std::ops::{ Range };
	use std::fmt::{ Display, Formatter, Result as FmtResult };
	use rand::{ XorShiftRng };
	use rand::distributions::{ IndependentSample, Range as RandRange };

	use cmn::{ CelCoords };
	use cmn::{ CorticalDims };
	use dendrites::{ self };
	use super::{ Synapses };

	const PRINT_DEBUG_INFO: bool = false;

	pub trait SynapsesTest {
		fn set_offs_to_zero(&mut self);
		fn set_all_to_zero(&mut self);
		fn set_src_offs(&mut self, v_ofs: i8, u_ofs: i8, idx: usize);
		fn set_src_slc(&mut self, src_slc_id: u8, idx: usize);
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

		fn set_src_offs(&mut self, v_ofs: i8, u_ofs: i8, idx: usize) {
			let sdr_v = vec![v_ofs];
			let sdr_u = vec![u_ofs];
			self.src_col_v_offs.write(&sdr_v[..], idx, None, None);
			self.src_col_u_offs.write(&sdr_u[..], idx, None, None);
		}

		fn set_src_slc(&mut self, src_slc_id: u8, idx: usize) {
			let sdr = vec![src_slc_id];
			self.src_slc_ids.write(&sdr[..], idx, None, None);
		}

		fn syn_state(&self, idx: u32) -> u8 {
			let mut sdr = vec![0u8];
			self.states.read(&mut sdr[..], idx as usize, None, None);
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
		// pub layer_dims: CorticalDims,
	}

	impl SynCoords {
		pub fn new(tft_id: u32, den_id_tft: u32, syn_id_den: u32, cel_coords: &CelCoords, 
					// layer_dims: &CorticalDims
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
	pub fn syn_idx(cel_layer_dims: &CorticalDims, tft_count: u32, dens_per_cel_tft: u32, syns_per_den: u32, 
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

