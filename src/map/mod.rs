

pub use self::area_map::{ AreaMap, InterAreaInfoCache };
pub use self::slice_map::{ SliceMap };
pub use self::layer_map:: { LayerMap, LayerSourceAreas, SourceAreaInfo };
pub use self::slice_dims::{ SliceDims };

#[cfg(test)]
pub use self::area_map::tests::{ AreaMapTest };


pub mod area_map;
mod layer_map;
mod slice_map;
mod slice_dims;

// [FIXME] TODO: Potentially separate layer concerns from axon 'flavor' into a new bitflag (AxonFlags?).
// Currently: bits 48 - 63 pertain to axon 'flavor' and are relevant to inter-area axons, 32 - 47 pertain to layer properties relevant to interlaminar axons and cells. 0 - 31 are reserved for unique ids (particularly non-specific group ids).
bitflags! {
	flags LayerFlags: u64 {
		const DEFAULT				= 0b0000_0000_0000_0000__0000_0000_0000_0000 << 32,
		const INPUT					= 0b0000_0000_0000_0001__0000_0000_0000_0000 << 32,
		const OUTPUT 				= 0b0000_0000_0000_0010__0000_0000_0000_0000 << 32,
		const SPATIAL 				= 0b0000_0000_0000_0100__0000_0000_0000_0000 << 32,
		const NONSPATIAL 			= 0b0000_0000_0000_1000__0000_0000_0000_0000 << 32,
		const FEEDFORWARD			= 0b0000_0000_0001_0000__0000_0000_0000_0000 << 32,
		const FEEDBACK 				= 0b0000_0000_0010_0000__0000_0000_0000_0000 << 32,	
		const SPECIFIC 				= 0b0000_0000_0100_0000__0000_0000_0000_0000 << 32,	
		const NONSPECIFIC			= 0b0000_0000_1000_0000__0000_0000_0000_0000 << 32,	

		const SPATIAL_ASSOCIATIVE 	= 0b0000_0000_0000_0000__0000_0000_0001_0000 << 32,
		const TEMPORAL_ASSOCIATIVE 	= 0b0000_0000_0000_0000__0000_0000_0010_0000 << 32,		
		const UNUSED_TESTING		= 0b0000_0000_0000_0000__1000_0000_0000_0000 << 32,

		const FF_IN			= FEEDFORWARD.bits | INPUT.bits | SPATIAL.bits,
		const FF_OUT		= FEEDFORWARD.bits | OUTPUT.bits | SPATIAL.bits,
		const FB_IN			= FEEDBACK.bits | INPUT.bits | SPATIAL.bits,
		const FB_OUT		= FEEDBACK.bits | OUTPUT.bits | SPATIAL.bits,
	}
}

impl LayerFlags {
	// [FIXME]: Consider: Return result instead of asserts?
	pub fn mirror_io(&self) -> LayerFlags {
		debug_assert!(!(self.contains(INPUT) && self.contains(OUTPUT)),
			"LayerFlags::mirror_io(): LayerFlags input / output cannot be flipped if the bitfield \
			contains both input and output flags active. [bits: '{:?}']", self);

		debug_assert!(self.bits & !(INPUT.bits & OUTPUT.bits) == self.bits,
			"LayerFlags::mirror_io(): LayerFlags input / output cannot be flipped if the bitfield \
			contains neither input nor output flags. [bits: '{:?}']", self);

		let bits = if self.contains(INPUT) {
			(self.bits & !INPUT.bits) | OUTPUT.bits
		} else if self.contains(OUTPUT) {
			(self.bits & !OUTPUT.bits) | INPUT.bits
		} else {
			panic!("LayerFlags::mirror_io(): Internal consistency error.");
		};

		LayerFlags { bits: bits }
	}
}


#[cfg(test)]
mod tests {
	#[test]
	fn test_layer_flags() {
		// let flags = ;
		assert!(super::INPUT.mirror_io() == super::OUTPUT);
		assert!(super::OUTPUT.mirror_io() == super::INPUT);
	}
}
