#![allow(dead_code)]

use std::fmt::{self, Formatter};

// # Tags
//
// (From DOI: http://dx.doi.org/10.1016/j.neuron.2015.11.002)
//
// - L5CC (L5A): Tlx3-Cre PL56 (intratelencephalic)(thin tuft)(70mum^2)(RS - soma)
//   - eGFP+ Tlx3-Cre+ neurons in V1 project their axons densely and
//     predominantly to adjacent visual cortical areas such as V2L, V2ML, and
//     V2MM, as well as further cortical regions including other sensory
//     cortices, frontal cortices, and the contralateral visual cortex
//     (Figures 1A, 1B, and S1A–S1C). This suggests that Tlx3-Cre selectively
//     labels L5 CC PNs.
//
// - L5CS (L5B): Glt25d2-Cre NF107 (pyramidal tract)(thick tuft)(124mum^2)(Intrinsicly BS - soma)
//   - In contrast, eGFP+ axons from Glt25d2-Cre+ neurons in V1 do not project
//     to other cortical areas (Figures 1A and 1B). Instead, their axons enter
//     white matter and travel to target subcortical structures including the
//     superior colliculus, lateral posterior (LP) and lateral dorsal (LD)
//     nuclei of thalamus, pons, and ipsilateral striatum (Figures 1B and
//     S1D–S1O). These axonal projections suggest Glt25d2-Cre selectively
//     labels L5 CS PNs. We also labeled L5 CS PNs in V1 by injecting
//     retrogradely infecting Cav2-Cre virus into the superior colliculus and
//     AAV-FLEX-eGFP into V1. L5 neurons labeled by Cre expression in the
//     Glt25d2- Cre mouse line and those labeled following Cav2-Cre injection
//     to superior colliculus exhibit similar cell body locations and axon
//     target profiles (Figures 1A and 1B). We conclude that although
//     Glt25d2-Cre+ neurons are sparse, they are a representative sample of CS
//     neurons.
//
// - L5CC-NS (NEW) (non-striatal): Efr3a-Cre NO108 (64mum^2)(RS - soma)
//   - Notably, Efr3a-Cre+ L5 V1 neurons lack projections to known axonal
//     targets of L5 CC and CS neurons such as superior colliculus, thalamus,
//     brainstem, and striatum (Figure 1B). Efr3a-Cre+ neurons do project to
//     other adjacent cortical areas, a target they share in common with L5 CC
//     neurons. Dense eGFP+ labeled long distance axons are also found in
//     known layer 6 neuron targets, including the dorsal lateral geniculate
//     nucleus (dLGN), and LD and LP of thalamus (Figures 1B and S1J–S1O). In
//     LP, a shared target region of L5 and 6 neurons, axon terminals of
//     Efr3a-Cre+ neurons are thin and small type I morphology, distinct from
//     the thick and large type II morphology of Glt25d2-Cre+ neurons (Figures
//     S1P and S1Q) (Li et al., 2003). The presence of labeled neurons in both
//     L5 and 6 of Efr3a-Cre mice makes it less straightforward to study the
//     projections of L5 Efr3a-Cre+ neurons in isolation. However, several
//     lines of evidence detailed below indicate that L5 Efr3aCre+ neurons do
//     not project to the thalamus and that they include both local pyramids
//     (not projecting out of V1) and projection neurons. Since the only
//     targets of Efr3a-Cre+ neurons that are known to receive input from L5
//     rather than L6 are adjacent cortical areas (see above), L5 Efr3a-Cre+
//     neurons must include CC projection neurons and might therefore
//     represent a subgroup of CC neurons. Despite the fact that both L5
//     Efr3a-Cre+ neurons and Tlx3-Cre+ CC cells share a common extrinsic
//     target (adjacent cortical areas), they are clearly distinct and
//     non-overlapping groups. In addition to differences in projections to
//     striatum, as described in further detail below, the morphology and
//     intrinsic physiology of L5 Efr3a-Cre+ neurons further distinguish them
//     from Tlx3-Cre+ CC cells
//
//


