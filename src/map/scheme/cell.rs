//use bittags;
// use map::LayerKind::{self, Cellular};
use map::{CellKind, CellClass, LayerKind, DendriteClass, DendriteKind};
//use std::option::{Option};
use cmn;


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct DenSrcLyr {
    name: String,
    syn_reach: i8,
}

impl DenSrcLyr {
    pub fn new(name: String, syn_reach: i8) -> DenSrcLyr {
        DenSrcLyr { name: name, syn_reach: syn_reach }
    }
}


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct DendriteScheme {
    class: DendriteClass,
    kind: DendriteKind,
    src_lyrs: Vec<DenSrcLyr>,
    thresh_init: Option<u32>,
}

/* PROTOCELL:
         Merge srcs to a Vec<Box<Vec<..>>>, A Vec of src vec lists
            - use composable functions to define
            - maybe redefine Vec<&'static str> to it's own type with an enum property
            defining what it's source type is
*/
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct CellScheme {
    pub dens_per_tuft_l2: u8,
    pub syns_per_den_l2: u8,
    pub cols_per_cel_l2: u8,
    pub cell_kind: CellKind,
    pub cell_class: CellClass,
    // pub den_prx_src_lyrs: Option<Vec<&'static str>>,
    // pub den_dst_src_lyrs: Option<Vec<(Vec<&'static str>, DendriteClass)>>,
    // pub den_prx_syn_reach: i8,
    // pub den_dst_syn_reaches: Vec<i8>,
    // pub den_thresh_init: Option<u32>,
    pub den_schemes: Vec<DendriteScheme>,
}

