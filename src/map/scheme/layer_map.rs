// use std::collections::{HashMap};
use std::ops::{Index, IndexMut, Deref};
// use std::hash::{Hasher};
use cmn::MapStore;
use map::{LayerTags, LayerMapKind, LayerScheme, LayerSchemeDefinition, AxonTopology, LayerKind,
    AxonDomain, AxonTags};



#[derive(Debug, Clone)]
pub struct LayerMapScheme {
    name: String,
    kind: LayerMapKind,
    layers: MapStore<String, LayerScheme>,
}

impl LayerMapScheme {
    pub fn new<S>(name: S, kind: LayerMapKind) -> LayerMapScheme
            where S: Into<String> {
        LayerMapScheme {
            name: name.into(),
            kind: kind,
            layers: MapStore::new(),
        }
    }

    // <A: Into<AxonTags>>
    pub fn input_layer<S, D>(self, layer_name: S, layer_tags: LayerTags,
            axon_domain: D, axn_topo: AxonTopology) -> LayerMapScheme
            where S: Into<String>, D: Into<AxonDomain>
    {
        self.layer(
            // LayerScheme::new(layer_name, LayerKind::Axonal(axn_kind), None, layer_tags,
            //     axn_domain)
            LayerScheme::define(layer_name)
                .axonal(axn_topo)
                .tags(layer_tags)
                .axon_domain(axon_domain)
        )
    }

    // pub fn axn_layer(mut self, layer_name: S, layer_tags: LayerTags,
    //         axn_domain: AxonDomain, axn_kind: AxonTopology) -> LayerMapScheme
    // {
    //     self.add_layer(LayerScheme::new(layer_name, LayerKind::Axonal(axn_kind), None, layer_tags,
    //         axn_domain));
    //     self
    // }

    pub fn layer(mut self, bldr: LayerSchemeDefinition) -> LayerMapScheme {
        // let layer_id = self.layers.len();
        self.add_layer(bldr);
        self
    }

    pub fn layer_old<S, D>(self, layer_name: S, layer_depth: u8, layer_tags: LayerTags,
            axon_domain: D, kind: LayerKind) -> LayerMapScheme
            where S: Into<String>, D: Into<AxonDomain>
    {
        // let validated_depth = match kind {
        //     LayerKind::Cellular(ref cell_scheme) => cell_scheme.validate_depth(Some(layer_depth)),
        //     LayerKind::Axonal(_) => Some(layer_depth),
        // };

        // self.add_layer(LayerScheme::new(layer_name, kind, validated_depth, layer_tags, axn_domain));
        // self

        self.layer(
            LayerScheme::define(layer_name)
                .kind(kind)
                .depth(layer_depth)
                .tags(layer_tags)
                .axon_domain(axon_domain)
        )
    }

    pub fn add_layer(&mut self, layer_bldr: LayerSchemeDefinition) {
        // layer.set_layer_id();
        let layer_id = self.layers.len();
        let layer = layer_bldr.build(layer_id);
        self.layers.insert(layer.name().to_owned(), layer)
            .map(|ls| panic!("LayerMapScheme::layer(): Duplicate layer names: \
                (layer: \"{}\").", ls.name()));
    }

    /// Returns all layers containing 'tags'.
    pub fn layers_with_layer_tags(&self, layer_tags: LayerTags) -> Vec<&LayerScheme> {
        let mut layers: Vec<&LayerScheme> = Vec::with_capacity(16);

        for layer in self.layers.values().iter().filter(|layer|
             layer.tags().meshes(layer_tags))
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

     pub fn name<'s>(&'s self) -> &'s str {
        &self.name
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