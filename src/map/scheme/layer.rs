use map::{LayerTags, AxonTopology, AxonDomain, CellScheme, CellSchemeDefinition};


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
    pub fn axon_topology(&self) -> AxonTopology {
        match *self {
            LayerKind::Axonal(ak) => ak.clone(),
            LayerKind::Cellular(_) => AxonTopology::Spatial,
        }
    }

    pub fn cell_scheme(&self) -> Option<&CellScheme> {
        match *self {
            LayerKind::Cellular(ref cs) => Some(cs),
            LayerKind::Axonal(_) => None,
        }
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
            axon_domain: D) -> LayerScheme
            where S: Into<String>, D: Into<AxonDomain> {
        LayerScheme {
            layer_id,
            name: name.into(),
            kind: kind,
            depth: depth,
            tags: tags,
            axon_domain: axon_domain.into(),
        }
    }

    pub fn axon_topology(&self) -> AxonTopology {
        self.kind.axon_topology()
    }

    pub fn layer_id(&self) -> usize { self.layer_id }
    pub fn depth(&self) -> Option<u8> { self.depth }
    pub fn name<'s>(&'s self) -> &'s str { &self.name }
    pub fn kind(&self) -> &LayerKind { &self.kind }
    pub fn tags(&self) -> LayerTags { self.tags }
    pub fn axon_domain(&self) -> &AxonDomain { &self.axon_domain }
}


#[derive(Clone, Debug)]
pub struct LayerSchemeDefinition {
    name: String,
    kind: Option<LayerKind>,
    depth: Option<u8>,
    tags: LayerTags,
    axon_domain: AxonDomain,
}

impl LayerSchemeDefinition {
    pub fn new<S: Into<String>>(name: S) -> LayerSchemeDefinition {
        LayerSchemeDefinition {
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