// use std::collections::{HashMap};
use std::ops::{Index, IndexMut, Deref};
// use std::hash::{Hasher};
use cmn::MapStore;
use map::{LayerTags, LayerMapKind, LayerScheme, AxonTopology, LayerKind, AxonDomain, AxonTags};



#[derive(Debug, Clone)]
pub struct LayerMapScheme {
    name: &'static str,
    kind: LayerMapKind,
    layers: MapStore<&'static str, LayerScheme>,
}

impl LayerMapScheme {
    pub fn new (name: &'static str, kind: LayerMapKind) -> LayerMapScheme {
        LayerMapScheme {
            name: name,
            kind: kind,
            layers: MapStore::new(),
        }
    }

    // <A: Into<AxonTags>>
    pub fn input_layer<D>(mut self, layer_name: &'static str, layer_tags: LayerTags,
            axn_domain: D, axn_kind: AxonTopology) -> LayerMapScheme
            where D: Into<AxonDomain>
    {
        self.add(LayerScheme::new(layer_name, LayerKind::Axonal(axn_kind), None, layer_tags,
            axn_domain));
        self
    }

    // pub fn axn_layer(mut self, layer_name: &'static str, layer_tags: LayerTags,
    //         axn_domain: AxonDomain, axn_kind: AxonTopology) -> LayerMapScheme
    // {
    //     self.add(LayerScheme::new(layer_name, LayerKind::Axonal(axn_kind), None, layer_tags,
    //         axn_domain));
    //     self
    // }

    // [FIXME]: TODO: Change axonal default depth to 'None' so that input source or layer map can set.
    pub fn layer<D>(mut self, layer_name: &'static str, layer_depth: u8, layer_tags: LayerTags,
            axn_domain: D, kind: LayerKind) -> LayerMapScheme
            where D: Into<AxonDomain>
    {
        let validated_depth = match kind {
            LayerKind::Cellular(ref cell_scheme) => cell_scheme.validate_depth(Some(layer_depth)),
            LayerKind::Axonal(_) => Some(layer_depth),
        };

        self.add(LayerScheme::new(layer_name, kind, validated_depth, layer_tags, axn_domain));
        self
    }

    pub fn add(&mut self, mut layer: LayerScheme) {
        let layer_name = layer.name();
        layer.set_layer_id(self.layers.len());
        self.layers.insert(layer.name(), layer)
            .map(|_| panic!("LayerMapScheme::layer(): Duplicate layer names: \
                (layer: \"{}\").", layer_name));
    }

    /// Returns all layers containing 'tags'.
    pub fn layers_with_layer_tags(&self, layer_tags: LayerTags) -> Vec<&LayerScheme> {
        let mut layers: Vec<&LayerScheme> = Vec::with_capacity(16);

        for layer in self.layers.values().iter().filter(|layer|
             layer.layer_tags().meshes(layer_tags))
        {
         layers.push(&layer);
        }

        layers
    }

    /// Returns all output layers containing 'tags'.
    pub fn output_layers_with_axon_tags(&self, search_tags: &AxonTags) -> Vec<&LayerScheme> {
        let mut layers: Vec<&LayerScheme> = Vec::with_capacity(16);

        for layer in self.layers.values().iter() {
            match *layer.axon_domain() {
                AxonDomain::Output(ref at) => {
                    debug_assert!(at.is_output());

                    if at.tags() == search_tags {
                        layers.push(&layer);
                    }
                },
                _ => (),
            }
        }

        layers
    }

     pub fn layers(&self) -> &[LayerScheme] {
        self.layers.values()
    }

     pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn kind(&self) -> &LayerMapKind {
        &self.kind
    }
}

// impl<'b> Index<&'b&'static str> for LayerMapScheme
// {
//     type Output = LayerScheme;

//     fn index<'a>(&'a self, index: &'b&'static str) -> &'a LayerScheme {
//         self.layers.by_index(index).unwrap_or_else(|| panic!("LayerMapScheme::index(): invalid layer name: '{}'", index))
//     }
// }

// impl<'b> IndexMut<&'b&'static str> for LayerMapScheme
// {
//     fn index_mut<'a>(&'a mut self, index: &'b&'static str) -> &'a mut LayerScheme {
//         self.layers.get_mut(index).unwrap_or_else(|| panic!("[LayerMapScheme::index(): invalid layer name: '{}'", index))
//     }
// }



/// A map of `LayerMapScheme`s indexed by layer map name.
pub struct LayerMapSchemeList {
    schemes: MapStore<String, LayerMapScheme>,
}

impl LayerMapSchemeList {
    pub fn new() -> LayerMapSchemeList {
        LayerMapSchemeList {
            schemes: MapStore::new(),
        }
    }

    pub fn lmap(mut self, lm_scheme: LayerMapScheme) -> LayerMapSchemeList {
        self.add(lm_scheme);
        self
    }

    pub fn add(&mut self, lm_scheme: LayerMapScheme) {
        self.schemes.insert(lm_scheme.name.to_owned(), lm_scheme);
    }
}

impl<'b> Index<&'b str> for LayerMapSchemeList {
    type Output = LayerMapScheme;

    fn index<'a>(&'a self, region_name: &'b str) -> &'a LayerMapScheme {
        self.schemes.by_key(region_name)
            .expect(&format!("map::regions::LayerMapSchemeList::index(): \
            Invalid layer map name: '{}'.", region_name))
    }
}

impl<'b> IndexMut<&'b str> for LayerMapSchemeList {
    fn index_mut<'a>(&'a mut self, region_name: &'b str) -> &'a mut LayerMapScheme {
        self.schemes.by_key_mut(region_name)
            .expect(&format!("map::regions::LayerMapSchemeList::index_mut(): \
            Invalid layer map name: '{}'.", region_name))
    }
}

impl Deref for LayerMapSchemeList {
    type Target = MapStore<String, LayerMapScheme>;

    fn deref(&self) -> &MapStore<String, LayerMapScheme> {
        &self.schemes
    }
}