// use std::fmt;
use rand::{ /*self,*/ XorShiftRng };
use rand::distributions::{ IndependentSample, Range as RandRange };
use std::collections::{ BTreeMap, BTreeSet };

use cmn::{ self, CorticalDims };
use map::{ AreaMap, SliceDims, AxonKind };
// use proto::{ CellKind, Protocell, DendriteKind };

const INTENSITY_REDUCTION_L2: i8 = 3;
const STR_MIN: i8 = -3;
const STR_MAX: i8 = 4;

/// Information about the boundaries and synapse ranges for each source slice, on
/// each tuft.
///
/// Used to calculate a valid source axon index during synapse growth or regrowth.
pub struct SrcSlices {
	tft_slcs: Vec<BTreeMap<u8, SliceInfo>>,	
	slc_ids: Vec<Vec<u8>>,
	slc_id_ranges: Vec<RandRange<usize>>,
	str_ranges: Vec<RandRange<i8>>,
	// rng: XorShiftRng,
}

impl SrcSlices {
	pub fn new(src_slc_ids_by_tft: &Vec<Vec<u8>>, syn_reaches_by_tft: Vec<u8>, area_map: &AreaMap
			) -> SrcSlices 
	{
		let mut tft_slcs = Vec::with_capacity(src_slc_ids_by_tft.len());
		let mut slc_id_ranges = Vec::with_capacity(tft_slcs.len());
		let mut str_ranges = Vec::with_capacity(tft_slcs.len());

		let mut tft_id = 0;

		for src_slc_ids in src_slc_ids_by_tft.iter() {
			let mut slcs = BTreeMap::new();
			let syn_reach = *syn_reaches_by_tft.get(tft_id).expect("SrcSlices::new(): {{1}}");

			slc_id_ranges.push(RandRange::new(0, src_slc_ids.len()));
			str_ranges.push(RandRange::new(STR_MIN, STR_MAX));

			for &slc_id in src_slc_ids {				
				let axn_kind = area_map.slices().axn_kinds().get(slc_id as usize)
					.expect("SrcSlices::new(): {{2}}");
				let dims = area_map.slices().dims().get(slc_id as usize)
					.expect("SrcSlices::new(): {{3}}");

				slcs.insert(slc_id, SliceInfo::new(axn_kind, dims, syn_reach))
					.map(|_| panic!("SrcSlices::new(): {{4}}"));
			}

			tft_slcs.push(slcs);
			tft_id += 1;
		}

		SrcSlices { tft_slcs: tft_slcs, slc_id_ranges: slc_id_ranges, 
			slc_ids: src_slc_ids_by_tft.clone(), str_ranges: str_ranges, }
	}

	// [FIXME]: TODO: MORE THOROUGH BOUNDS CHECKING. These unchecked array accesses are potentially dangerous.
	// [FIXME]: TODO: Bounds check ofs against v and u id (will need to figure out how to deconstruct this from the syn_idx or something).
	pub fn gen_src(&self, tft_id: usize, rng: &mut XorShiftRng) -> SynSrc {
		debug_assert!(tft_id < self.slc_ids.len() && tft_id < self.tft_slcs.len());

		let slc_id = unsafe { *self.slc_ids.get_unchecked(tft_id).get_unchecked(
			self.slc_id_ranges.get_unchecked(tft_id).ind_sample(rng)) };

		let slc_info = unsafe { &self.tft_slcs.get_unchecked(tft_id)
			.get(&slc_id).expect("SrcSlices::gen_offs(): Internal error: invalid slc_id.") };

		match slc_info.slc_off_pool {
			OfsPool::Horizontal((ref v_rr, ref u_rr)) => {
				SynSrc { 
					slc_id: slc_id,
					v_ofs: v_rr.ind_sample(rng),
					u_ofs: u_rr.ind_sample(rng),
					strength: 0,
				}
			},

			OfsPool::Spatial((ref offs, ref range)) => {
				let (v_ofs, u_ofs) = offs[range.ind_sample(rng)];

				let syn_str_intensity = (
					slc_info.syn_reach() as i8
					- (v_ofs.abs() + u_ofs.abs())
					) >> INTENSITY_REDUCTION_L2;

				let strength = syn_str_intensity * 
					unsafe {self.str_ranges.get_unchecked(tft_id).ind_sample(rng) };

				SynSrc { 
					slc_id: slc_id, 
					v_ofs: v_ofs,
					u_ofs: u_ofs,
					strength: strength,
				}
			},
		}
	}
}



