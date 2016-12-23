//! Mapping

mod layer_map;
mod layer_info;
mod slice_map;
mod syn_src;
mod slice_tract_map;
mod layer_tags;
mod area_map;
mod scheme;
mod axon_tags;

// use cmn::CmnError;

pub use self::area_map::AreaMap;
pub use self::slice_map::SliceMap;
pub use self::layer_map::LayerMap;
pub use self::layer_info::{LayerInfo, SourceLayerInfo};
pub use self::syn_src::{SynSrcSlices, SynSrcIdxCache, SynSrc, gen_syn_offs};
pub use self::slice_tract_map::SliceTractMap;
pub use self::scheme::{LayerMapScheme, LayerMapSchemeList, AreaScheme, AreaSchemeList, CellScheme,
	LayerScheme, FilterScheme, InputScheme, TuftSourceLayer, TuftScheme};
pub use self::layer_tags::{LayerTags, DEFAULT, INPUT, OUTPUT, /*SPATIAL,*/ /*HORIZONTAL,*/ FEEDFORWARD,
    FEEDBACK, SPECIFIC, NONSPECIFIC, PRIMARY, SPATIAL_ASSOCIATIVE, TEMPORAL_ASSOCIATIVE,
    UNUSED_TESTING, FF_IN, FF_OUT, FB_IN, FB_OUT, FF_FB_OUT, NS_IN, NS_OUT, PSAL, PTAL, PMEL};
// FIXME: IMPORT MANUALLY:
pub use self::axon_tags::*;
#[cfg(test)] pub use self::area_map::tests::{AreaMapTest};



/// An absolute location and unique identifier of a layer.
//
// [TODO]: Add a 'system' or 'node' id to represent the machine within a network.
//
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct LayerAddress {
    layer_id: usize,
    area_id: usize,
}

impl LayerAddress {
    pub fn new(layer_id: usize, area_id: usize) -> LayerAddress {
        LayerAddress { layer_id: layer_id, area_id: area_id }
    }

    #[inline] pub fn layer_id(&self) -> usize { self.layer_id }
    #[inline] pub fn area_id(&self) -> usize { self.area_id }
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum InhibitoryCellKind {
    BasketSurround { lyr_name: String, field_radius: u8  },
    //AspinyStellate,
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellKind {
    Pyramidal,
    SpinyStellate,
    Inhibitory(InhibitoryCellKind),
    Complex,
}


// Roughly whether or not a cell is excitatory or inhibitory.
//
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellClass {
    Data,
    Control,
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DendriteKind {
    Proximal,
    Distal,
    Other,
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum DendriteClass {
    Apical,
    Basal,
    Other,
}


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum LayerMapKind {
    // Associational,
    // Sensory,
    // Motor,
    Cortical,
    Subcortical,
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum LayerKind {
    Cellular(CellScheme),
    Axonal(AxonTopology),
}

impl LayerKind {
    pub fn axn_kind(&self) -> Option<AxonTopology> {
        match self {
            &LayerKind::Axonal(ak) => Some(ak.clone()),
            _ => None,
        }
    }

    pub fn apical<'a>(mut self, src_lyrs: &[(&'a str, i8)], dens_per_tft_l2: u8,
                syns_per_den_l2: u8, thresh_init: u32) -> LayerKind
    {
        match &mut self {
            &mut LayerKind::Cellular(ref mut cs) => {
                // match cs.den_dst_src_lyrs {
                //     Some(ref mut ddsl) => ddsl.push(dst_srcs),
                //     None => (),
                // }
                // cs.den_dst_syn_reaches.push(syn_reach);
                let src_lyrs_vec = src_lyrs.into_iter().map(|&sl| sl.into()).collect();
                // let tft_id = cs.tft_schemes().len();

                let tft_scheme = TuftScheme::new(DendriteClass::Apical, DendriteKind::Distal,
                    dens_per_tft_l2, syns_per_den_l2, src_lyrs_vec, Some(thresh_init));

                cs.add_tft(tft_scheme);
            },

            &mut LayerKind::Axonal(_) => panic!("::apical(): Axonal layers do not have dendrites."),
        }

        self
    }
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum InputTrack {
    Afferent,
    Efferent,
    Other,
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum AxonDomain {
    Input(Vec<(InputTrack, AxonTags)>),
    Output(AxonTags),
    Local,
}

impl AxonDomain {
    pub fn input(slice: &[(InputTrack, &[AxonTag])]) -> AxonDomain {
        AxonDomain::Input(slice.into_iter()
            .map(|&(ref it, ats)| (it.clone(), ats.into()))
            .collect())
    }

    pub fn output(ats: &[AxonTag]) -> AxonDomain {
        AxonDomain::Output(ats.into())
    }
}

/// [NOTE]: This enum is redundantly represented as a bitflag in `LayerTags`
/// and may eventually be removed pending evaluation. [UPDATE]: Nevermind:
/// layer tags representing this removed.
///
#[derive(PartialEq, Debug, Clone, Eq, Hash, Copy)]
pub enum AxonTopology {
    Spatial,
    Horizontal,
    None,
}

// impl AxonTopology {
//     pub fn from_tags<'a>(tags: LayerTags) -> Result<AxonTopology, CmnError> {
//         if tags.contains(SPATIAL) && tags.contains(HORIZONTAL) {
//             Err(CmnError::new(format!("Error converting tags to AxonTopology, tags must contain \
//                 only one of either 'SPATIAL' or 'HORIZONTAL'. (tags: '{:?}')", tags)))
//         } else if tags.contains(SPATIAL) {
//             Ok(AxonTopology::Spatial)
//         } else if tags.contains(HORIZONTAL) {
//             Ok(AxonTopology::Horizontal)
//         } else {
//             // Err(CmnError::new(format!("Unable to determine axon kind from tags: '{:?}'", tags)))
//             Ok(AxonTopology::None)
//         }
//     }

//     pub fn matches_tags(&self, tags: LayerTags) -> bool {
//         match self {
//             &AxonTopology::Spatial => tags.contains(SPATIAL),
//             &AxonTopology::Horizontal => tags.contains(HORIZONTAL),
//             &AxonTopology::None => (!tags.contains(SPATIAL)) && (!tags.contains(HORIZONTAL)),
//         }
//     }
// }

