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
pub use self::layer_tags::{LayerTags, DEFAULT, PRIMARY, SPATIAL_ASSOCIATIVE, TEMPORAL_ASSOCIATIVE,
    UNUSED_TESTING, PSAL, PTAL, PMEL};
// FIXME: IMPORT MANUALLY:
pub use self::axon_tags::*;
#[cfg(test)] pub use self::area_map::tests::{AreaMapTest};


/// An absolute location and unique identifier of a layer.
//
// [TODO]: Add a 'system' or 'node' id to represent the machine within a network.
//
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

impl From<(usize, usize)> for LayerAddress {
    fn from(tup: (usize, usize)) -> LayerAddress {
        LayerAddress::new(tup.0, tup.1)
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InhibitoryCellKind {
    BasketSurround { lyr_name: String, field_radius: u8  },
    //AspinyStellate,
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CellKind {
    Pyramidal,
    SpinyStellate,
    Inhibitory(InhibitoryCellKind),
    Complex,
}


// Roughly whether or not a cell is excitatory or inhibitory.
//
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DendriteClass {
    Apical,
    Basal,
    Other,
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LayerMapKind {
    // Associational,
    // Sensory,
    // Motor,
    Cortical,
    Subcortical,
}


// [TODO]: Figure out whether or not to keep `AxonTopology` here since only
// input layer topology matters and since cellular layers are assigned
// `AxonTopology::Spatial`.
//
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LayerKind {
    Cellular(CellScheme),
    Axonal(AxonTopology),
}

impl LayerKind {
    pub fn axn_topology(&self) -> AxonTopology {
        match *self {
            LayerKind::Axonal(ak) => ak.clone(),
            LayerKind::Cellular(_) => AxonTopology::Spatial,
        }
    }

    pub fn apical<'a>(mut self, src_lyrs: &[(&'a str, i8, u8)], dens_per_tft_l2: u8,
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


/// [NOTE]: Axon topology is largely ignored for output layers but is
/// currently stored within `SourceLayerInfo` anyway. [TODO]: Figure out what
/// to do with `LayerKind::Axonal(_)` in the output case.
///
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub enum AxonTopology {
    Spatial,
    Horizontal,
    None,
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum InputTrack {
    Afferent,
    Efferent,
    Other,
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AxonDomainRoute {
    Input,
    Output,
    Local,
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AxonSignature {
    track: Option<InputTrack>,
    tags: AxonTags,
}

impl AxonSignature {
    pub fn new<A: Into<AxonTags>>(track: Option<InputTrack>, tags: A) -> AxonSignature {
        AxonSignature { track: track, tags: tags.into() }
    }

    pub fn is_output(&self) -> bool {
        self.track.is_none()
    }

    pub fn is_input(&self) -> bool {
        self.track.is_some()
    }

    #[inline] pub fn track(&self) -> Option<&InputTrack> { self.track.as_ref() }
    #[inline] pub fn tags(&self) -> &AxonTags { &self.tags }
}

impl<'a, A> From<(&'a InputTrack, A)> for AxonSignature where A: Into<AxonTags> {
    fn from(tup: (&InputTrack, A)) -> AxonSignature {
        AxonSignature::new(Some(tup.0.clone()), tup.1)
    }
}

impl<A> From<(Option<InputTrack>, A)> for AxonSignature where A: Into<AxonTags> {
    fn from(tup: (Option<InputTrack>, A)) -> AxonSignature {
        AxonSignature::new(tup.0, tup.1)
    }
}

impl<A> From<(InputTrack, A)> for AxonSignature where A: Into<AxonTags> {
    fn from(tup: (InputTrack, A)) -> AxonSignature {
        AxonSignature::new(Some(tup.0), tup.1)
    }
}

impl<A> From<A> for AxonSignature where A: Into<AxonTags> {
    fn from(tags: A) -> AxonSignature {
        AxonSignature::new(None, tags)
    }
}



#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AxonDomain {
    // Input(Vec<(InputTrack, AxonTags)>),
    Input(Vec<AxonSignature>),
    Output(AxonSignature),
    Local,
}

impl AxonDomain {
    // pub fn input<A: Into<AxonTags> + Clone>(sigs: &[(InputTrack, A)]) -> AxonDomain {
    //     AxonDomain::Input(sigs.into_iter()
    //         .map(|s| {
    //             // let (it, at) = s.clone();
    //             // (it, at.into())
    //             s.clone().into()

    //         })
    //         .collect())
    // }

    pub fn input<S: Into<AxonSignature> + Clone>(sigs: &[S]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }

    pub fn output<S: Into<AxonSignature>>(sig: S) -> AxonDomain {
        AxonDomain::Output(sig.into())
    }

    pub fn is_input(&self) -> bool {
        match *self {
            AxonDomain::Input(ref sigs) => {
                for sig in sigs { debug_assert!(sig.is_input()); }
                true
            },
            _ => false,
        }
    }

    pub fn is_output(&self) -> bool {
        match *self {
            AxonDomain::Output(ref sig) => {
                debug_assert!(sig.is_output());
                true
            },
            _ => false,
        }
    }

    pub fn route(&self) -> AxonDomainRoute {
        match *self {
            AxonDomain::Input(_) => AxonDomainRoute::Input,
            AxonDomain::Output(_) => AxonDomainRoute::Output,
            AxonDomain::Local => AxonDomainRoute::Local,
        }
    }
}

impl<'a, S> From<&'a [S]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: &'a [S]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<&'a [S; 1]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: &'a [S; 1]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<&'a [S; 2]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: &'a [S; 2]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<&'a [S; 3]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: &'a [S; 3]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<&'a [S; 4]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: &'a [S; 4]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<&'a [S; 5]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: &'a [S; 5]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<[S; 1]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: [S; 1]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<[S; 2]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: [S; 2]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<[S; 3]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: [S; 3]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<[S; 4]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: [S; 4]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}

impl<'a, S> From<[S; 5]> for AxonDomain where S: Into<AxonSignature> + Clone {
    fn from(sigs: [S; 5]) -> AxonDomain {
        AxonDomain::Input(sigs.into_iter().map(|s| s.clone().into()).collect())
    }
}


// impl<'a, A> From<&'a [(InputTrack, A)]> for AxonDomain where A: Into<AxonTags> + Clone {
//     fn from(sigs: &'a [(InputTrack, A)]) -> AxonDomain {
//         AxonDomain::input(sigs)
//     }
// }

// impl<A> From<A> for AxonDomain where A: Into<AxonTags> {
//     fn from(tags: A) -> AxonDomain {
//         AxonDomain::output(tags)
//     }
// }



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

