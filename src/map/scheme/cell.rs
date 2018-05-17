use map::{CellClass, LayerKind, DendriteClass, DendriteKind, DataCellKind, ControlCellKind};
use SrcOfs;
// use cmn;


/// A source layer.
///
/// `prevalence` is a simple weight or factor applied to each layer during
/// learning. If one source layer has a `prevalence` of `5` and all other
/// source layers for a tuft have a `prevalence` of `1`, the source layer with
/// the `5` will be five times more likely to form a synapse during
/// regrowth/growth.
///
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct TuftSourceLayer {
    name: String,
    syn_reach: SrcOfs,
    prevalence: u8,
}

impl TuftSourceLayer {
    pub fn define<S: Into<String>>(name: S) -> TuftSourceLayerDefinition {
        TuftSourceLayerDefinition::new(name)
    }

    pub fn new<S: Into<String>>(name: S, syn_reach: SrcOfs, prevalence: u8) -> TuftSourceLayer {
        assert!(prevalence > 0, "Tuft source layer definitions must have a prevalence \
            of at least one. {{ Layer: name: {}, reach: {} }}", name.into(), syn_reach);

        TuftSourceLayer {
            name: name.into(),
            syn_reach: syn_reach,
            prevalence: prevalence,
        }
    }

    #[inline] pub fn name(&self) -> &str { self.name.as_str() }
    #[inline] pub fn syn_reach(&self) -> SrcOfs { self.syn_reach }
    #[inline] pub fn prevalence(&self) -> u8 { self.prevalence }
}

impl<'a> From<(&'a str, SrcOfs, u8)> for TuftSourceLayer {
    fn from(tup: (&'a str, SrcOfs, u8)) -> TuftSourceLayer {
        TuftSourceLayer::new(tup.0.to_owned(), tup.1, tup.2)
    }
}

impl<'a> From<&'a (&'a str, SrcOfs, u8)> for TuftSourceLayer {
    fn from(tup: &'a (&'a str, SrcOfs, u8)) -> TuftSourceLayer {
        TuftSourceLayer::new(tup.0.to_owned(), tup.1, tup.2)
    }
}

pub struct TuftSourceLayerDefinition {
    name: String,
    syn_reach: Option<SrcOfs>,
    prevalence: Option<u8>,
}

impl TuftSourceLayerDefinition {
    pub fn new<S: Into<String>>(name: S) -> TuftSourceLayerDefinition {
        TuftSourceLayerDefinition {
            name: name.into(),
            syn_reach: None,
            // Make this default to something instead?:
            prevalence: None,
        }
    }

    // pub fn src_lyr_name<S: Into<String>>(mut self, ) -> TuftSourceLayerDefinition {
    //     self.name = Some(name.into());
    //     self
    // }

    pub fn syn_reach(mut self, syn_reach: SrcOfs) -> TuftSourceLayerDefinition {
        self.syn_reach = Some(syn_reach);
        self
    }

    // Make this optional? (see above)
    pub fn prevalence(mut self, prevalence: u8) -> TuftSourceLayerDefinition {
        self.prevalence = Some(prevalence);
        self
    }

    pub fn build(self) -> TuftSourceLayer {
        TuftSourceLayer::new(
            self.name,
            self.syn_reach.expect("TuftSourceLayerDefinition::build"),
            self.prevalence.expect("TuftSourceLayerDefinition::build"),
        )
    }
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct TuftScheme {
    tft_id: usize,
    den_class: DendriteClass,
    den_kind: DendriteKind,
    dens_per_tft: u32,
    syns_per_den: u32,
    max_active_dens_l2: u8,
    src_lyrs: Vec<TuftSourceLayer>,
    thresh_init: Option<u32>,
}

impl TuftScheme {
    pub fn define() -> TuftSchemeDefinition {
        TuftSchemeDefinition::new()
    }

    pub fn apical() -> TuftSchemeDefinition {
        TuftSchemeDefinition::new().apical()
    }

    pub fn basal() -> TuftSchemeDefinition {
        TuftSchemeDefinition::new().basal()
    }

    pub fn new(tft_id: usize, den_class: DendriteClass, den_kind: DendriteKind,
            dens_per_tft: u32, syns_per_den: u32, max_active_dens_l2: u8,
            src_lyrs: Vec<TuftSourceLayer>, thresh_init: Option<u32>) -> TuftScheme {
        TuftScheme {
            tft_id,
            den_class,
            den_kind,
            dens_per_tft,
            syns_per_den,
            max_active_dens_l2,
            src_lyrs,
            thresh_init,
        }
    }

