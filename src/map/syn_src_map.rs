// use std::fmt;
use rand::{ /*self,*/ XorShiftRng };
use rand::distributions::{ IndependentSample, Range as RandRange };
use std::collections::{ BTreeMap, BTreeSet };

use cmn::{ self, CorticalDims };
use map::{ AreaMap, SliceDims, AxonKind };
// use proto::{ CellKind, Protocell, DendriteKind };


/// Information about the boundaries and synapse ranges for each source slice, on
/// each tuft.
///
/// Used to calculate a valid source axon index during synapse growth or regrowth.
pub struct SrcSlices {
	tft_slcs: Vec<BTreeMap<u8, SliceInfo>>,
	// rng: XorShiftRng,
}

impl SrcSlices {
	pub fn new(src_slc_ids_by_tft: &Vec<Vec<u8>>, syn_reaches_by_tft: Vec<u8>, area_map: &AreaMap
			) -> SrcSlices 
	{
		let mut tft_slcs = Vec::with_capacity(src_slc_ids_by_tft.len());

		let mut tft_id = 0;

		for slc_ids in src_slc_ids_by_tft.iter() {
			let mut slcs = BTreeMap::new();
			let syn_reach = *syn_reaches_by_tft.get(tft_id).expect("SrcSlices::new(): {{1}}");

			for &slc_id in slc_ids {				
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

		SrcSlices { tft_slcs: tft_slcs, /*rng: rand::weak_rng()*/ }
	}

	// [FIXME]: TODO: Bounds check.
	pub fn gen_offs(&self, tft_id: usize, slc_id: u8, rng: &mut XorShiftRng) -> (i8, i8) {
		let slc_info = &self.tft_slcs[tft_id][&slc_id];

		match slc_info.slc_off_pool {
			SlcOfsPool::Horizontal((ref v_rr, ref u_rr)) => {
				(v_rr.ind_sample(rng), u_rr.ind_sample(rng))
			},

			SlcOfsPool::Spatial((ref offs, ref range)) => {
				// let vec_range = RandRange::new(0, offs.len());
				offs[range.ind_sample(rng)]
			},
		}
	}
}



pub struct SliceInfo {
	slc_off_pool: SlcOfsPool,
	v_size: u32,
	u_size: u32,
}

impl SliceInfo {
	pub fn new(axn_kind: &AxonKind, slc_dims: &SliceDims, syn_reach: u8) -> SliceInfo {
		match axn_kind {
			&AxonKind::Horizontal => {
				// Already checked within SliceDims.
				debug_assert!(slc_dims.v_size() <= 252);
				debug_assert!(slc_dims.u_size() <= 252);

				let v_reach = (slc_dims.v_size() / 2) as i8;
				let u_reach = (slc_dims.u_size() / 2) as i8;

				SliceInfo { 
					slc_off_pool: SlcOfsPool::Horizontal((
						RandRange::new(0 - v_reach, v_reach + 1),
						RandRange::new(0 - u_reach, u_reach + 1), )),
					v_size: slc_dims.v_size(),
					u_size: slc_dims.u_size(),
				}
			},

			&AxonKind::Spatial | &AxonKind::None => {
				let hex_tile_offs = cmn::hex_tile_offs(syn_reach);
				let len = hex_tile_offs.len();

				// println!("\nhex_tile_offs: {:?}\n", hex_tile_offs);

				SliceInfo { 
					slc_off_pool: SlcOfsPool::Spatial((
						hex_tile_offs, 
						RandRange::new(0, len), )),
					v_size: slc_dims.v_size(),
					u_size: slc_dims.u_size(),
				}
			},
		}
	}

	pub fn slc_off_pool(&self) -> &SlcOfsPool {
		&self.slc_off_pool
	}
}



pub enum SlcOfsPool {
	Horizontal((RandRange<i8>, RandRange<i8>)),
	Spatial((Vec<(i8, i8)>, RandRange<usize>)),
}

// impl fmt::Debug for SlcOfsPool {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         // f.write_fmt(format_args!());
//         match self {
//         	&SlcOfsPool::Horizontal(_) => write!(f, "SlcOfsPool::Horizontal"),
//         	&SlcOfsPool::Spatial(_) => write!(f, "SlcOfsPool::Spatial"),
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

pub struct AxnOfs {
	pub slc: u8,
	pub v_ofs: i8,
	pub u_ofs: i8,
}

