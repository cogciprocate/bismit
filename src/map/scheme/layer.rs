use map::{LayerTags, AxonTopology, AxonDomain, CellScheme, CellSchemeDefinition,
    TuftScheme, DendriteClass, DendriteKind};


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

    pub fn apical<'a>(mut self, tft_id: usize, src_lyrs: &[(&'a str, i8, u8)], dens_per_tft_l2: u8,
                syns_per_den_l2: u8, max_active_dens_l2: u8, thresh_init: u32) -> LayerKind
    {
        match &mut self {
            &mut LayerKind::Cellular(ref mut cs) => {
                let src_lyrs_vec = src_lyrs.into_iter().map(|&sl| sl.into()).collect();

                let tft_scheme = TuftScheme::new(tft_id, DendriteClass::Apical, DendriteKind::Distal,
                    dens_per_tft_l2, syns_per_den_l2, max_active_dens_l2, src_lyrs_vec, Some(thresh_init));

                cs.add_tft(tft_scheme);
            },

            &mut LayerKind::Axonal(_) => panic!("::apical(): Axonal layers do not have dendrites."),
        }

        self
    }
}



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LayerScheme {
    layer_id: usize,
    name: String,
    kind: LayerKind,
    depth: Option<u8>,
    tags: LayerTags,
    axon_domain: AxonDomain,
}

impl LayerScheme {
    pub fn define<S: Into<String>>(name: S) -> LayerSchemeDefinition {
        LayerSchemeDefinition::new(name)
    }

    pub fn new<S, D>(layer_id: usize, name: S, kind: LayerKind, depth: Option<u8>, tags: LayerTags,
            axn_domain: D) -> LayerScheme
            where S: Into<String>, D: Into<AxonDomain>
    {
        // if cfg!(debug) { tags.debug_validate(); }

        LayerScheme {
            layer_id,
            name: name.into(),
            kind: kind,
            depth: depth,
            tags: tags,
            axon_domain: axn_domain.into(),
        }
    }

    pub fn axn_topology(&self) -> AxonTopology {
        self.kind.axn_topology()
    }

    // pub fn set_layer_id(&mut self, layer_id: usize) { self.layer_id = Some(layer_id) }
    // pub fn layer_id(&self) -> usize { self.layer_id.expect("Layer ID not set!") }
    pub fn layer_id(&self) -> usize { self.layer_id }
    pub fn depth(&self) -> Option<u8> { self.depth }
    pub fn name<'s>(&'s self) -> &'s str { &self.name }
    pub fn kind(&self) -> &LayerKind { &self.kind }
    pub fn tags(&self) -> LayerTags { self.tags }
    pub fn axon_domain(&self) -> &AxonDomain { &self.axon_domain }
}


#[derive(Clone, Debug)]
pub struct LayerSchemeDefinition {
    // layer_id: Option<usize>,
    name: String,
    kind: Option<LayerKind>,
    depth: Option<u8>,
    tags: LayerTags,
    axon_domain: AxonDomain,
}

impl LayerSchemeDefinition {
    pub fn new<S: Into<String>>(name: S) -> LayerSchemeDefinition {
        LayerSchemeDefinition {
            // layer_id: None,
            name: name.into(),
            kind: None,
            depth: None,
            tags: LayerTags::DEFAULT,
            axon_domain: AxonDomain::Local,
        }
    }

    pub fn kind(mut self, kind: LayerKind) -> LayerSchemeDefinition {
        assert!(self.kind.is_none());
        self.kind = Some(kind);
        self
    }

    pub fn cellular(mut self, scheme: CellSchemeDefinition) -> LayerSchemeDefinition {
        assert!(self.kind.is_none());
        self.kind = Some(LayerKind::Cellular(scheme.build()));
        self
    }

    pub fn axonal(mut self, topology: AxonTopology) -> LayerSchemeDefinition {
        assert!(self.kind.is_none());
        self.kind = Some(LayerKind::Axonal(topology));
        self
    }

    pub fn depth(mut self, depth: u8) -> LayerSchemeDefinition {
        self.depth = Some(depth);
        self
    }

    pub fn tags(mut self, tags: LayerTags) -> LayerSchemeDefinition {
        self.tags = tags;
        self
    }

    pub fn axon_domain<Ad: Into<AxonDomain>>(mut self, axon_domain: Ad) -> LayerSchemeDefinition {
        self.axon_domain = axon_domain.into();
        self
    }

    pub fn build(self, layer_id: usize) -> LayerScheme {
        LayerScheme {
            layer_id: layer_id,
            name: self.name,
            kind: self.kind.expect("LayerSchemeDefinition::build"),
            depth: self.depth,
            tags: self.tags,
            axon_domain: self.axon_domain,
        }
    }
}