    // pub fn with_tft_id(mut self, tft_id: usize) -> TuftScheme {
    //     self.tft_id = Some(tft_id);
    //     self
    // }

    // #[inline] pub fn tft_id(&self) -> usize { self.tft_id.expect("Tuft ID not set!") }
    #[inline] pub fn tft_id(&self) -> usize { self.tft_id }
    #[inline] pub fn den_class(&self) -> DendriteClass { self.den_class }
    #[inline] pub fn den_kind(&self) -> DendriteKind { self.den_kind }
    #[inline] pub fn dens_per_tft(&self) -> u32 { self.dens_per_tft }
    #[inline] pub fn syns_per_den(&self) -> u32 { self.syns_per_den }
    #[inline] pub fn syns_per_tft(&self) -> u32 { self.dens_per_tft * self.syns_per_den }
    #[inline] pub fn max_active_dens_l2(&self) -> u8 { self.max_active_dens_l2 }
    #[inline] pub fn src_lyrs(&self) -> &[TuftSourceLayer] { self.src_lyrs.as_slice() }
    #[inline] pub fn thresh_init(&self) -> &Option<u32> { &self.thresh_init }
}


pub struct TuftSchemeDefinition {
    den_class: Option<DendriteClass>,
    den_kind: Option<DendriteKind>,
    dens_per_tft: u32,
    syns_per_den: Option<u32>,
    max_active_dens_l2: u8,
    src_lyrs: Vec<TuftSourceLayer>,
    thresh_init: Option<u32>,
}

impl TuftSchemeDefinition {
    pub fn new() -> TuftSchemeDefinition {
        TuftSchemeDefinition {
            den_class: None,
            den_kind: None,
            dens_per_tft: 1,
            syns_per_den: None,
            max_active_dens_l2: 0,
            src_lyrs: Vec::with_capacity(4),
            thresh_init: None,
        }
    }

    pub fn den_class(mut self, den_class: DendriteClass) -> TuftSchemeDefinition {
        assert!(self.den_class.is_none());
        self.den_class = Some(den_class);
        self
    }

    pub fn apical(self) -> TuftSchemeDefinition {
        assert!(self.den_class.is_none());
        self.den_class(DendriteClass::Apical)
    }

    pub fn basal(self) -> TuftSchemeDefinition {
        assert!(self.den_class.is_none());
        self.den_class(DendriteClass::Basal)
    }

    pub fn den_kind(mut self, den_kind: DendriteKind) -> TuftSchemeDefinition {
        assert!(self.den_kind.is_none());
        self.den_kind = Some(den_kind);
        self
    }

    pub fn proximal(self) -> TuftSchemeDefinition {
        assert!(self.den_kind.is_none());
        self.den_kind(DendriteKind::Proximal)
    }

    pub fn distal(self) -> TuftSchemeDefinition {
        assert!(self.den_kind.is_none());
        self.den_kind(DendriteKind::Distal)
    }

    // If (dens_per_tft * syns_per_den) % 4 == 0, a vec4 kernel will be used.
    pub fn dens_per_tft(mut self, dens_per_tft: u32) -> TuftSchemeDefinition {
        self.dens_per_tft = dens_per_tft;
        self
    }

    // If (dens_per_tft * syns_per_den) % 4 == 0, a vec4 kernel will be used.
    pub fn syns_per_den(mut self, syns_per_den: u32) -> TuftSchemeDefinition {
        self.syns_per_den = Some(syns_per_den);
        self
    }

    /// Sets the number of active dendrites required to reach 100% activation
    /// for this tuft (in log2).
    pub fn max_active_dens_l2(mut self, max_active_dens_l2: u8) -> TuftSchemeDefinition {
        self.max_active_dens_l2 = max_active_dens_l2;
        self
    }

    // pub fn src_lyr<S: Into<String>>(mut self, name: S, syn_reach: SrcOfs, prevalence: u8)
    //         -> TuftSchemeDefinition {
    //     self.src_lyrs.push(TuftSourceLayer::new(name, syn_reach, prevalence));
    //     self
    // }

