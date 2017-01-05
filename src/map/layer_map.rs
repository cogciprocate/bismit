use std::collections::{BTreeMap};
use std::ops::{Range};
// use std::ops::{Range};
use std::slice::{Iter};

use map::{AreaScheme, AreaSchemeList, LayerMapSchemeList, LayerMapKind};
use cmn::{self, MapStore};
use map::{LayerTags, LayerInfo, SourceLayerInfo, LayerAddress};
use thalamus::ExternalPathway;

const DEBUG_PRINT: bool = false;

#[derive(Clone)]
// [FIXME]: TODO: Add caches.
pub struct LayerMap {
    area_name: &'static str,
    // layers: Vec<LayerInfo>,
    layers: MapStore<String, LayerInfo>,
    depth: u8,
    kind: LayerMapKind,
}

impl LayerMap {
    pub fn new(area_sch: &AreaScheme, layer_map_sl: &LayerMapSchemeList, area_sl: &AreaSchemeList,
                    ext_paths: &MapStore<String, (ExternalPathway, Vec<LayerAddress>)>) -> LayerMap {
        println!("{mt}{mt}LAYERMAP::NEW(): Assembling layer map for area \"{}\"...",
            area_sch.name(), mt = cmn::MT);
        println!("{mt}{mt}{mt}[Layer ID] <Layer Name>: Option(Slice Range): {{ Layer Tags }}",
            mt = cmn::MT);

        let lm_scheme = layer_map_sl[area_sch.layer_map_name()].clone();
        // let lm_scheme_kind = lm_scheme.kind().clone();
        // lm_scheme.freeze(&area_sch);

        let mut layers = MapStore::with_capacity(lm_scheme.layers().len());
        let mut slc_total = 0u8;

        for (layer_id, ls) in lm_scheme.layers().iter().enumerate() {
            assert!(ls.layer_id() == layer_id);
            let new_layer = LayerInfo::new(layer_id, ls, lm_scheme.kind().clone(), area_sch,
                area_sl, layer_map_sl, ext_paths, slc_total);
            slc_total += new_layer.depth();
            layers.insert(ls.name().to_owned(), new_layer);
            assert!(layers[layer_id].layer_addr().layer_id() == layer_id);
        }

        // assert_eq!(slc_total as usize, lm_scheme.slc_map().len());

        print!("\n");

        let lm = LayerMap { area_name: area_sch.name(), layers: layers, depth: slc_total,
            kind: lm_scheme.kind().clone() };

        if DEBUG_PRINT {
            // println!("{mt}{mt}LAYERMAP::NEW(): layers: {:?}, lm_scheme.slc_map(): {:?}",
            //     layers, lm_scheme.slc_map(), mt = cmn::MT);
            println!("{:#?}", lm.slc_map());
        }

        lm
    }

    pub fn slc_map(&self) -> BTreeMap<u8, &LayerInfo> {
        let mut slc_map = BTreeMap::new();
        let mut slc_id_count = 0;

        // if DEBUG_PRINT {
        //     println!("\n{mt}Creating Slice Map...", mt = cmn::MT);
        // }

        for layer in self.layers.values().iter() {
            // if DEBUG_PRINT {
            //     println!("{mt}{mt}Processing layer: '{}', slc_range: {:?}", layer.name(),
            //         layer.slc_range(), mt = cmn::MT);
            // }

            if layer.slc_range().is_some() {
                for slc_id in layer.slc_range().unwrap().clone() {
                    // if DEBUG_PRINT {
                    //     println!("{mt}{mt}{mt}Processing slice: '{}'", slc_id, mt = cmn::MT);
                    // }
                    debug_assert_eq!(slc_id_count, slc_id);

                    if slc_map.insert(slc_id, layer).is_some() {
                        panic!("LayerMap::slc_map(): Duplicate slices found in LayerMap: \
                            layer: '{}', slc_id: '{}'.", layer.name(), slc_id);
                    }

                    slc_id_count = slc_id + 1;
                }
            }
        }

        // if DEBUG_PRINT {
        //     print!("\n");
        // }

        slc_map
    }

    pub fn layers_meshing_tags(&self, tags: LayerTags) -> Vec<&LayerInfo> {
        self.layers.values().iter().filter(|li| li.layer_tags().meshes(tags)).map(|li| li).collect()
    }

