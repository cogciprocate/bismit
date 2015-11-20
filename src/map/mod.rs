

pub use self::area_map::{ AreaMap };
pub use self::slice_map::{ SliceMap };
pub use self::layer_map:: { InterAreaInfoCache };

#[cfg(test)]
pub use self::area_map::tests::{ AreaMapTest };


pub mod area_map;
mod layer_map;
mod slice_map;


// TODO: Potentially separate layer concerns from input topology type into a new bitflag.
bitflags! {
	// #[derive(Debug)]
	flags LayerFlags: u64 {
		const DEFAULT				= 0b0000_0000_0000_0000__0000_0000_0000_0000 << 32,
		const INPUT					= 0b0000_0000_0000_0001__0000_0000_0000_0000 << 32,
		const OUTPUT 				= 0b0000_0000_0000_0010__0000_0000_0000_0000 << 32,
		const SPATIAL 				= 0b0000_0000_0000_0100__0000_0000_0000_0000 << 32,
		const NONSPATIAL 			= 0b0000_0000_0000_1000__0000_0000_0000_0000 << 32,
		const AFFERENT				= 0b0000_0000_0001_0000__0000_0000_0000_0000 << 32,
		const EFFERENT 				= 0b0000_0000_0010_0000__0000_0000_0000_0000 << 32,		

		const SPATIAL_ASSOCIATIVE 	= 0b0000_0000_0000_0000__0000_0000_0001_0000 << 32,
		const TEMPORAL_ASSOCIATIVE 	= 0b0000_0000_0000_0000__0000_0000_0010_0000 << 32,		
		const UNUSED_TESTING		= 0b0000_0000_0000_0000__0000_0000_0100_0000 << 32,

		const AFFERENT_INPUT		= AFFERENT.bits | INPUT.bits,
		const AFFERENT_OUTPUT		= AFFERENT.bits | OUTPUT.bits,
		const EFFERENT_INPUT		= EFFERENT.bits | INPUT.bits,
		const EFFERENT_OUTPUT		= EFFERENT.bits | OUTPUT.bits,
		//const INTERAREA				= 0b01000000,
	}
}
