//! Markers to identify axons for the purpose of filtering.
//!
//!

use std::iter::FromIterator;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

const UID_IDZ: u64 = 1 << 16;
const UID_IDN: u64 = 1 << 31;
const PRESET_IDZ: u64 = 1 << 31;
// const PRESET_IDN: u64 = 1 << 32;

pub const THAL_SP:      AxonTag = AxonTag(PRESET_IDZ + 0x010);
pub const THAL_NSP:     AxonTag = AxonTag(PRESET_IDZ + 0x011);
pub const THAL_ASC:     AxonTag = AxonTag(PRESET_IDZ + 0x012);
pub const L2:           AxonTag = AxonTag(PRESET_IDZ + 0x200);
pub const L3:           AxonTag = AxonTag(PRESET_IDZ + 0x300);
pub const L4:           AxonTag = AxonTag(PRESET_IDZ + 0x400);
pub const L5CC:         AxonTag = AxonTag(PRESET_IDZ + 0x500);
pub const L5CS:         AxonTag = AxonTag(PRESET_IDZ + 0x501);
pub const L5CC_NS:      AxonTag = AxonTag(PRESET_IDZ + 0x502);

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
// | Range (idz..idn)| Description                           |
// | --------------- | ------------------------------------- |
// | 0..(2^16)       | User defined (`::custom()`)           |
// | (2^16)..(2^31)  | Auto-generated (`::unique()`)         |
// | (2^31)..(2^32)  | Built-in presets (constants)          |
// | (2^32)..(2^64)  | Reserved for future use               |
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
    pub fn new() -> AxonTags {
        AxonTags { tags: BTreeSet::new(), }
    }
}

impl<'a> From<&'a [AxonTag]> for AxonTags {
    fn from(tag_list: &'a [AxonTag]) -> AxonTags {
        AxonTags { tags: BTreeSet::from_iter(tag_list.into_iter().cloned()) }
    }
}