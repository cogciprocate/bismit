use map::{LayerTags, LayerKind, AxonTopology, AxonDomain};


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct LayerScheme {
    layer_id: Option<usize>,
    name: &'static str,
    kind: LayerKind,
    depth: Option<u8>,
    layer_tags: LayerTags,
    axon_domain: AxonDomain,
}

impl LayerScheme {
    pub fn new<D>(name: &'static str, kind: LayerKind, depth: Option<u8>, layer_tags: LayerTags,
            axn_domain: D) -> LayerScheme
            where D: Into<AxonDomain>
    {
        // if cfg!(debug) { layer_tags.debug_validate(); }

        LayerScheme {
            layer_id: None,
            name: name,
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
    pub fn name(&self) -> &'static str { self.name }
    pub fn kind(&self) -> &LayerKind { &self.kind }
    pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    pub fn axon_domain(&self) -> &AxonDomain { &self.axon_domain }
}