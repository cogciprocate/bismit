use std::collections::BTreeSet;


pub const L5CC:         AxonTag = AxonTag(21 << 16);
pub const L5CS:         AxonTag = AxonTag(22 << 16);
pub const L5CC_NS:      AxonTag = AxonTag(23 << 16);
pub const L2:           AxonTag = AxonTag(24 << 16);
pub const L3:           AxonTag = AxonTag(25 << 16);
pub const L4:           AxonTag = AxonTag(26 << 16);
pub const THAL_SP:      AxonTag = AxonTag(27 << 16);
pub const THAL_NSP:     AxonTag = AxonTag(28 << 16);
pub const THAL_ASC:     AxonTag = AxonTag(29 << 16);


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AxonTag(u64);

impl AxonTag {
    pub fn custom(id: u16) -> AxonTag { AxonTag(id as u64) }
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