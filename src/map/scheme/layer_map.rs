use std::collections::{HashMap};
use std::ops::{Index, IndexMut, };
// use std::hash::{Hasher};

use map::{LayerTags, LayerMapKind, LayerScheme, AxonTopology, LayerKind, AxonDomain};


/// A map of `LayerMapScheme`s indexed by layer map name.
pub struct LayerMapSchemeList {
    map: HashMap<&'static str, LayerMapScheme>,
}

impl LayerMapSchemeList {
    pub fn new() -> LayerMapSchemeList {
        LayerMapSchemeList {
            map: HashMap::new(),
        }
    }

    pub fn lmap(mut self, pr: LayerMapScheme) -> LayerMapSchemeList {
        self.add(pr);
        self
    }

    pub fn add(&mut self, pr: LayerMapScheme) {
        self.map.insert(pr.name.clone(), pr);
    }
}

impl<'b> Index<&'b str> for LayerMapSchemeList
{
    type Output = LayerMapScheme;

    fn index<'a>(&'a self, region_name: &'b str) -> &'a LayerMapScheme {
        self.map.get(region_name).expect(&format!("map::regions::LayerMapSchemeList::index(): \
            Invalid layer map name: '{}'.", region_name))
    }
}

impl<'b> IndexMut<&'b str> for LayerMapSchemeList
{
    fn index_mut<'a>(&'a mut self, region_name: &'b str) -> &'a mut LayerMapScheme {
        self.map.get_mut(region_name).expect(&format!("map::regions::LayerMapSchemeList::index_mut(): \
            Invalid layer map name: '{}'.", region_name))
    }
}



#[derive(Clone)]
pub struct LayerMapScheme {
    pub name: &'static str,
    pub kind: LayerMapKind,
    layers: HashMap<&'static str, LayerScheme>,
}

impl LayerMapScheme {
    pub fn new (region_name: &'static str, kind: LayerMapKind) -> LayerMapScheme {
        LayerMapScheme {
            name: region_name,
            kind: kind,
            layers: HashMap::new(),
        }
    }

    // <A: Into<AxonTags>>
    pub fn input_layer(mut self, layer_name: &'static str, layer_tags: LayerTags,
            axn_domain: AxonDomain, axn_kind: AxonTopology) -> LayerMapScheme
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
    pub fn layer(mut self, layer_name: &'static str, layer_depth: u8, layer_tags: LayerTags,
            axn_domain: AxonDomain, kind: LayerKind) -> LayerMapScheme
    {
        let validated_depth = match kind {
            LayerKind::Cellular(ref cell_scheme) => cell_scheme.validate_depth(Some(layer_depth)),
            LayerKind::Axonal(_) => Some(layer_depth),
        };

        self.add(LayerScheme::new(layer_name, kind, validated_depth, layer_tags, axn_domain));
        self
    }

    // [FIXME][DONE]: NEED TO CHECK FOR DUPLICATE LAYERS!
    pub fn add(&mut self, layer: LayerScheme) {
        let layer_name = layer.name();
        self.layers.insert(layer.name(), layer)
            .map(|_| panic!("LayerMapScheme::layer(): Duplicate layers: \
                (layer: \"{}\")", layer_name));
    }

     /// Returns all layers containing 'tags'.
    pub fn layers_with_tags(&self, layer_tags: LayerTags) -> Vec<&LayerScheme> {
         let mut layers: Vec<&LayerScheme> = Vec::with_capacity(16);

         for (_, layer) in self.layers.iter().filter(|&(_, layer)|
                 layer.layer_tags().meshes(layer_tags))
         {
             layers.push(&layer);
         }

         layers
     }

     pub fn layers(&self) -> &HashMap<&'static str, LayerScheme> {
        &self.layers
    }

     pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn kind(&self) -> &LayerMapKind {
        &self.kind
    }
}

impl<'b> Index<&'b&'static str> for LayerMapScheme
{
    type Output = LayerScheme;

    fn index<'a>(&'a self, index: &'b&'static str) -> &'a LayerScheme {
        self.layers.get(index).unwrap_or_else(|| panic!("LayerMapScheme::index(): invalid layer name: '{}'", index))
    }
}

impl<'b> IndexMut<&'b&'static str> for LayerMapScheme
{
    fn index_mut<'a>(&'a mut self, index: &'b&'static str) -> &'a mut LayerScheme {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[LayerMapScheme::index(): invalid layer name: '{}'", index))
    }
}