impl CellScheme {
    pub fn new(
            dens_per_tuft_l2: u8,
            syns_per_den_l2: u8,
            // cols_per_cel_l2: u8,
            cell_kind: CellKind,
            cell_class: CellClass,
            den_dst_src_lyrs: Option<Vec<(Vec<&'static str>, DendriteClass)>>,
            den_prx_src_lyrs: Option<Vec<&'static str>>,
            den_prx_syn_reach: i8,
            den_dst_syn_reaches: Vec<i8>,
            thresh: Option<u32>,
            den_schemes: Vec<DendriteScheme>,
            ) -> CellScheme
    {

        // DO SOME CHECKS ON PARAMETERS (certain cell types must/mustn't have certain dendritic segments)
        CellScheme {
            cell_kind: cell_kind,
            cell_class: cell_class,
            dens_per_tuft_l2: dens_per_tuft_l2,
            syns_per_den_l2: syns_per_den_l2,
            cols_per_cel_l2: 0,
            // den_dst_src_lyrs: den_dst_src_lyrs,
            // den_prx_src_lyrs: den_prx_src_lyrs,
            // den_prx_syn_reach: den_prx_syn_reach,
            // den_dst_syn_reaches: den_dst_syn_reaches,
            // den_thresh_init: thresh,
            den_schemes: den_schemes,
        }.validate()
    }

    pub fn pyramidal(dens_per_tuft_l2: u8, syns_per_den_l2: u8, dst_srcs: Vec<&'static str>,
            thresh: u32, dst_reach: i8) -> LayerKind
    {
        let den_scheme = DendriteScheme {
            class: DendriteClass::Basal,
            kind: DendriteKind::Distal,
            src_lyrs: dst_srcs.iter().map(|&lyr_name|
                DenSrcLyr::new(lyr_name.to_owned(), dst_reach)).collect(),
            thresh_init: Some(thresh),
        };

        LayerKind::Cellular(CellScheme {
            dens_per_tuft_l2: dens_per_tuft_l2,
            syns_per_den_l2: syns_per_den_l2,
            cols_per_cel_l2: 0,
            cell_kind: CellKind::Pyramidal,
            cell_class: CellClass::Data,
            // den_dst_src_lyrs: Some(vec![(dst_srcs, DendriteClass::Basal)]),
            // den_prx_src_lyrs: None,
            // den_prx_syn_reach: dst_reach,
            // den_dst_syn_reaches: vec![dst_reach],
            // den_thresh_init: Some(thresh),
            den_schemes: vec![den_scheme]
        }.validate())
    }

    // SWITCH TO DISTAL
    pub fn spiny_stellate(syns_per_den_l2: u8, prx_srcs: Vec<&'static str>, thresh: u32,
            prx_reach: i8) -> LayerKind
    {
        let den_scheme = DendriteScheme {
            class: DendriteClass::Basal,
            kind: DendriteKind::Proximal,
            src_lyrs: prx_srcs.iter().map(|&lyr_name|
                DenSrcLyr::new(lyr_name.to_owned(), prx_reach)).collect(),
            thresh_init: Some(thresh),
        };

        LayerKind::Cellular(CellScheme {
            dens_per_tuft_l2: 0,
            syns_per_den_l2: syns_per_den_l2,
            cols_per_cel_l2: 0,
            cell_kind: CellKind::SpinyStellate,
            cell_class: CellClass::Data,
            // den_dst_src_lyrs: None, // Some(vec![dst_srcs]),
            // den_prx_src_lyrs: Some(prx_srcs),
            // den_prx_syn_reach: prx_reach,
            // den_dst_syn_reaches: Vec::new(),
            // den_thresh_init: Some(thresh),
            den_schemes: vec![den_scheme],
        }.validate())
    }

    pub fn inhibitory(cols_per_cel_l2: u8, dst_src: &'static str) -> LayerKind {
        let den_scheme = DendriteScheme {
            class: DendriteClass::Basal,
            kind: DendriteKind::Other,
            src_lyrs: vec![DenSrcLyr::new(dst_src.to_owned(), 0)],
            thresh_init: None,
        };

        LayerKind::Cellular(CellScheme {
            dens_per_tuft_l2: 0,
            syns_per_den_l2: 0,
            cols_per_cel_l2: cols_per_cel_l2,
            cell_kind: CellKind::Inhibitory,
            cell_class: CellClass::Control,
            // [FIXME]: Create a better place to store this source information for control cells:
            // den_dst_src_lyrs: Some(vec![(vec![dst_src], DendriteClass::Basal)]),
            // den_prx_src_lyrs: None,
            // den_prx_syn_reach: 0,
            // den_dst_syn_reaches: vec![0],
            // den_thresh_init: None,
            den_schemes: vec![den_scheme],
        }.validate())
    }

    pub fn minicolumn(psal_lyr: &'static str, ptal_lyr: &'static str,) -> LayerKind {
        let den_scheme = DendriteScheme {
            class: DendriteClass::Basal,
            kind: DendriteKind::Other,
            src_lyrs: vec![DenSrcLyr::new(psal_lyr.to_owned(), 0),
                DenSrcLyr::new(ptal_lyr.to_owned(), 0)],
            thresh_init: None,
        };

        LayerKind::Cellular(CellScheme {
            dens_per_tuft_l2: 0,
            syns_per_den_l2: 0,
            cols_per_cel_l2: 0,
            cell_kind: CellKind::Complex,
            cell_class: CellClass::Control,
            // [FIXME]: Create a better place to store this source information for control cells:
            // den_dst_src_lyrs: Some(vec![(vec![psal_lyr], DendriteClass::Basal),
            //     (vec![ptal_lyr], DendriteClass::Basal)]),
            // den_prx_src_lyrs: None,
            // den_prx_syn_reach: 0,
            // den_dst_syn_reaches: vec![0],
            // den_thresh_init: None,
            den_schemes: vec![den_scheme],
        }.validate())
    }

    pub fn validate(self) -> CellScheme {
        // assert!(self.den_prx_syn_reach >= 0, "Synapse reach must be between 0..127");

        // for &reach in self.den_dst_syn_reaches.iter() {
        //     assert!(reach >= 0, "Synapse reach must be between 0..127");
        // }

        for den_scheme in self.den_schemes.iter() {
            for src_lyr in den_scheme.src_lyrs.iter() {
                assert!(src_lyr.syn_reach >= 0);
            }
        }

        self
    }

    pub fn validate_depth(&self, depth: Option<u8>) -> Option<u8> {
        match self.cell_kind {
            CellKind::Inhibitory => Some(0),
            CellKind::Complex => Some(cmn::DEFAULT_OUTPUT_LAYER_DEPTH),
            _ => depth,
        }
    }
}