    pub fn src_lyr(mut self, bldr: TuftSourceLayerDefinition) -> TuftSchemeDefinition {
        self.src_lyrs.push(bldr.build());
        self
    }

    pub fn src_lyrs(mut self, src_lyrs: &[(&str, SrcOfs, u8)]) -> TuftSchemeDefinition {
        assert!(self.src_lyrs.is_empty());
        let src_lyrs = src_lyrs.into_iter().map(|tsl| tsl.into()).collect();
        self.src_lyrs = src_lyrs;
        self
    }

    pub fn thresh_init(mut self, thresh_init: u32) -> TuftSchemeDefinition {
        self.thresh_init = Some(thresh_init);
        self
    }

    pub fn build(self, tft_id: usize) -> TuftScheme {
        TuftScheme {
            tft_id: tft_id,
            den_class: self.den_class.expect("TuftScheme::build"),
            den_kind: self.den_kind.expect("TuftScheme::build"),
            dens_per_tft: self.dens_per_tft,
            syns_per_den: self.syns_per_den.expect("TuftScheme::build"),
            max_active_dens_l2: self.max_active_dens_l2,
            src_lyrs: self.src_lyrs,
            thresh_init: self.thresh_init,
        }
    }
}




#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct CellScheme {
    cell_class: CellClass,
    tft_schemes: Vec<TuftScheme>,
}

impl CellScheme {
    pub fn new(cell_class: CellClass, tft_schemes: Vec<TuftScheme>) -> CellScheme {
        // DO SOME CHECKS ON PARAMETERS (certain cell types must/mustn't have certain dendritic segments)
        CellScheme {
            cell_class: cell_class,
            tft_schemes: tft_schemes,
        }.validate()
    }

    pub fn define(cell_class: CellClass) -> CellSchemeDefinition {
        CellSchemeDefinition::new(cell_class)
    }

    pub fn data(kind: DataCellKind) -> CellSchemeDefinition {
        CellSchemeDefinition::new(CellClass::Data(kind))
    }

    pub fn pyramidal() -> CellSchemeDefinition {
        Self::data(DataCellKind::Pyramidal)
    }

    pub fn spiny_stellate() -> CellSchemeDefinition {
        Self::data(DataCellKind::SpinyStellate)
    }

    pub fn control(kind: ControlCellKind, exe_order: usize) -> CellSchemeDefinition {
        CellSchemeDefinition::new(CellClass::Control { kind, exe_order })
    }

    // //                             &[name, reach, prevalence]
    // pub fn pyr<'a>(dst_srcs: &[(&'a str, SrcOfs, u8)], dens_per_tft_l2: u8,
    //         syns_per_den_l2: u8, max_active_dens_l2: u8, thresh: u32) -> LayerKind {
    //     let src_lyrs_vec = dst_srcs.into_iter().map(|&sl| sl.into()).collect();

    //     let tft_scheme = TuftScheme::new(0, DendriteClass::Basal, DendriteKind::Distal,
    //         dens_per_tft_l2, syns_per_den_l2, max_active_dens_l2, src_lyrs_vec, Some(thresh));

    //     LayerKind::Cellular(CellScheme {
    //         cell_class: CellClass::Data(DataCellKind::Pyramidal),
    //         tft_schemes: vec![tft_scheme]
    //     }.validate())
    // }

    // //                                  &[name, reach, prevalence]
    // pub fn ssc<'a>(prx_srcs: &[(&'a str, SrcOfs, u8)], syns_per_den_l2: u8, thresh: u32,)
    //         -> LayerKind {
    //     let src_lyrs_vec = prx_srcs.into_iter().map(|&sl| sl.into()).collect();

    //     let tft_scheme = TuftScheme::new(0, DendriteClass::Basal, DendriteKind::Proximal, 0,
    //         syns_per_den_l2, 0, src_lyrs_vec, Some(thresh));

    //     LayerKind::Cellular(CellScheme {
    //         cell_class: CellClass::Data(DataCellKind::SpinyStellate),
    //         tft_schemes: vec![tft_scheme],
    //     }.validate())
    // }

    pub fn inhib(src: &str, field_radius: u8, exe_order: usize) -> LayerKind {
        LayerKind::Cellular(CellScheme {
            cell_class: CellClass::Control {
                kind: ControlCellKind::InhibitoryBasketSurround {
                    host_lyr_name: src.to_owned(),
                    field_radius: field_radius
                },
                exe_order,
            },
            tft_schemes: Vec::new(),
        }.validate())
    }

