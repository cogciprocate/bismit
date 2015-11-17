

pub use self::area_map::{ AreaMap };
pub use self::slice_map::{ SliceMap };
pub use self::layer_map:: { InterAreaInfoCache };

#[cfg(test)]
pub use self::area_map::tests::{ AreaMapTest };


pub mod area_map;
mod layer_map;
mod slice_map;


bitflags! {
	// #[derive(Debug)]
	flags LayerFlags: usize {
		const DEFAULT				= 0b0000_0000__0000_0000_0000_0000,
		const INPUT					= 0b0000_0011__0000_0000_0000_0000,
		const OUTPUT 				= 0b0000_1100__0000_0000_0000_0000,

		const AFFERENT_INPUT		= 0b0000_0011__0000_0000_0000_0001,
		const AFFERENT_OUTPUT		= 0b0000_1100__0000_0000_0000_0010,
		const EFFERENT_INPUT		= 0b0000_0011__0000_0000_0000_0100,
		const EFFERENT_OUTPUT		= 0b0000_1100__0000_0000_0000_1000,

		const SPATIAL_ASSOCIATIVE 	= 0b0000_0000__0000_0000_0001_0000,
		const TEMPORAL_ASSOCIATIVE 	= 0b0000_0000__0000_0000_0010_0000,		
		const UNUSED_TESTING		= 0b0000_0000__0000_0000_0100_0000,
		//const INTERAREA				= 0b01000000,
	}
}