    pub fn layers_meshing_tags_either_way(&self, tags: LayerTags) -> Vec<&LayerInfo> {
        self.layers.values().iter().filter(|li| li.layer_tags().meshes_either_way(tags)).map(|li| li).collect()
    }

    // [FIXME] TODO: Cache results (use TractArea cache style).
    pub fn layers_containing_tags(&self, tags: LayerTags) -> Vec<&LayerInfo> {
        self.layers.values().iter().filter(|li| li.layer_tags().contains(tags)).map(|li| li).collect()
    }

    /// Returns the slice range associated with matching layers.
    pub fn layers_containing_tags_slc_range(&self, layer_tags: LayerTags) -> Vec<Range<u8>> {
        self.layers_containing_tags(layer_tags).iter()
            .filter(|l| l.slc_range().is_some())
            .map(|l| l.slc_range().unwrap().clone())
            .collect()
    }

    // [FIXME] TODO: Cache results. Use iterator mapping and filtering.
    pub fn layers_containing_tags_src_lyrs(&self, tags: LayerTags) -> Vec<&SourceLayerInfo> {
        let mut src_layers = Vec::with_capacity(8);

        for layer in self.layers_containing_tags(tags).iter() {
            for src_layer in layer.sources().iter() {
                if DEBUG_PRINT {
                    println!("LAYER_MAP::LAYER_SRC_INFO(): Comparing: 'src_layer.tags()', \
                        'tags.mirror_io()'.");
                    src_layer.layer_tags().debug_print_compare(tags.mirror_io());
                }
                debug_assert!(src_layer.layer_tags().contains(tags.mirror_io()));
                src_layers.push(src_layer);
            }
        }

        src_layers
    }

    /// Returns a list of source area ids for a given layer.
    pub fn layers_containing_tags_src_area_names(&self, tags: LayerTags) -> Vec<usize> {
        self.layers_containing_tags_src_lyrs(tags).iter().map(|sli| sli.area_id()).collect()
    }

    /// Returns a list of the (area name, layer tags) tuple necessary to
    /// access thalamic tracts.
    pub fn layers_containing_tags_src_tract_keys(&self, tags: LayerTags) -> Vec<(usize, LayerTags)> {
        if DEBUG_PRINT {
            print!("LAYER_SRC_AREA_NAMES_CONTAINING_TAGS: tags: ");
            for sli in self.layers_containing_tags_src_lyrs(tags).iter() {
                print!("{}", sli.layer_tags());
            }
            print!("\n");
        }

        self.layers_containing_tags_src_lyrs(tags).iter().map(|sli|
            (sli.area_id(), sli.layer_tags())
        ).collect()
    }

    // // [FIXME] TODO: Create HashMap to index layer names.
    // pub fn layer_info_by_name(&self, name: String) -> Option<&LayerInfo> {
    //     let layers: Vec<&LayerInfo> = self.layers.iter().filter(|li| li.name() == name)
    //         .map(|li| li).collect();
    //     assert!(layers.len() <= 1, format!("Multiple ({}) layers match the name: {}",
    //         layers.len(), name));
    //     layers.get(0).map(|&li| li)
    // }

    pub fn layer_info(&self, lyr_id: usize) -> Option<&LayerInfo> {
        self.layers.by_index(lyr_id)
    }

    pub fn layer_info_by_name(&self, name: &str) -> Option<&LayerInfo> {
        self.layers.by_key(name)
    }


    /// [FIXME]: REMOVE/REDESIGN THIS: More than one layer can have the same
    /// slice id.
    pub fn slc_src_layer_info(&self, slc_id: u8, layer_tags: LayerTags) -> Option<&SourceLayerInfo> {
        let mut src_layer_info = Vec::with_capacity(8);
        let layer_info = self.layers_containing_tags(layer_tags);

        for lyr in layer_info {
            if lyr.depth() > 0 {
                for src_lyr in lyr.sources() {
                    if slc_id >= src_lyr.tar_slc_range().start
                        && slc_id < src_lyr.tar_slc_range().end
                    {
                        src_layer_info.push(src_lyr);
                    }
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
        self.layers.values().iter()
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

