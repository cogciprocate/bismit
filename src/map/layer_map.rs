use std::collections::{BTreeMap, HashSet};
use std::ops::{Range};
use std::slice::{Iter};
use map::{AreaScheme, AreaSchemeList, LayerMapSchemeList, LayerMapKind, AxonDomainRoute,
    AxonDomain, AxonSignature};
use cmn::{self, MapStore, CmnResult};
use map::{LayerTags, LayerInfo, SourceLayerInfo};
use subcortex::Subcortex;

const PRNT: bool = false;


/// Stores unique fingerprints for each interconnecting layer or sub-layer.
///
struct AxonDomainCache {
    cache: HashSet<(AxonDomainRoute, AxonSignature)>,
}

impl AxonDomainCache {
    pub fn new() -> AxonDomainCache {
        AxonDomainCache { cache: HashSet::with_capacity(64) }
    }

    /// Adds the unique fingerprint[s] for the layer or sub-layers defined
    /// within `domain` to the cache and returns an error if a duplicate is
    /// found.
    ///
    pub fn add(&mut self, domain: &AxonDomain) -> CmnResult<()> {
        match *domain {
            AxonDomain::Input(ref sigs) => {
                for sig in sigs {
                    debug_assert!(sig.is_input());
                    if !self.cache.insert((AxonDomainRoute::Input, sig.clone())) {
                        return Err(format!("Two input layers within the same layer map have the same axon \
                            signature ({:?}).", sig).into())
                    }
                }
            },
            AxonDomain::Output(ref sig) => {
                debug_assert!(sig.is_output());
                if !self.cache.insert((AxonDomainRoute::Output, sig.clone())) {
                    return Err(format!("Two output layers within the same layer map have the same axon \
                        domain: '{:?}'.", domain).into())
                }
            },
            AxonDomain::Local => (),
        }

        Ok(())
    }
}


#[derive(Clone, Debug)]
pub struct LayerMap {
    area_name: &'static str,
    area_id: usize,
    layers: MapStore<String, LayerInfo>,
    depth: u8,
    kind: LayerMapKind,
}

impl LayerMap {
    pub fn new(area_sch: &AreaScheme, layer_map_sl: &LayerMapSchemeList, area_sl: &AreaSchemeList,
            subcortex: &Subcortex) -> CmnResult<LayerMap> {
        println!("{mt}{mt}LAYERMAP::NEW(): Assembling layer map for area \"{}\"...",
            area_sch.name(), mt = cmn::MT);
        println!("{mt}{mt}{mt}[Layer ID] <Layer Name>: Option(Slice Range): {{ Layer Tags }}",
            mt = cmn::MT);
        print!("\n");

        let lm_scheme = layer_map_sl[area_sch.layer_map_name()].clone();

        let mut layers = MapStore::with_capacity(lm_scheme.layers().len());
        let mut slc_total = 0u8;
        let mut domain_cache = AxonDomainCache::new();

        for (layer_id, ls) in lm_scheme.layers().iter().enumerate() {
            assert!(ls.layer_id() == layer_id);
            let new_layer = LayerInfo::new(layer_id, ls, lm_scheme.kind().clone(), area_sch,
                area_sl, layer_map_sl, subcortex, slc_total);

            // Check for duplicate input or output domains:
            domain_cache.add(new_layer.axon_domain())?;

            slc_total += new_layer.depth();
            layers.insert(ls.name().to_owned(), new_layer);
            assert!(layers[layer_id].layer_addr().layer_id() == layer_id);
        }

        let lm = LayerMap {
            area_name: area_sch.name(),
            area_id: area_sch.area_id(),
            layers: layers,
            depth: slc_total,
            kind: lm_scheme.kind().clone()
        };

        Ok(lm)
    }