// [TODO (DONE)]: Potentially separate layer concerns from axon 'flavor' into
// a new bitflag (AxonFlags?). Currently: bits 48 - 63 pertain to axon
// 'flavor' and are relevant to inter-area axons, 32 - 47 pertain to layer
// properties relevant to interlaminar axons and cells. 0 - 31 are reserved
// for unique ids (particularly non-specific group ids).
bitflags! {
    pub flags LayerTags: u64 {
        const DEFAULT = 0b0000_0000_0000_0000__0000_0000_0000_0000 << 32,
        const INPUT = 0b0000_0000_0000_0001__0000_0000_0000_0000 << 32,
        const OUTPUT = 0b0000_0000_0000_0010__0000_0000_0000_0000 << 32,
        // const SPATIAL = 0b0000_0000_0000_0100__0000_0000_0000_0000 << 32,
        // const HORIZONTAL = 0b0000_0000_0000_1000__0000_0000_0000_0000 << 32,
        // const FEEDFORWARD = 0b0000_0000_0001_0000__0000_0000_0000_0000 << 32,
        // const FEEDBACK = 0b0000_0000_0010_0000__0000_0000_0000_0000 << 32,
        // const SPECIFIC = 0b0000_0000_0100_0000__0000_0000_0000_0000 << 32,
        // const NONSPECIFIC = 0b0000_0000_1000_0000__0000_0000_0000_0000 << 32,

        const PRIMARY = 0b0000_0000_0000_0000__0000_0000_0000_0001 << 32,
        const SPATIAL_ASSOCIATIVE = 0b0000_0000_0000_0000__0000_0000_0001_0000 << 32,
        const TEMPORAL_ASSOCIATIVE = 0b0000_0000_0000_0000__0000_0000_0010_0000 << 32,
        const MOTOR = 0b0000_0000_0000_0000__0000_0000_0100_0000 << 32,
        const UNUSED_TESTING = 0b0000_0000_0000_0000__1000_0000_0000_0000 << 32,

        // const FF_IN = FEEDFORWARD.bits | INPUT.bits | SPECIFIC.bits,
        // const FF_OUT = FEEDFORWARD.bits | OUTPUT.bits | SPECIFIC.bits,
        // const FB_IN = FEEDBACK.bits | INPUT.bits | SPECIFIC.bits,
        // const FB_OUT = FEEDBACK.bits | OUTPUT.bits | SPECIFIC.bits,
        // const FF_FB_OUT = FEEDBACK.bits | FEEDFORWARD.bits | OUTPUT.bits | SPECIFIC.bits,

        // const NS_IN = INPUT.bits | NONSPECIFIC.bits,
        // const NS_OUT = OUTPUT.bits | NONSPECIFIC.bits,

        const PSAL = PRIMARY.bits | SPATIAL_ASSOCIATIVE.bits,
        const PTAL = PRIMARY.bits | TEMPORAL_ASSOCIATIVE.bits,
        const PMEL = PRIMARY.bits | MOTOR.bits,
    }
}

impl LayerTags {
    pub fn uid(uid: u32) -> LayerTags {
        LayerTags { bits: uid as u64 }
    }

    pub fn from_u64(val: u64) -> LayerTags {
        LayerTags { bits: val }
    }

    // pub fn mirror_io(&self) -> LayerTags {
    //     assert!(self.contains(INPUT) != self.contains(OUTPUT),
    //         "LayerTags::mirror_io(): LayerTags must have one of either input or output active. \
    //         [bits: '{:?}']", self);

    //     // println!("LAYER_TAGS::MIRROR_IO(): before: {}", self);

    //     let bits = if self.contains(INPUT) {
    //         (self.bits & !INPUT.bits) | OUTPUT.bits
    //     } else if self.contains(OUTPUT) {
    //         (self.bits & !OUTPUT.bits) | INPUT.bits
    //     } else {
    //         panic!("LayerTags::mirror_io(): Internal consistency error.");
    //     };

    //     // println!("LAYER_TAGS::MIRROR_IO(): after: {}", self);

    //     LayerTags { bits: bits }
    // }

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

    /// Returns whether or not `self` both contains and equals the unique id of
    /// `other` or the other way around.
    ///
    /// Useful when comparing tags which must match uids where `self` may be a
    /// superset of `other` or `other` a superset of `self`.
    pub fn meshes_either_way(&self, other: LayerTags) -> bool {
        (self.contains(other) || other.contains(*self)) &&
            (*self != DEFAULT && other != DEFAULT) &&
            self.get_uid() == other.get_uid()
    }

    // // Presently called from LayerScheme::new() on a debug build.
    // pub fn debug_validate(&self) {
    //     debug_assert!(!(self.contains(OUTPUT) && self.contains(INPUT)));
    //     debug_assert!((self.contains(FEEDBACK) || self.contains(FEEDFORWARD))
    //         == self.contains(SPECIFIC));
    //     debug_assert!(self.contains(NONSPECIFIC) == (self.get_uid() != 0));
    //     debug_assert!((self.contains(INPUT) || self.contains(OUTPUT))
    //         == (self.contains(SPECIFIC) || self.contains(NONSPECIFIC)));
    // }

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

impl fmt::Display for LayerTags {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{{ {:?} | uid: {} }}", self, self.get_uid())
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn layer_tags() {
        // assert!(super::INPUT.mirror_io() == super::OUTPUT);
        // assert!(super::OUTPUT.mirror_io() == super::INPUT);
    }
}
