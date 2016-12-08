
use std::iter::FromIterator;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

static NEXT_UID: AtomicUsize = ATOMIC_USIZE_INIT;
// static mut next_uid: u16 = 0;

pub const L5CC:         AxonTag = AxonTag(21 << 16);
pub const L5CS:         AxonTag = AxonTag(22 << 16);
pub const L5CC_NS:      AxonTag = AxonTag(23 << 16);
pub const L2:           AxonTag = AxonTag(24 << 16);
pub const L3:           AxonTag = AxonTag(25 << 16);
pub const L4:           AxonTag = AxonTag(26 << 16);
pub const THAL_SP:      AxonTag = AxonTag(27 << 16);
pub const THAL_NSP:     AxonTag = AxonTag(28 << 16);
pub const THAL_ASC:     AxonTag = AxonTag(29 << 16);


fn uid() -> u16 {
    // let uid = unsafe { next_uid };
    let uid = NEXT_UID.fetch_add(1, Ordering::SeqCst);

    if uid == u16::max_value() as usize {
        panic!("Error while creating AxonTag: The maximum number ({}) of unique tags has \
            already been allocated.", u16::max_value());
    }

    // unsafe { next_uid += 1; }
    uid as u16
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AxonTag(u64);

impl AxonTag {
    // pub fn custom(id: u16) -> AxonTag { AxonTag(id as u64) }
    pub fn unique() -> AxonTag {
        AxonTag(uid() as u64)
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