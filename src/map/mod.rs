//! Mapping

mod layer_map;
mod layer_info;
mod slice_map;
mod syn_src_map;
mod slice_tract_map;
mod layer_tags;
pub mod area_map;
pub mod proto;

pub use self::area_map::AreaMap;
pub use self::slice_map::SliceMap;
pub use self::layer_map::LayerMap;
pub use self::layer_info::{LayerInfo, SourceLayerInfo};
pub use self::syn_src_map::{SrcSlices, SrcIdxCache, SynSrc};
pub use self::slice_tract_map::SliceTractMap;
pub use self::proto::{ProtolayerMap, ProtolayerMaps, ProtoareaMap, ProtoareaMaps, AxonKind};
pub use self::layer_tags::{LayerTags, DEFAULT, INPUT, OUTPUT, SPATIAL, HORIZONTAL, FEEDFORWARD, 
    FEEDBACK, SPECIFIC, NONSPECIFIC, PRIMARY, SPATIAL_ASSOCIATIVE, TEMPORAL_ASSOCIATIVE, 
    UNUSED_TESTING, FF_IN, FF_OUT, FB_IN, FB_OUT, FF_FB_OUT, NS_IN, NS_OUT, PSAL, PTAL};
#[cfg(test)] pub use self::area_map::tests::{AreaMapTest};
