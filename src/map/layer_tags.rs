#![allow(dead_code)]

use std::fmt::{self, Formatter};


// [TODO]: Potentially separate layer concerns from axon 'flavor' into a new
// bitflag (AxonFlags?). Currently: bits 48 - 63 pertain to axon 'flavor' and
// are relevant to inter-area axons, 32 - 47 pertain to layer properties
// relevant to interlaminar axons and cells. 0 - 31 are reserved for unique
// ids (particularly non-specific group ids).
bitflags! {
    pub flags LayerTags: u64 {
        const DEFAULT = 0b0000_0000_0000_0000__0000_0000_0000_0000 << 32,
        const INPUT = 0b0000_0000_0000_0001__0000_0000_0000_0000 << 32,
        const OUTPUT = 0b0000_0000_0000_0010__0000_0000_0000_0000 << 32,
        const SPATIAL = 0b0000_0000_0000_0100__0000_0000_0000_0000 << 32,
        const HORIZONTAL = 0b0000_0000_0000_1000__0000_0000_0000_0000 << 32,
        const FEEDFORWARD = 0b0000_0000_0001_0000__0000_0000_0000_0000 << 32,
        const FEEDBACK = 0b0000_0000_0010_0000__0000_0000_0000_0000 << 32,    
        const SPECIFIC = 0b0000_0000_0100_0000__0000_0000_0000_0000 << 32,    
        const NONSPECIFIC = 0b0000_0000_1000_0000__0000_0000_0000_0000 << 32,    

        const PRIMARY = 0b0000_0000_0000_0000__0000_0000_0000_0001 << 32,
        const SPATIAL_ASSOCIATIVE = 0b0000_0000_0000_0000__0000_0000_0001_0000 << 32,
        const TEMPORAL_ASSOCIATIVE = 0b0000_0000_0000_0000__0000_0000_0010_0000 << 32,
        const UNUSED_TESTING = 0b0000_0000_0000_0000__1000_0000_0000_0000 << 32,

        const FF_IN = FEEDFORWARD.bits | INPUT.bits | SPATIAL.bits | SPECIFIC.bits,
        const FF_OUT = FEEDFORWARD.bits | OUTPUT.bits | SPATIAL.bits | SPECIFIC.bits,
        const FB_IN = FEEDBACK.bits | INPUT.bits | SPATIAL.bits | SPECIFIC.bits,
        const FB_OUT = FEEDBACK.bits | OUTPUT.bits | SPATIAL.bits | SPECIFIC.bits,
        const FF_FB_OUT = FEEDBACK.bits | FEEDFORWARD.bits | OUTPUT.bits | SPATIAL.bits | SPECIFIC.bits,

        const NS_IN = INPUT.bits | HORIZONTAL.bits | NONSPECIFIC.bits,
        const NS_OUT = OUTPUT.bits | HORIZONTAL.bits | NONSPECIFIC.bits,

        const PSAL = PRIMARY.bits | SPATIAL_ASSOCIATIVE.bits, 
        const PTAL = PRIMARY.bits | TEMPORAL_ASSOCIATIVE.bits,
    }
}

impl LayerTags {
    pub fn uid(uid: u32) -> LayerTags {
        LayerTags { bits: uid as u64 }
    }

    pub fn mirror_io(&self) -> LayerTags {
        assert!(self.contains(INPUT) != self.contains(OUTPUT),
            "LayerTags::mirror_io(): LayerTags must have one of either input or output active. \
            [bits: '{:?}']", self);

        // println!("LAYER_TAGS::MIRROR_IO(): before: {}", self);

        let bits = if self.contains(INPUT) {
            (self.bits & !INPUT.bits) | OUTPUT.bits
        } else if self.contains(OUTPUT) {
            (self.bits & !OUTPUT.bits) | INPUT.bits
        } else {
            panic!("LayerTags::mirror_io(): Internal consistency error.");
        };

        // println!("LAYER_TAGS::MIRROR_IO(): after: {}", self);

        LayerTags { bits: bits }
    }

    pub fn get_uid(&self) -> u32 {
        (self.bits & 0xFFFFFFFF) as u32
    }

    /// Returns whether or not `self` both contains and equals the unique id of 
    /// `other`.
    ///
    /// Useful when comparing tags which must match uids where `self` may be a 
    /// superset of `other`.
    pub fn meshes(&self, other: LayerTags) -> bool {
        self.contains(other) && self.get_uid() == other.get_uid()
    }

    // Presently called from LayerScheme::new() on a debug build.
    pub fn debug_validate(&self) {
        debug_assert!(!(self.contains(OUTPUT) && self.contains(INPUT)));
        debug_assert!((self.contains(FEEDBACK) || self.contains(FEEDFORWARD)) 
            == self.contains(SPECIFIC));
        debug_assert!(self.contains(NONSPECIFIC) == (self.get_uid() != 0));
        debug_assert!((self.contains(INPUT) || self.contains(OUTPUT))
            == (self.contains(SPECIFIC) || self.contains(NONSPECIFIC)));
    }

    pub fn debug_print_compare(&self, other: LayerTags) {
        println!("self : {{ {} }}, self.get_uid() : {}, \n\
            other: {{ {} }}, other.get_uid(): {}", 
            self, self.get_uid(), other, other.get_uid());
        println!("    CONTAINS: {}, UID_MATCH: {}, MESHES: {}
            ", self.contains(other), 
            self.get_uid() == other.get_uid(),
            self.meshes(other));
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn layer_tags() {
        assert!(super::INPUT.mirror_io() == super::OUTPUT);
        assert!(super::OUTPUT.mirror_io() == super::INPUT);
    }
}

impl fmt::Display for LayerTags {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{ {:?} | uid: {} }}", self, self.get_uid())
    }
}