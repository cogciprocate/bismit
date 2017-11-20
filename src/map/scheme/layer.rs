use map::{LayerTags, AxonTopology, AxonDomain, CellScheme, TuftScheme, DendriteClass,
    DendriteKind};


// * TODO: Figure out whether or not to keep `AxonTopology` here since only
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
                let src_lyrs_vec = src_lyrs.into_iter().map(|&sl| sl.into()).collect();

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
pub struct LayerScheme {
    layer_id: Option<usize>,
    name: String,
    kind: LayerKind,
    depth: Option<u8>,
    layer_tags: LayerTags,
    axon_domain: AxonDomain,
}

impl LayerScheme {
    pub fn builder() -> LayerSchemeBuilder {
        LayerSchemeBuilder::new()
    }

    pub fn new<S, D>(name: S, kind: LayerKind, depth: Option<u8>, layer_tags: LayerTags,
            axn_domain: D) -> LayerScheme
            where S: Into<String>, D: Into<AxonDomain>
    {
        // if cfg!(debug) { layer_tags.debug_validate(); }

        LayerScheme {
            layer_id: None,
            name: name.into(),
            kind: kind,
            depth: depth,
            layer_tags: layer_tags,
            axon_domain: axn_domain.into(),
        }
    }

    pub fn axn_topology(&self) -> AxonTopology {
        self.kind.axn_topology()
    }

    pub fn set_layer_id(&mut self, layer_id: usize) { self.layer_id = Some(layer_id) }
    pub fn layer_id(&self) -> usize { self.layer_id.expect("Layer ID not set!") }
    pub fn depth(&self) -> Option<u8> { self.depth }
    pub fn name<'s>(&'s self) -> &'s str { &self.name }
    pub fn kind(&self) -> &LayerKind { &self.kind }
    pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    pub fn axon_domain(&self) -> &AxonDomain { &self.axon_domain }
}


pub struct LayerSchemeBuilder {
    layer_id: Option<usize>,
    name: Option<String>,
    kind: Option<LayerKind>,
    depth: Option<u8>,
    layer_tags: Option<LayerTags>,
    axon_domain: Option<AxonDomain>,
}

impl LayerSchemeBuilder {
    pub fn new() -> LayerSchemeBuilder {
        LayerSchemeBuilder {
            layer_id: None,
            name: None,
            kind: None,
            depth: None,
            layer_tags: None,
            axon_domain: None,
        }
    }
}