    pub fn slc_map(&self) -> BTreeMap<u8, &LayerInfo> {
        let mut slc_map = BTreeMap::new();
        let mut slc_id_count = 0;

        for layer in self.layers.values().iter() {

            if let Some(slc_range) = layer.slc_range() {
                for slc_id in slc_range.clone() {
                    debug_assert_eq!(slc_id_count, slc_id);

                    if slc_map.insert(slc_id as u8, layer).is_some() {
                        panic!("LayerMap::slc_map(): Duplicate slices found in LayerMap: \
                            layer: '{}', slc_id: '{}'.", layer.name(), slc_id);
                    }

                    slc_id_count = slc_id + 1;
                }
            }
        }

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

    /// Returns the slice ranges associated with matching layers.
    pub fn layers_containing_tags_slc_range(&self, layer_tags: LayerTags) -> Vec<Range<usize>> {
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
        if PRNT {
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

    pub fn layer_info(&self, lyr_id: usize) -> Option<&LayerInfo> {
        self.layers.by_index(lyr_id)
    }

    pub fn layer_info_by_name(&self, name: &str) -> Option<&LayerInfo> {
        self.layers.by_key(name)
    }

    /// Returns the layer matching the specified input track and axon tags.
    ///
    /// Include `track_opt` for input layers.
    ///
    //
    // * TODO: Create a map of track/tags -> layer_id instead of this lookup.
    //
    // pub fn layer_info_by_tags(&self, track_opt: Option<&InputTrack>, tags: &AxonTags)
    pub fn layer_info_by_sig(&self, sig: &AxonSignature) -> Option<(&LayerInfo)> {
        let mut matching_layers = Vec::with_capacity(1);

        for lyr_info in self.layers.values().iter() {
            match *lyr_info.axon_domain() {
                AxonDomain::Input(ref filter_sigs) => {
                    // If `track_opt` is not defined, caller is requesting an output layer.
                    // let track = match sig.track() {
                    //     Some(it) => it,
                    //     None => continue,
                    // };
                    if sig.is_output() { continue }

                    for filter_sig in filter_sigs {
                        if filter_sig == sig {
                            matching_layers.push(lyr_info);
                        }
                    }
                },
                AxonDomain::Output(ref lyr_info_sig) => {
                    if sig.is_input() { continue }

                    if lyr_info_sig == sig {
                        matching_layers.push(lyr_info);
                    }
                }
                _ => (),
            }
        }

        match matching_layers.len() {
            0 => None,
            1 => {
                if sig.track().is_some() {
                    debug_assert!(matching_layers[0].layer_addr() ==
                        self.src_layer_info_by_sig(sig).unwrap().1.layer_addr());
                }

                Some(matching_layers[0])
            },
            _ => panic!("Internal error: Duplicate axon signatures ({:?}) found within the \
                layer map for area: \"{}\".", sig, self.area_name),
        }
    }

    // pub fn src_layer_info_by_sig(&self, track: &InputTrack, tags: &AxonTags)
    pub fn src_layer_info_by_sig(&self, sig: &AxonSignature)
            -> Option<(&SourceLayerInfo, &LayerInfo)>
    {
        let mut matching_layers = Vec::with_capacity(1);

        for lyr_info in self.layers.values().iter() {
            for src_lyr_info in lyr_info.sources().iter() {
                if src_lyr_info.input_sig() == sig {
                    matching_layers.push((src_lyr_info, lyr_info))
                }
            }
        }

        match matching_layers.len() {
            0 => None,
            1 => Some(matching_layers[0]),
            _ => panic!("Internal error: Duplicate axon signatures ({:?}) found within the \
                layer map for area: \"{}\".", sig, self.area_name),
        }
    }

    /// [FIXME]: REMOVE/REDESIGN THIS: More than one layer can have the same
    /// slice id.
    pub fn slc_src_layer_info(&self, slc_id: u8, layer_tags: LayerTags) -> Option<&SourceLayerInfo> {
        let mut src_layer_info = Vec::with_capacity(8);
        let layer_info = self.layers_containing_tags(layer_tags);

        for lyr in layer_info {
            if lyr.depth() > 0 {
                for src_lyr in lyr.sources() {
                    if (slc_id as usize) >= src_lyr.tar_slc_range().start
                        && (slc_id as usize) < src_lyr.tar_slc_range().end
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

    #[inline] pub fn layers(&self) -> &MapStore<String, LayerInfo> { &self.layers }
    #[inline] pub fn iter(&self) -> Iter<LayerInfo> { self.layers.values().iter() }
    #[inline] pub fn region_kind(&self) -> &LayerMapKind { &self.kind }
    #[inline] pub fn depth(&self) -> u8 { self.depth }
}
