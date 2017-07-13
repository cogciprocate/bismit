use map::{/*CellKind,*/ CellClass, LayerKind, DendriteClass, DendriteKind, DataCellKind,
    ControlCellKind};
use cmn;


/// A source layer.
///
/// `prevalence` is a simple weight or factor applied to each layer during
/// learning. If one source layer has a `prevalance` of `5` and all other
/// source layers for a tuft have a `prevalance` of `1`, the source layer with
/// the `5` will be five times more likely to form a synapse during
/// regrowth/growth.
///
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct TuftSourceLayer {
    name: String,
    syn_reach: i8,
    prevalence: u8,
}

impl TuftSourceLayer {
    pub fn new(name: String, syn_reach: i8, prevalence: u8) -> TuftSourceLayer {
        assert!(prevalence > 0, "Tuft source layer definitions must have a prevalence \
            of at least one. {{ Layer: name: {}, reach: {} }}", name, syn_reach);

        TuftSourceLayer {
            name: name,
            syn_reach: syn_reach,
            prevalence: prevalence,
        }
    }

    #[inline] pub fn name(&self) -> &str { self.name.as_str() }
    #[inline] pub fn syn_reach(&self) -> i8 { self.syn_reach }
    #[inline] pub fn prevalence(&self) -> u8 { self.prevalence }
}

impl<'a> From<(&'a str, i8, u8)> for TuftSourceLayer {
    fn from(tup: (&'a str, i8, u8)) -> TuftSourceLayer {
        TuftSourceLayer::new(tup.0.to_owned(), tup.1, tup.2)
    }
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct TuftScheme {
    tft_id: Option<usize>,
    den_class: DendriteClass,
    den_kind: DendriteKind,
    dens_per_tft_l2: u8,
    syns_per_den_l2: u8,
    src_lyrs: Vec<TuftSourceLayer>,
    thresh_init: Option<u32>,
}

impl TuftScheme {
    pub fn new(den_class: DendriteClass, den_kind: DendriteKind, dens_per_tft_l2: u8,
            syns_per_den_l2: u8, src_lyrs: Vec<TuftSourceLayer>, thresh_init: Option<u32>)
            -> TuftScheme
    {
        TuftScheme {
            tft_id: None,
            den_class: den_class,
            den_kind: den_kind,
            dens_per_tft_l2: dens_per_tft_l2,
            syns_per_den_l2: syns_per_den_l2,
            src_lyrs: src_lyrs,
            thresh_init: thresh_init,
        }
    }

    pub fn and_tft_id(mut self, tft_id: usize) -> TuftScheme {
        self.tft_id = Some(tft_id);
        self
    }

    #[inline] pub fn tft_id(&self) -> usize { self.tft_id.expect("Tuft ID not set!") }
    #[inline] pub fn den_class(&self) -> &DendriteClass { &self.den_class }
    #[inline] pub fn den_kind(&self) -> &DendriteKind { &self.den_kind }
    #[inline] pub fn dens_per_tft_l2(&self) -> u8 { self.dens_per_tft_l2 }
    #[inline] pub fn syns_per_den_l2(&self) -> u8 { self.syns_per_den_l2 }
    #[inline] pub fn syns_per_tft_l2(&self) -> u8 { self.dens_per_tft_l2 + self.syns_per_den_l2 }
    #[inline] pub fn src_lyrs(&self) -> &[TuftSourceLayer] { self.src_lyrs.as_slice() }
    #[inline] pub fn thresh_init(&self) -> &Option<u32> { &self.thresh_init }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct CellScheme {
    cell_class: CellClass,
    tft_schemes: Vec<TuftScheme>,
}

impl CellScheme {
    pub fn new(
            cell_class: CellClass,
            tft_schemes: Vec<TuftScheme>,
            ) -> CellScheme
    {

        // DO SOME CHECKS ON PARAMETERS (certain cell types must/mustn't have certain dendritic segments)
        CellScheme {
            cell_class: cell_class,
            tft_schemes: tft_schemes,
        }.validate()
    }

    //                             &[name, reach, prevalance]
    pub fn pyramidal<'a>(dst_srcs: &[(&'a str, i8, u8)], dens_per_tft_l2: u8, syns_per_den_l2: u8,
            thresh: u32) -> LayerKind
    {
        let src_lyrs_vec = dst_srcs.into_iter().map(|&sl| sl.into()).collect();

        let tft_scheme = TuftScheme::new(DendriteClass::Basal, DendriteKind::Distal,
            dens_per_tft_l2, syns_per_den_l2, src_lyrs_vec, Some(thresh)).and_tft_id(0);

        LayerKind::Cellular(CellScheme {
            cell_class: CellClass::Data(DataCellKind::Pyramidal),
            tft_schemes: vec![tft_scheme]
        }.validate())
    }

    // SWITCH TO DISTAL
    //                                  &[name, reach, prevalance]
    pub fn spiny_stellate<'a>(prx_srcs: &[(&'a str, i8, u8)], syns_per_den_l2: u8, thresh: u32,
            ) -> LayerKind
    {
        let src_lyrs_vec = prx_srcs.into_iter().map(|&sl| sl.into()).collect();

        let tft_scheme = TuftScheme::new(DendriteClass::Basal, DendriteKind::Proximal, 0,
            syns_per_den_l2, src_lyrs_vec, Some(thresh)).and_tft_id(0);

        LayerKind::Cellular(CellScheme {
            cell_class: CellClass::Data(DataCellKind::SpinyStellate),
            tft_schemes: vec![tft_scheme],
        }.validate())
    }

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

    pub fn minicolumn(psal_lyr: &'static str, ptal_lyr: &'static str, exe_order: usize) -> LayerKind {
        let tft_scheme = TuftScheme::new(DendriteClass::Basal, DendriteKind::Other, 0, 0,
            vec![TuftSourceLayer::new(psal_lyr.to_owned(), 0, 1),
            TuftSourceLayer::new(ptal_lyr.to_owned(), 0, 1)], None).and_tft_id(0);

        LayerKind::Cellular(CellScheme {
            cell_class: CellClass::Control { kind: ControlCellKind::Complex, exe_order, },
            tft_schemes: vec![tft_scheme],
        }.validate())
    }

    pub fn add_tft(&mut self, tft: TuftScheme) {
        let tft_id = self.tft_schemes.len();
        self.tft_schemes.push(tft.and_tft_id(tft_id));
    }

    pub fn validate(self) -> CellScheme {
        for tft_scheme in self.tft_schemes.iter() {
            for src_lyr in tft_scheme.src_lyrs.iter() {
                assert!(src_lyr.syn_reach >= 0, "Synapse reach must be greater than zero.");
            }
        }

        self
    }

    // [FIXME]: This check would be better to do within `CorticalArea`.
    pub fn validate_depth(&self, depth: Option<u8>) -> Option<u8> {
        match self.cell_class {
            CellClass::Control { ref kind, exe_order: _ } => match *kind {
                ControlCellKind::InhibitoryBasketSurround { .. } => Some(0),
                ControlCellKind::ActivitySmoother { .. } => Some(0),
                ControlCellKind::Complex => Some(cmn::DEFAULT_OUTPUT_LAYER_DEPTH),
                // _ => ,
            },
            _ => depth,
        }
    }

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
    #[inline] pub fn cell_class(&self) -> &CellClass { &self.cell_class }
    #[inline] pub fn tft_schemes(&self) -> &[TuftScheme] { self.tft_schemes.as_slice() }
    #[inline] pub fn tft_count(&self) -> usize { self.tft_schemes.len() }
}