pub struct SliceInfo {
	slc_off_pool: OfsPool,
	v_size: u32,
	u_size: u32,
	syn_reach: u8,
}

impl SliceInfo {
	pub fn new(axn_kind: &AxonKind, slc_dims: &SliceDims, syn_reach: u8) -> SliceInfo {
		let slc_off_pool = match axn_kind {
			&AxonKind::Horizontal => {
				// Already checked within SliceDims.
				debug_assert!(slc_dims.v_size() <= 252);
				debug_assert!(slc_dims.u_size() <= 252);

				// [FIXME] Tweak how the middle and ranges are calc'd!
				// Adjust SliceDims if necessary.

				let v_reach = (slc_dims.v_size() / 2) as i8;
				let u_reach = (slc_dims.u_size() / 2) as i8;
				
				OfsPool::Horizontal((
					RandRange::new(0 - v_reach, v_reach + 1),
					RandRange::new(0 - u_reach, u_reach + 1), ))
			},

			&AxonKind::Spatial | &AxonKind::None => {
				let hex_tile_offs = cmn::hex_tile_offs(syn_reach);
				let len = hex_tile_offs.len();

				OfsPool::Spatial((
					hex_tile_offs, 
					RandRange::new(0, len), ))
			},
		};

		SliceInfo { 
			slc_off_pool: slc_off_pool,
			v_size: slc_dims.v_size(),
			u_size: slc_dims.u_size(),
			syn_reach: syn_reach,
		}
	}

	pub fn slc_off_pool(&self) -> &OfsPool {
		&self.slc_off_pool
	}

	pub fn syn_reach(&self) -> u8 {
		self.syn_reach
	}
}



pub enum OfsPool {
	Horizontal((RandRange<i8>, RandRange<i8>)),
	Spatial((Vec<(i8, i8)>, RandRange<usize>)),
}

// impl fmt::Debug for OfsPool {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // f.write_fmt(format_args!());
//         match self {
//         	&OfsPool::Horizontal(_) => write!(f, "OfsPool::Horizontal"),
//         	&OfsPool::Spatial(_) => write!(f, "OfsPool::Spatial"),
//     	}
//     }
// }



pub struct SrcIdxCache {
	syns_per_den_l2: u8,
	dens_per_tft_l2: u8,
	dims: CorticalDims,
	dens: Vec<Box<BTreeSet<i32>>>,
}

impl SrcIdxCache {
	pub fn new(syns_per_den_l2: u8, dens_per_tft_l2: u8, dims: CorticalDims) -> SrcIdxCache {
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

	pub fn insert(&mut self, syn_idx: usize, new_ofs: SynSrc, old_ofs: SynSrc) -> bool {
		let den_idx = syn_idx >> self.syns_per_den_l2;

		let new_ofs_key: i32 = self.axn_ofs(&new_ofs);
		let is_unique: bool = self.dens[den_idx].insert(new_ofs_key);

		if is_unique {
			let old_ofs_key: i32 = self.axn_ofs(&old_ofs);
			self.dens[den_idx].remove(&old_ofs_key);
		}

		is_unique
	}

	fn axn_ofs(&self, axn_ofs: &SynSrc) -> i32 {
		(axn_ofs.slc_id as i32 * self.dims.columns() as i32) 
		+ (axn_ofs.v_ofs as i32 * self.dims.u_size() as i32)
		+ axn_ofs.u_ofs as i32
	}
}

pub struct SynSrc {
	pub slc_id: u8,
	pub v_ofs: i8,
	pub u_ofs: i8,
	pub strength: i8,
}

