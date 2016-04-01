use std::collections::{BTreeMap, HashMap};
// use std::ops::{Range};
use std::slice::{Iter};

use proto::{ProtoareaMap, ProtoareaMaps, ProtolayerMaps, LayerMapKind};
use cmn::{self};
use map::{LayerTags, LayerInfo, SourceLayerInfo};
use external_source::ExternalSource;


#[derive(Clone)]
// [FIXME]: TODO: Add caches.
pub struct LayerMap {
    area_name: &'static str,
    index: Vec<LayerInfo>,
    depth: u8,
    kind: LayerMapKind,
}

impl LayerMap {
    pub fn new(pamap: &ProtoareaMap, plmaps: &ProtolayerMaps, pamaps: &ProtoareaMaps, 
            input_sources: &HashMap<String, (ExternalSource, Vec<LayerTags>)>) -> LayerMap 
    {
        println!("{mt}{mt}LAYERMAP::NEW(): Assembling layer map for area \"{}\"...", 
            pamap.name, mt = cmn::MT);

        let plmap = plmaps[pamap.layer_map_name].clone();
        // plmap.freeze(&pamap);

        let mut index = Vec::with_capacity(plmap.layers().len());
        let mut slc_total = 0u8;

        for (_, pl) in plmap.layers().iter() {
            index.push(LayerInfo::new(pl, pamap, pamaps, plmaps, input_sources, &mut slc_total));
        }

        // assert_eq!(slc_total as usize, plmap.slc_map().len());

        // println!("{mt}{mt}LAYERMAP::NEW(): index: {:?}, plmap.slc_map(): {:?}", 
        //     index, plmap.slc_map(), mt = cmn::MT);
        LayerMap { area_name: pamap.name, index: index, depth: slc_total, kind: plmap.kind }
    }

    pub fn slc_map(&self) -> BTreeMap<u8, &LayerInfo> {
        let mut slc_map = BTreeMap::new();
        let mut slc_id_check = 0;

        // println!("\n{mt}Creating Slice Map...", mt = cmn::MT);

        for layer in self.index.iter() {
            // println!("{mt}{mt}Processing layer: '{}', slc_range: {:?}", layer.name(), 
            //     layer.slc_range(), mt = cmn::MT);

            for slc_id in layer.slc_range().clone() {
                // println!("{mt}{mt}{mt}Processing slice: '{}'", slc_id, mt = cmn::MT);
                debug_assert_eq!(slc_id_check, slc_id);

                if slc_map.insert(slc_id, layer).is_some() {
                    // panic!("LayerMap::slc_map(): Duplicate slices found in LayerMap: \
                    //     layer: '{}', slc_id: '{}'.", layer.name(), slc_id);
                }

                slc_id_check = slc_id + 1;
            }
        }

        print!("\n");

        slc_map
    }

    // [FIXME] TODO: Cache results (use TractArea cache style).
    pub fn layer_info(&self, tags: LayerTags) -> Vec<&LayerInfo> {
        self.index.iter().filter(|li| li.tags().contains(tags)).map(|li| li).collect()
    }

    // [FIXME] TODO: Create HashMap to index layer names.
    pub fn layer_info_by_name(&self, name: &'static str) -> &LayerInfo {
        let layers: Vec<&LayerInfo> = self.index.iter().filter(|li| li.name() == name)
            .map(|li| li).collect();
        debug_assert_eq!(layers.len(), 1);
        layers[0]
    }

    // [FIXME] TODO: Cache results. Use iterator mapping and filtering.
    pub fn layer_src_info(&self, tags: LayerTags) -> Vec<&SourceLayerInfo> {
        let mut src_layers = Vec::with_capacity(8);

        for layer in self.layer_info(tags).iter() {
            for src_layer in layer.sources().iter() {
                debug_assert!(src_layer.tags().meshes(tags.mirror_io()));
                src_layers.push(src_layer);
            }
        }

        src_layers
    }

    pub fn layer_src_area_names_by_tags(&self, tags: LayerTags) -> Vec<&'static str> {
        self.layer_src_info(tags).iter().map(|sli| sli.area_name()).collect()
    }

    pub fn slc_src_layer_info(&self, slc_id: u8, layer_tags: LayerTags) -> Option<&SourceLayerInfo> {
        let mut src_layer_info = Vec::with_capacity(8);
        let layer_info = self.layer_info(layer_tags);

        for lyr in layer_info {            
            for src_lyr in lyr.sources() {
                if slc_id >= src_lyr.tar_slc_range().start 
                    && slc_id < src_lyr.tar_slc_range().end
                {
                    src_layer_info.push(src_lyr);
                }
            }
        }

        if src_layer_info.len() == 1 {
            Some(src_layer_info[0])
        } else {
            None
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<LayerInfo> {
        self.index.iter()
    }

    #[inline]
    pub fn region_kind(&self) -> &LayerMapKind {
        &self.kind
    }

    #[inline]
    pub fn depth(&self) -> u8 {
        self.depth
    }
}

