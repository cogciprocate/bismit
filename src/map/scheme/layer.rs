use cmn::{CmnError};
use map::{LayerTags, LayerKind, AxonTopology, DendriteKind, AxonDomain};


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct LayerScheme {
    layer_id: Option<usize>,
    name: &'static str,
    kind: LayerKind,
    depth: Option<u8>,
    // base_slc_id: u8,
    // kind_base_slc_id: u8,
    layer_tags: LayerTags,
    axon_domain: AxonDomain,
    // axon_tags: AxonTags,
}

impl LayerScheme {
    pub fn new(name: &'static str, kind: LayerKind, depth: Option<u8>, layer_tags: LayerTags,
            axon_domain: AxonDomain) -> LayerScheme
    {
        if cfg!(debug) { layer_tags.debug_validate(); }

        LayerScheme {
            layer_id: None,
            name: name,
            kind: kind,
            depth: depth,
            layer_tags: layer_tags,
            axon_domain: axon_domain,
        }
    }

    // pub fn set_depth(&mut self, depth: u8) {
    //     self.depth = Some(depth);
    // }


    // SRC_LAYER_NAMES(): TODO: DEPRICATE OR RENAME
    pub fn src_lyr_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
        let layer_names = match self.kind {
            LayerKind::Cellular(ref cell_scheme) => match den_type {
                DendriteKind::Distal => Some(cell_scheme.den_dst_src_lyrs.clone().unwrap()[0].clone()),
                DendriteKind::Proximal => cell_scheme.den_prx_src_lyrs.clone(),
            },
            _ => panic!(format!("LayerScheme '{}' is not 'Cellular'.", self.name)),
        };

        match layer_names {
            Some(v) => v,
            None => Vec::with_capacity(0),
        }
    }

    pub fn dst_src_lyrs_by_tuft(&self) -> Vec<Vec<&'static str>> {
        let layers_by_tuft = match self.kind {
            LayerKind::Cellular(ref cell_scheme) => cell_scheme.den_dst_src_lyrs.clone(),
            _ => panic!(format!("LayerScheme '{}' is not 'Cellular'.", self.name)),
        };

        match layers_by_tuft {
            Some(v) => v,
            None => Vec::with_capacity(0),
        }
    }

    pub fn axn_kind(&self) -> Result<AxonTopology, CmnError> {
        match self.kind {
            LayerKind::Axonal(ak) => Ok(ak.clone()),
            LayerKind::Cellular(_) => Ok(AxonTopology::Spatial),
                // Ok(try!(AxonTopology::from_tags(self.tags))),
        }
    }

    pub fn set_layer_id(&mut self, layer_id: usize) { self.layer_id = Some(layer_id) }
    pub fn layer_id(&self) -> usize { self.layer_id.expect("Layer ID not set!") }
    pub fn depth(&self) -> Option<u8> { self.depth }
    pub fn name(&self) -> &'static str { self.name }
    pub fn kind(&self) -> &LayerKind { &self.kind }
    pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    pub fn axon_domain(&self) -> &AxonDomain { &self.axon_domain }
}