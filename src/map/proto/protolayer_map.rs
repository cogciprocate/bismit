use std::collections::{HashMap};
use std::ops::{Index, IndexMut, };
use std::hash::{Hasher};

use map::{LayerTags};
use super::{Protolayer, AxonKind, LayerKind, Axonal};


#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum LayerMapKind {
    // Associational,
    // Sensory,
    // Motor,
    Cortical,
    Thalamic,
}


#[derive(Clone)]
pub struct ProtolayerMap {
    pub name: &'static str,
    pub kind: LayerMapKind,
    layers: HashMap<&'static str, Protolayer>,
}

impl ProtolayerMap {
    pub fn new (region_name: &'static str, kind: LayerMapKind) -> ProtolayerMap {    
        ProtolayerMap { 
            name: region_name,
            kind: kind,
            layers: HashMap::new(),
        }
    }

    pub fn axn_layer(mut self, layer_name: &'static str, tags: LayerTags, axn_kind: AxonKind) 
            -> ProtolayerMap
    {
        self.add(Protolayer::new(layer_name, Axonal(axn_kind), None, tags));
        self
    }

    // [FIXME]: TODO: Change axonal default depth to 'None' so that input source or layer map can set.
    pub fn layer(mut self, layer_name: &'static str, layer_depth: u8, tags: LayerTags, 
            kind: LayerKind) -> ProtolayerMap 
    {
        let validated_depth = match kind {
            LayerKind::Cellular(ref protocell) => protocell.validate_depth(Some(layer_depth)),
            LayerKind::Axonal(_) => Some(layer_depth),
        };
        
        self.add(Protolayer::new(layer_name, kind, validated_depth, tags));
        self
    }

    // [FIXME][DONE]: NEED TO CHECK FOR DUPLICATE LAYERS!    
    pub fn add(&mut self, layer: Protolayer) {
        let layer_name = layer.name();
        self.layers.insert(layer.name(), layer)
            .map(|_| panic!("ProtolayerMap::layer(): Duplicate layers: \
                (layer: \"{}\")", layer_name));
    }        

     /// Returns all layers containing 'tags'.
    pub fn layers_with_tags(&self, tags: LayerTags) -> Vec<&Protolayer> {
         let mut layers: Vec<&Protolayer> = Vec::with_capacity(16);
                  
         for (_, layer) in self.layers.iter().filter(|&(_, layer)| 
                 layer.tags().meshes(tags))
         {
             layers.push(&layer);
         }

         layers
     }

     pub fn layers(&self) -> &HashMap<&'static str, Protolayer> {
        &self.layers
    }

     pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn kind(&self) -> &LayerMapKind {
        &self.kind
    }
}

impl<'b> Index<&'b&'static str> for ProtolayerMap
{
    type Output = Protolayer;

    fn index<'a>(&'a self, index: &'b&'static str) -> &'a Protolayer {
        self.layers.get(index).unwrap_or_else(|| panic!("ProtolayerMap::index(): invalid layer name: '{}'", index))
    }
}

impl<'b> IndexMut<&'b&'static str> for ProtolayerMap
{
    fn index_mut<'a>(&'a mut self, index: &'b&'static str) -> &'a mut Protolayer {
        self.layers.get_mut(index).unwrap_or_else(|| panic!("[ProtolayerMap::index(): invalid layer name: '{}'", index))
    }
}


/// A map of `ProtolayerMap`s indexed by layer map name.
pub struct ProtolayerMaps {
    map: HashMap<&'static str, ProtolayerMap>,
}

impl ProtolayerMaps {
    pub fn new() -> ProtolayerMaps {
        ProtolayerMaps {
            map: HashMap::new(),
        }
    }

    pub fn lmap(mut self, pr: ProtolayerMap) -> ProtolayerMaps {
        self.add(pr);
        self
    }    

    pub fn add(&mut self, pr: ProtolayerMap) {
        self.map.insert(pr.name.clone(), pr);
    }
}

impl<'b> Index<&'b str> for ProtolayerMaps
{
    type Output = ProtolayerMap;

    fn index<'a>(&'a self, region_name: &'b str) -> &'a ProtolayerMap {
        self.map.get(region_name).expect(&format!("proto::regions::ProtolayerMaps::index(): \
            Invalid layer map name: '{}'.", region_name))
    }
}

impl<'b> IndexMut<&'b str> for ProtolayerMaps
{
    fn index_mut<'a>(&'a mut self, region_name: &'b str) -> &'a mut ProtolayerMap {
        self.map.get_mut(region_name).expect(&format!("proto::regions::ProtolayerMaps::index_mut(): \
            Invalid layer map name: '{}'.", region_name))
    }
}
