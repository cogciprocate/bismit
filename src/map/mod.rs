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
mod execution;

use std::fmt;
pub use self::area_map::AreaMap;
pub use self::slice_map::SliceMap;
pub use self::layer_map::LayerMap;
pub use self::layer_info::{LayerInfo, SourceLayerInfo};
pub use self::syn_src::{SynSrcSlices, SynSrcIdxCache, SynSrc, gen_syn_offs};
pub use self::slice_tract_map::SliceTractMap;
pub use self::scheme::{LayerMapScheme, LayerMapSchemeList, AreaScheme, AreaSchemeList,
    TuftSourceLayer, TuftSourceLayerDefinition, TuftScheme, TuftSchemeDefinition, CellScheme,
    CellSchemeDefinition, LayerScheme, LayerSchemeDefinition, FilterScheme, EncoderScheme, LayerKind};
pub use self::layer_tags::LayerTags;

/////// FIXME: IMPORT MANUALLY:
// pub use self::execution::{ExecutionGraphError, ExecutionCommandKind, ExecutionGraph};
pub use self::execution::*;
///////

/////// FIXME: IMPORT MANUALLY:
pub use self::axon_tags::*;
///////

#[cfg(test)] pub use self::area_map::tests::{AreaMapTest};



/// An absolute location and unique identifier of a layer.
//
// * TODO: Add a 'system' or 'node' id to represent the machine within a network.
//
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LayerAddress {
    area_id: usize,
    layer_id: usize,
}

impl LayerAddress {
    pub fn new(area_id: usize, layer_id: usize) -> LayerAddress {
        LayerAddress { area_id: area_id, layer_id: layer_id }
    }

    #[inline] pub fn area_id(&self) -> usize { self.area_id }
    #[inline] pub fn layer_id(&self) -> usize { self.layer_id }
}

impl fmt::Display for LayerAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


// impl From<(usize, usize)> for LayerAddress {
//     fn from(tup: (usize, usize)) -> LayerAddress {
//         LayerAddress::new(tup.0, tup.1)
//     }
// }


// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub enum ControlCellKind {
//     InhibitoryBasketSurround { tar_lyr_name: String, field_radius: u8  },
//     ActivitySmoother { tar_lyr_name: String, field_radius: u8 },
//     //AspinyStellate,
// }


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DataCellKind {
    Pyramidal,
    SpinyStellate,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ControlCellKind {
    InhibitoryBasketSurround { host_lyr_name: String, field_radius: u8  },
    ActivitySmoother { host_lyr_name: String, field_radius: u8 },
    PyrOutputter { host_lyr_name: String },
    IntraColumnInhib { host_lyr_name: String },
    Complex,
}

impl ControlCellKind {
    pub fn field_radius(&self) -> u8 {
        match *self {
            ControlCellKind::InhibitoryBasketSurround { field_radius, .. } => field_radius,
            ControlCellKind::ActivitySmoother { field_radius, .. } => field_radius,
            _ => panic!("ControlCellKind::field_radius: This control cell kind has no field radius."),
        }
    }
}

/// Roughly whether or not a cell is excitatory or inhibitory.
//
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CellClass {
    /// Cells that directly contribute to the stream of information.
    Data(DataCellKind),
    // Data { kind: ControlCellKind, exe_order: usize }
    /// Cells that indirectly contribute to the stream of information.
    Control { kind: ControlCellKind, exe_order: usize },
}

impl CellClass {
    pub fn control_kind(&self) -> ControlCellKind {
        match *self {
            CellClass::Control { ref kind, exe_order: _ } => kind.clone(),
            _ => panic!("CellClass::control_kind: Not a control cell."),
        }
    }
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum DendriteKind {
    Proximal,
    Distal,
    Other,
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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


/// [NOTE]: Axon topology is largely ignored for output layers but is
/// currently stored within `SourceLayerInfo` anyway. * TODO: Figure out what
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
    Input(Vec<AxonSignature>),
    Output(AxonSignature),
    Local,
}

impl AxonDomain {
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