    pub fn smooth(src: &str, field_radius: u8, exe_order: usize) -> LayerKind {
        LayerKind::Cellular(CellScheme {
            cell_class: CellClass::Control {
                kind: ControlCellKind::ActivitySmoother {
                    host_lyr_name: src.to_owned(),
                    field_radius: field_radius
                },
                exe_order,
            },
            tft_schemes: Vec::new(),
        }.validate())
    }

    pub fn pyr_outputter(src: &str, exe_order: usize) -> LayerKind {
        LayerKind::Cellular(CellScheme {
            cell_class: CellClass::Control {
                kind: ControlCellKind::PyrOutputter {
                    host_lyr_name: src.to_owned(),
                },
                exe_order,
            },
            tft_schemes: Vec::new(),
        }.validate())
    }

    // pub fn minicolumn(exe_order: usize) -> LayerKind {
    //     // let tft_scheme = TuftScheme::new(DendriteClass::Basal, DendriteKind::Other, 0, 0,
    //     //     vec![TuftSourceLayer::new(psal_lyr.to_owned(), 0, 1),
    //     //     TuftSourceLayer::new(ptal_lyr.to_owned(), 0, 1)], None).with_tft_id(0);

    //     LayerKind::Cellular(CellScheme {
    //         cell_class: CellClass::Control { kind: ControlCellKind::Complex, exe_order, },
    //         tft_schemes: Vec::new(),
    //     }.validate())
    // }

    pub fn add_tft(&mut self, tft: TuftScheme) {
        // let tft_id = self.tft_schemes.len();
        self.tft_schemes.push(tft);
    }

    pub fn validate(self) -> CellScheme {
        for tft_scheme in self.tft_schemes.iter() {
            for src_lyr in tft_scheme.src_lyrs.iter() {
                assert!(src_lyr.syn_reach >= 0, "Synapse reach must be greater than zero.");
            }
        }

        self
    }

    // // [FIXME]: This check would be better to do within `CorticalArea`.
    // pub fn validate_depth(&self, depth: Option<u8>) -> Option<u8> {
    //     match self.cell_class {
    //         CellClass::Control { ref kind, exe_order: _ } => match *kind {
    //             ControlCellKind::InhibitoryBasketSurround { .. } => Some(0),
    //             ControlCellKind::ActivitySmoother { .. } => Some(0),
    //             ControlCellKind::PyrOutputter { .. } => Some(0),
    //             ControlCellKind::Complex => Some(cmn::DEFAULT_OUTPUT_LAYER_DEPTH),
    //             // _ => ,
    //         },
    //         _ => depth,
    //     }
    // }

    #[inline] pub fn data_cell_kind(&self) -> Option<&DataCellKind> {
        match self.cell_class {
            CellClass::Data(ref ck) => Some(ck),
            _ => None,
        }
    }

    #[inline] pub fn control_cell_kind(&self) -> Option<&ControlCellKind> {
        match self.cell_class {
            CellClass::Control { ref kind, exe_order: _ } => Some(kind),
            _ => None,
        }
    }


    // #[inline] pub fn cols_per_cel_l2(&self) -> u8 { self.cols_per_cel_l2 }
    #[inline] pub fn class(&self) -> &CellClass { &self.cell_class }
    #[inline] pub fn tft_schemes(&self) -> &[TuftScheme] { self.tft_schemes.as_slice() }
    #[inline] pub fn tft_count(&self) -> usize { self.tft_schemes.len() }
}


pub struct CellSchemeDefinition {
    cell_class: CellClass,
    tft_schemes: Vec<TuftScheme>,
}

impl CellSchemeDefinition {
    pub fn new(cell_class: CellClass) -> CellSchemeDefinition {
        CellSchemeDefinition {
            cell_class,
            tft_schemes: Vec::with_capacity(3),
        }
    }

    pub fn tft(mut self, tft: TuftSchemeDefinition) -> CellSchemeDefinition {
        let tft_id = self.tft_schemes.len();
        self.tft_schemes.push(tft.build(tft_id));
        self
    }

    pub fn build(self) -> CellScheme {
        CellScheme {
            cell_class: self.cell_class,
            tft_schemes: self.tft_schemes,
        }
    }
}