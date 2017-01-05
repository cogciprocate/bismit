//! Markers to identify axons for the purpose of filtering.
//!
//!

use std::ops::BitOr;
use std::iter::FromIterator;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

const UID_IDZ: u64 = 1 << 16;
const UID_IDN: u64 = 1 << 31;
const PRESET_IDZ: u64 = 1 << 31;
// const PRESET_IDN: u64 = 1 << 32;

// MAX CONSTANT: ((1 << 32) - 1) = (PRESET_IDZ + 0x7FFF_FFFF)
// ----------------------------------------------------------
pub const THAL_SP:      AxonTag = AxonTag(PRESET_IDZ + 0x0010);
pub const THAL_NSP:     AxonTag = AxonTag(PRESET_IDZ + 0x0011);
pub const THAL_ASC:     AxonTag = AxonTag(PRESET_IDZ + 0x0012);
pub const L2:           AxonTag = AxonTag(PRESET_IDZ + 0x0200);
pub const L3:           AxonTag = AxonTag(PRESET_IDZ + 0x0300);
pub const L4:           AxonTag = AxonTag(PRESET_IDZ + 0x0400);
pub const L5CC:         AxonTag = AxonTag(PRESET_IDZ + 0x0500);
pub const L5CS:         AxonTag = AxonTag(PRESET_IDZ + 0x0501);
pub const L5CC_NS:      AxonTag = AxonTag(PRESET_IDZ + 0x0502);

// External / Subcortical:
pub const EXT:          AxonTag = AxonTag(PRESET_IDZ + 0x7000);
pub const GLY_SEQ_IMG:    AxonTag = AxonTag(EXT.0 + 0x10);
pub const GLY_SEQ_VAL:    AxonTag = AxonTag(EXT.0 + 0x11);


// static NEXT_UID: AtomicUsize = AtomicUsize::new(UID_IDZ as usize);
static NEXT_UID: AtomicUsize = ATOMIC_USIZE_INIT;


fn uid() -> u64 {
    let uid = UID_IDZ + NEXT_UID.fetch_add(1, Ordering::SeqCst) as u64;

    // Leave as an if statement (rather than assert) in case we later want to
    // return a Result or something.
    if uid >= UID_IDN {
        panic!("Error while creating AxonTag: The maximum number ({}) of unique tags has \
            already been allocated.", UID_IDN - UID_IDZ);
    }

    // unsafe { next_uid += 1; }
    uid
}

/// Markers to identify axons.
///
//
// Ranges of `AxonTag` values:
//
// idz: index[0]; idn: index[N]; idm: index[N-1];
//
// | Range (idz..idn)| Subrange (idz...idm)      | Description                           |
// | --------------- |                           | ------------------------------------- |
// | 0..(2^16)       |                           | User defined (`::custom()`)           |
// | (2^16)..(2^31)  |                           | Auto-generated (`::unique()`)         |
// | (2^31)..(2^32)  |                           | Built-in presets (constants)          |
// |       ''        | 0x0000_0000...0x0000_9FFF | Bismit constants                      |
// |       ''        | 0x0000_A000...0x0000_EFFF | External library defined constants    |
// |       ''        | 0x0000_F000...0x0000_FFFF | User defined constants                |
// |       ''        | 0x0001_0000...0x7FFF_FFFF | Reserved for future use               |
// | (2^32)..(2^64)  |                           | Reserved for future use               |
//
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AxonTag(u64);

impl AxonTag {
    pub fn custom(id: u16) -> AxonTag { AxonTag(id as u64) }

    pub fn unique() -> AxonTag {
        AxonTag(uid())
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AxonTags {
    tags: BTreeSet<AxonTag>,
}

impl AxonTags {
    pub fn new(tags: &[AxonTag]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tags.into_iter().cloned()) }
    }
}

impl<'a, 'b> BitOr<&'b AxonTags> for &'a AxonTags {
    type Output = AxonTags;

    fn bitor(self, rhs: &AxonTags) -> AxonTags {
        AxonTags{ tags: self.tags.bitor(&rhs.tags) }
    }
}

impl<'a> From<&'a [AxonTag]> for AxonTags {
    fn from(tag_list: &'a [AxonTag]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}

impl From<AxonTag> for AxonTags {
    fn from(tag: AxonTag) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter([tag].into_iter().cloned()) }
    }
}

impl<'a> From<&'a AxonTag> for AxonTags {
    fn from(tag: &'a AxonTag) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter([tag.clone()].into_iter().cloned()) }
    }
}

impl From<[AxonTag; 1]> for AxonTags {
    fn from(tag_list: [AxonTag; 1]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}

impl From<[AxonTag; 2]> for AxonTags {
    fn from(tag_list: [AxonTag; 2]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}

impl From<[AxonTag; 3]> for AxonTags {
    fn from(tag_list: [AxonTag; 3]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}

impl From<[AxonTag; 4]> for AxonTags {
    fn from(tag_list: [AxonTag; 4]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}

impl From<[AxonTag; 5]> for AxonTags {
    fn from(tag_list: [AxonTag; 5]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}

impl<'a> From<&'a [AxonTag; 1]> for AxonTags {
    fn from(tag_list: &'a [AxonTag; 1]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}

impl<'a> From<&'a [AxonTag; 2]> for AxonTags {
    fn from(tag_list: &'a [AxonTag; 2]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}
impl<'a> From<&'a [AxonTag; 3]> for AxonTags {
    fn from(tag_list: &'a [AxonTag; 3]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}
impl<'a> From<&'a [AxonTag; 4]> for AxonTags {
    fn from(tag_list: &'a [AxonTag; 4]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}
impl<'a> From<&'a [AxonTag; 5]> for AxonTags {
    fn from(tag_list: &'a [AxonTag; 5]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}