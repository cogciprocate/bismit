//! Mapping

mod layer_map;
mod layer_info;
mod slice_map;
mod syn_src_map;
mod slice_tract_map;
mod layer_tags;
mod area_map;
mod scheme;

use cmn::CmnError;

pub use self::area_map::AreaMap;
pub use self::slice_map::SliceMap;
pub use self::layer_map::LayerMap;
pub use self::layer_info::{LayerInfo, SourceLayerInfo};
pub use self::syn_src_map::{SrcSlices, SrcIdxCache, SynSrc, gen_syn_offs};
pub use self::slice_tract_map::SliceTractMap;
pub use self::scheme::{LayerMapScheme, LayerMapSchemeList, AreaScheme, AreaSchemeList, CellScheme,
	LayerScheme, FilterScheme, InputScheme};
pub use self::layer_tags::{LayerTags, DEFAULT, INPUT, OUTPUT, SPATIAL, HORIZONTAL, FEEDFORWARD,
    FEEDBACK, SPECIFIC, NONSPECIFIC, PRIMARY, SPATIAL_ASSOCIATIVE, TEMPORAL_ASSOCIATIVE,
    UNUSED_TESTING, FF_IN, FF_OUT, FB_IN, FB_OUT, FF_FB_OUT, NS_IN, NS_OUT, PSAL, PTAL};
#[cfg(test)] pub use self::area_map::tests::{AreaMapTest};



#[derive(Copy, PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellKind {
    Pyramidal,
    SpinyStellate,
    //AspinyStellate,
    Inhibitory,
    Complex,
}


#[derive(Copy, PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellClass {
    Data,
    Control,
}


#[derive(Copy, PartialEq, Debug, Clone)]
pub enum DendriteKind {
    Proximal,
    Distal,
}


#[allow(dead_code)]
pub enum DendriteClass {
    Apical,
    Distal,
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum LayerMapKind {
    // Associational,
    // Sensory,
    // Motor,
    Cortical,
    Subcortical,
}


/// [NOTE]: This enum is redundantly represented as a bitflag in `LayerTags`
/// and may eventually be removed pending evaluation.
///
#[derive(PartialEq, Debug, Clone, Eq, Hash, Copy)]
pub enum AxonKind {
    Spatial,
    Horizontal,
    None,
}

impl AxonKind {
    pub fn from_tags<'a>(tags: LayerTags) -> Result<AxonKind, CmnError> {
        if tags.contains(SPATIAL) && tags.contains(HORIZONTAL) {
            Err(CmnError::new(format!("Error converting tags to AxonKind, tags must contain \
                only one of either 'SPATIAL' or 'HORIZONTAL'. (tags: '{:?}')", tags)))
        } else if tags.contains(SPATIAL) {
            Ok(AxonKind::Spatial)
        } else if tags.contains(HORIZONTAL) {
            Ok(AxonKind::Horizontal)
        } else {
            // Err(CmnError::new(format!("Unable to determine axon kind from tags: '{:?}'", tags)))
            Ok(AxonKind::None)
        }
    }

    pub fn matches_tags(&self, tags: LayerTags) -> bool {
        match self {
            &AxonKind::Spatial => tags.contains(SPATIAL),
            &AxonKind::Horizontal => tags.contains(HORIZONTAL),
            &AxonKind::None => (!tags.contains(SPATIAL)) && (!tags.contains(HORIZONTAL)),
        }
    }
}



#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum LayerKind {
    Cellular(CellScheme),
    Axonal(AxonKind),
}

impl LayerKind {
    pub fn axn_kind(&self) -> Option<AxonKind> {
        match self {
            &LayerKind::Axonal(ak) => Some(ak.clone()),
            _ => None,
        }
    }

    pub fn apical(mut self, dst_srcs: Vec<&'static str>, syn_reach: i8) -> LayerKind {
        match &mut self {
            &mut LayerKind::Cellular(ref mut pc) => {
                match pc.den_dst_src_lyrs {
                    Some(ref mut vec) => vec.push(dst_srcs),
                    None => (),
                }

                pc.den_dst_syn_reaches.push(syn_reach);
            },

            &mut LayerKind::Axonal(_) => (),
        };

        self
    }
}

