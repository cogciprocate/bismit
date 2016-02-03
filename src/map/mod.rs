

pub use self::area_map::AreaMap;
pub use self::slice_map::SliceMap;
pub use self::layer_map::LayerMap;
pub use self::layer_info::{LayerInfo, SourceLayerInfo};
pub use self::syn_src_map::{SrcSlices, SrcIdxCache, SynSrc};
pub use self::ganglion_map::GanglionMap;

pub use self::proto::AxonKind;

#[cfg(test)]
pub use self::area_map::tests::{ AreaMapTest };


pub mod area_map;
pub mod proto;
mod layer_map;
mod layer_info;
mod slice_map;
mod syn_src_map;
mod ganglion_map;

// [FIXME] TODO: Potentially separate layer concerns from axon 'flavor' into a new bitflag (AxonFlags?).
// Currently: bits 48 - 63 pertain to axon 'flavor' and are relevant to inter-area axons, 32 - 47 pertain to layer properties relevant to interlaminar axons and cells. 0 - 31 are reserved for unique ids (particularly non-specific group ids).
bitflags! {
	flags LayerTags: u64 {
		const DEFAULT				= 0b0000_0000_0000_0000__0000_0000_0000_0000 << 32,
		const INPUT					= 0b0000_0000_0000_0001__0000_0000_0000_0000 << 32,
		const OUTPUT 				= 0b0000_0000_0000_0010__0000_0000_0000_0000 << 32,
		const SPATIAL 				= 0b0000_0000_0000_0100__0000_0000_0000_0000 << 32,
		const HORIZONTAL 			= 0b0000_0000_0000_1000__0000_0000_0000_0000 << 32,
		const FEEDFORWARD			= 0b0000_0000_0001_0000__0000_0000_0000_0000 << 32,
		const FEEDBACK 				= 0b0000_0000_0010_0000__0000_0000_0000_0000 << 32,	
		const SPECIFIC 				= 0b0000_0000_0100_0000__0000_0000_0000_0000 << 32,	
		const NONSPECIFIC			= 0b0000_0000_1000_0000__0000_0000_0000_0000 << 32,	

		const PRIMARY				= 0b0000_0000_0000_0000__0000_0000_0000_0001 << 32,
		const SPATIAL_ASSOCIATIVE 	= 0b0000_0000_0000_0000__0000_0000_0001_0000 << 32,
		const TEMPORAL_ASSOCIATIVE 	= 0b0000_0000_0000_0000__0000_0000_0010_0000 << 32,
		const UNUSED_TESTING		= 0b0000_0000_0000_0000__1000_0000_0000_0000 << 32,

		const FF_IN			= FEEDFORWARD.bits | INPUT.bits | SPATIAL.bits | SPECIFIC.bits,
		const FF_OUT		= FEEDFORWARD.bits | OUTPUT.bits | SPATIAL.bits | SPECIFIC.bits,
		const FB_IN			= FEEDBACK.bits | INPUT.bits | SPATIAL.bits | SPECIFIC.bits,
		const FB_OUT		= FEEDBACK.bits | OUTPUT.bits | SPATIAL.bits | SPECIFIC.bits,
		const FF_FB_OUT		= FEEDBACK.bits | FEEDFORWARD.bits | OUTPUT.bits | SPATIAL.bits | SPECIFIC.bits,

		const NS_IN			= INPUT.bits | HORIZONTAL.bits | NONSPECIFIC.bits,
		const NS_OUT		= OUTPUT.bits | HORIZONTAL.bits | NONSPECIFIC.bits,

		const PSAL			= PRIMARY.bits | SPATIAL_ASSOCIATIVE.bits, 
		const PTAL 			= PRIMARY.bits | TEMPORAL_ASSOCIATIVE.bits,
	}
}

impl LayerTags {
	// [FIXME]: Consider: Return result instead of asserts?
	pub fn with_uid(uid: u32) -> LayerTags {
		LayerTags { bits: uid as u64 }
	}

	pub fn mirror_io(&self) -> LayerTags {
		// debug_assert!(!(self.contains(INPUT) && self.contains(OUTPUT)),
		// 	"LayerTags::mirror_io(): LayerTags input / output cannot be flipped if the bitfield \
		// 	contains both input and output tags active. [bits: '{:?}']", self);

		// debug_assert!(self.bits & !(INPUT.bits & OUTPUT.bits) == self.bits,
		// 	"LayerTags::mirror_io(): LayerTags input / output cannot be flipped if the bitfield \
		// 	contains neither input nor output tags. [bits: '{:?}']", self);

		debug_assert!(self.contains(INPUT) != self.contains(OUTPUT),
			"LayerTags::mirror_io(): LayerTags must have one of either input or output active. \
			[bits: '{:?}']", self);

		let bits = if self.contains(INPUT) {
			(self.bits & !INPUT.bits) | OUTPUT.bits
		} else if self.contains(OUTPUT) {
			(self.bits & !OUTPUT.bits) | INPUT.bits
		} else {
			panic!("LayerTags::mirror_io(): Internal consistency error.");
		};

		LayerTags { bits: bits }
	}

	#[inline]
	pub fn uid(&self) -> u32 {
		(self.bits & 0xFFFFFFFF) as u32
	}

	/// Returns whether or not `self` both contains and equals the unique id of 
	/// `other`.
	///
	/// Useful when comparing tags which must match uids where `self` may be a 
	/// superset of `other`.
	#[inline]
	pub fn meshes(&self, other: LayerTags) -> bool {
		self.contains(other) && self.uid() == other.uid()
	}

	// Presently called from Protolayer::new().
	pub fn debug_validate(&self) {
		debug_assert!(!(self.contains(OUTPUT) && self.contains(INPUT)));
		debug_assert!((self.contains(FEEDBACK) || self.contains(FEEDFORWARD)) 
			== self.contains(SPECIFIC));
		debug_assert!(self.contains(NONSPECIFIC) == (self.uid() != 0));
		debug_assert!((self.contains(INPUT) || self.contains(OUTPUT))
			== (self.contains(SPECIFIC) || self.contains(NONSPECIFIC)));
	}
}


#[cfg(test)]
mod tests {
	#[test]
	fn test_layer_tags() {
		assert!(super::INPUT.mirror_io() == super::OUTPUT);
		assert!(super::OUTPUT.mirror_io() == super::INPUT);
	}
}
