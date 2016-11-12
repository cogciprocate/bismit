use std::fmt;
use std::ops::{Range};

use map::{LayerScheme, AreaScheme, AreaSchemeList, LayerMapSchemeList, LayerKind, DendriteKind,
    LayerMapKind, AxonKind};
use cmn::{self, CorticalDims, MapStore};
use map::{self, LayerTags,};
use thalamus::ExternalPathway;

const DEBUG_PRINT: bool = false;

// [FIXME]: Consolidate terminology and usage between source-layer layers (cellular)
// and source-area layers (axonal).
#[derive(Clone)]
pub struct LayerInfo {
    name: &'static str,
    tags: LayerTags,
    slc_range: Option<Range<u8>>,
    sources: Vec<SourceLayerInfo>,
    layer_map_kind: LayerMapKind,
    axn_kind: AxonKind,
    layer_scheme: LayerScheme,
    axn_count: u32,
    irregular_layer_dims: Option<CorticalDims>,
}

impl LayerInfo {
    /// [FIXME]: TODO: Break up, refactor, and optimize.
    /// [FIXME]: TODO: Create an error type enum just for map::Layer****.
    /// [FIXME]: TODO: Return result and get rid of panics, et al.
    pub fn new(layer_scheme: &LayerScheme, plmap_kind: LayerMapKind, pamap: &AreaScheme,
                area_sl: &AreaSchemeList, layer_map_sl: &LayerMapSchemeList,
                ext_paths: &MapStore<String, (ExternalPathway, Vec<LayerTags>)>,
                slc_total: u8) -> LayerInfo {
        let layer_scheme = layer_scheme.clone();
        let name = layer_scheme.name();
        let tags = layer_scheme.tags();
        let axn_kind = layer_scheme.axn_kind().expect("LayerInfo::new()");
        // let slc_range = layer_scheme.slc_idz()..(layer_scheme.slc_idz() + layer_scheme.depth());
        let mut sources = Vec::with_capacity(8);

        let mut next_slc_idz = slc_total;
        let mut axn_count = 0;

        let mut irregular_layer_dims: Option<CorticalDims> = None;
        let mut layer_debug: Vec<String> = Vec::new();
        let mut src_layer_debug: Vec<String> = Vec::new();

        if DEBUG_PRINT {
            layer_debug.push(format!("{mt}{mt}{mt}### LAYER: {:?}, next_slc_idz: {}, slc_total: {:?}",
                tags, next_slc_idz, slc_total, mt = cmn::MT));
        }

        // If layer is an input layer, add sources:
        if tags.contains(map::INPUT) {
            // Make sure this layer is axonal (cellular layers must not also
            // be input layers):
            match layer_scheme.kind() {
                &LayerKind::Axonal(_) => (),
                _ => panic!("Error assembling LayerInfo for '{}'. Layers containing \
                    'map::INPUT' must be 'AxonKind::Axonal'.", name),
            }

            // Assemble a list of layers, each given by an (area name, layer
            // tags) tuple which are either specific (not necessarily spatial)
            // and either feed-forward or feedback, or non-specific. This
            // should cover the gamut for the input layers of an area.
            let src_area_combos: Vec<(&'static str, LayerTags)> =
                pamap.get_aff_areas().iter().map(|&an| (an, map::FEEDBACK | map::SPECIFIC))
                    .chain(pamap.get_eff_areas().iter().map(|&an| (an, map::FEEDFORWARD | map::SPECIFIC)))
                .chain(pamap.get_aff_areas().iter().chain(pamap.get_eff_areas().iter())
                    .map(|&an| (an, map::NONSPECIFIC)))
                .collect();

            if DEBUG_PRINT {
                layer_debug.push(format!("{mt}{mt}{mt}{mt}### SRC_AREAS: {:?}",
                    src_area_combos, mt = cmn::MT));
            }

            // Assemble a list of sources for each input layer:
            //
            // For each potential source area (aff or eff):
            // - get that area's layers
            // - get the layers with a complimentary flag ('map::OUTPUT' in this case)
            //    - other tags identical
            // - filter out feedback from eff areas and feedforward from aff areas
            // - push what's left to sources
            //
            // Our layer must contain the flow direction flag corresponding
            // with the source area.
            //
            for (src_area_name, _) in src_area_combos.into_iter()
                    .filter(|&(_, src_layer_tag)| tags.contains(src_layer_tag))
            {
                // Get the source area map (proto):
                let src_pamap = area_sl.maps().get(src_area_name).expect("LayerInfo::new()");

                // Get the source layer map associated with this protoarea:
                let src_layer_map = &layer_map_sl[src_pamap.layer_map_name];

                // Get a list of layers with tags which are an i/o mirror
                // (input -> output, output -> input) of the tags for this
                // layer within this source area.
                let src_layers = src_layer_map.layers_with_tags(tags.mirror_io());

                if DEBUG_PRINT {
                    layer_debug.push(format!("{mt}{mt}{mt}{mt}{mt}### SRC_PROTOLAYERS: {:?}",
                        src_layers, mt = cmn::MT));
                }

                for src_layer in src_layers.iter() {
                    let (src_layer_dims, src_layer_axn_kind) = match src_layer_map.kind() {
                        // If the source layer is subcortical, we will be relying
                        // on the `ExternalPathway` associated with it to
                        // provide its dimensions.
                        &LayerMapKind::Subcortical => {
                            let src_area_name = src_area_name.to_owned();
                            let &(ref in_src, _) = ext_paths.by_key(&src_area_name)
                                .expect(&format!("LayerInfo::new(): Invalid input source key: \
                                    '{}'", src_area_name));
                            let in_src_layer = in_src.layer(src_layer.tags());
                            let in_src_layer_dims = in_src_layer.dims().expect(
                                &format!("LayerInfo::new(): External source layer dims for layer \
                                    '{}' in area '{}' are not set.", in_src_layer.name(),
                                    src_area_name)
                                ).clone();
                            (in_src_layer_dims, in_src_layer.axn_kind())
                        },
                        // If the source layer is cortical, we will give the
                        // layer dimensions depending on the source layer's
                        // size.
                        &LayerMapKind::Cortical => {
                            let depth = src_layer.depth().unwrap_or(cmn::DEFAULT_OUTPUT_LAYER_DEPTH);

                            let src_axn_kind = match src_layer.kind() {
                                &LayerKind::Axonal(ref ak) => {
                                    // [FIXME]: Make this a Result:
                                    assert!(ak.matches_tags(src_layer.tags()), "Incompatable layer \
                                        tags for layer: {:?}", src_layer);

                                    ak.clone()
                                },

                                &LayerKind::Cellular(_) => AxonKind::from_tags(src_layer.tags())
                                    .expect("LayerInfo::new(): Error determining axon kind"),
                                // _ => panic!("LayerInfo::new(): Unknown LayerKind."),
                            };

                            (src_pamap.dims().clone_with_depth(depth), src_axn_kind)
                        },
                    };

                    let tar_slc_range = next_slc_idz..(next_slc_idz + src_layer_dims.depth());

                    sources.push(SourceLayerInfo::new(src_area_name, src_layer_dims.clone(),
                        src_layer.tags(), src_layer_axn_kind, tar_slc_range.clone()));

                    if DEBUG_PRINT {
                        layer_debug.push(format!("{mt}{mt}{mt}{mt}{mt}{mt}### SOURCE_LAYER_INFO:\
                            (layer: '{}'): Adding source layer: \
                            src_area_name: '{}', src_layer.tags: '{}', src_layer_map.name: '{}', \
                            src_layer.name: '{}', tar_slc_range: '{:?}', depth: '{:?}'",
                            name, src_area_name, src_layer.tags(), src_layer_map.name,
                            src_layer.name(), tar_slc_range, src_layer.depth(), mt = cmn::MT));
                    }

                    src_layer_debug.push(format!("{mt}{mt}{mt}{mt}<{}>: {:?}: area: [\"{}\"], tags: {}",
                        src_layer.name(), tar_slc_range, src_area_name, src_layer.tags(), mt = cmn::MT));

                    // For (legacy) comparison purposes:
                    // layer_scheme.set_depth(src_layer_depth);

                    next_slc_idz += src_layer_dims.depth();
                    axn_count += src_layer_dims.cells();
                }
            }
        } else {
            // [NOTE]: This is a non-input layer.
            debug_assert!(!tags.contains(map::INPUT));

            // If this is a subcortical layer we need to use the dimensions
            // set by the `ExternalPathway` area instead of the dimensions of
            // the area. Thalamic output layers have irregular layer sizes.
            let columns = match plmap_kind {
                LayerMapKind::Subcortical => {
                    // If this is subcortical (previously thalamic), the
                    // OUTPUT flags should be set.
                    assert!(tags.contains(map::OUTPUT));
                    let pamap_name = pamap.name().to_owned();
                    let &(ref in_src, _) = ext_paths.by_key(&pamap_name)
                        .expect(&format!("LayerInfo::new(): Invalid input source key: \
                            '{}'", pamap.name()));
                    let in_src_layer = in_src.layer(tags);
                    let in_src_layer_dims = in_src_layer.dims().expect(&format!(
                        "LayerInfo::new(): External source layer dims for layer \
                        '{}' in area '{}' are not set.", in_src_layer.name(),
                        pamap.name()));
                    irregular_layer_dims = Some(in_src_layer_dims.clone());
                    in_src_layer_dims.columns()
                },
                LayerMapKind::Cortical => pamap.dims().columns(),
            };

            // [FIXME]: Get rid of the map::OUTPUT check and just default to 0.
            let layer_depth = match layer_scheme.depth() {
                Some(d) => d,
                None => if tags.contains(map::OUTPUT) { cmn::DEFAULT_OUTPUT_LAYER_DEPTH } else { 0 },
            };

            next_slc_idz += layer_depth;
            axn_count += columns * layer_depth as u32;
        }

        let ttl_slc_range = slc_total..next_slc_idz;
        let slc_range = if ttl_slc_range.len() > 0 { Some(ttl_slc_range) } else { None };
        sources.shrink_to_fit();

        println!("{mt}{mt}{mt}<{}>: {:?}: {}", name, slc_range, tags, mt = cmn::MT);

        // Print only the source layer info string:
        for dbg_string in src_layer_debug {
            println!("{}", &dbg_string);
        }

        if DEBUG_PRINT {
            // Print all of the other debug strings:
            for dbg_string in layer_debug {
                println!("{}", &dbg_string);
            }
        }

        if let Some(ref irr_dims) = irregular_layer_dims {
            debug_assert!(irr_dims.to_len() == axn_count as usize);
        }

        LayerInfo {
            name: name,
            tags: tags,
            slc_range: slc_range,
            sources: sources,
            layer_map_kind: plmap_kind,
            axn_kind: axn_kind,
            layer_scheme: layer_scheme,
            axn_count: axn_count,
            irregular_layer_dims: irregular_layer_dims,
        }
    }

    pub fn irregular_layer_dims(&self) -> Option<&CorticalDims> {
        self.irregular_layer_dims.as_ref()
    }

    pub fn thalamic_horizontal_axon_count(&self) -> Option<u32> {
        if self.layer_map_kind == LayerMapKind::Subcortical && self.axn_kind == AxonKind::Horizontal {
            debug_assert!(self.tags.contains(map::NS_OUT));
            Some(self.axn_count)
        } else {
            None
        }
    }

    pub fn src_lyr_names(&self, den_type: DendriteKind) -> Vec<&'static str> {
        self.layer_scheme.src_lyr_names(den_type)
    }

    pub fn dst_src_lyrs(&self) -> Vec<Vec<&'static str>> {
        let layers_by_tuft = match self.layer_scheme.kind() {
            &LayerKind::Cellular(ref cell_scheme) => cell_scheme.den_dst_src_lyrs.clone(),
            _ => None,
        };

        match layers_by_tuft {
            Some(v) => v,
            None => Vec::with_capacity(0),
        }
    }

    pub fn depth(&self) -> u8 {
        match self.slc_range {
            Some(ref r) => r.len() as u8,
            None => 0,
        }
    }

    #[inline] pub fn name(&self) -> &'static str { self.name }
    #[inline] pub fn tags(&self) -> LayerTags { self.tags }
    #[inline] pub fn kind(&self) -> &LayerKind { self.layer_scheme.kind() }
    #[inline] pub fn sources(&self) -> &[SourceLayerInfo]  { &self.sources }
    #[inline] pub fn axn_count(&self) -> u32 { self.axn_count }
    #[inline] pub fn axn_kind(&self) -> AxonKind { self.axn_kind.clone() }
    #[inline] pub fn layer_map_kind(&self) -> LayerMapKind { self.layer_map_kind.clone() }
    #[inline] pub fn slc_range(&self) -> Option<&Range<u8>> { self.slc_range.as_ref() }
}

impl fmt::Display for LayerInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(format_args!("{:#?}", self))
    }
}

impl fmt::Debug for LayerInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("LayerInfo")
            .field("name", &self.name)
            .field("tags", &self.tags.to_string())
            .field("slc_range", &self.slc_range)
            .field("sources", &self.sources)
            .field("layer_map_kind", &self.layer_map_kind)
            .field("axn_kind", &self.axn_kind)
            .field("layer_scheme", &self.layer_scheme)
            .field("axn_count", &self.axn_count)
            .field("irregular_layer_dims", &self.irregular_layer_dims)
            .finish()
    }
}



#[derive(Clone)]
pub struct SourceLayerInfo {
    area_name: &'static str,
    dims: CorticalDims,
    tags: LayerTags,
    axn_kind: AxonKind,
    tar_slc_range: Range<u8>,
}

impl SourceLayerInfo {
    #[inline]
    pub fn new(src_area_name: &'static str, src_layer_dims: CorticalDims, src_layer_tags: LayerTags,
                src_axn_kind: AxonKind, tar_slc_range: Range<u8>) -> SourceLayerInfo
    {
        assert!(tar_slc_range.len() == src_layer_dims.depth() as usize);

        SourceLayerInfo {
            area_name: src_area_name,
            dims: src_layer_dims,
            tags: src_layer_tags,
            axn_kind: src_axn_kind,
            tar_slc_range: tar_slc_range,
        }
    }

    #[inline] pub fn area_name(&self) -> &'static str { self.area_name }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn axn_count(&self) -> u32 { self.dims().cells() }
    #[inline] pub fn tags(&self) -> LayerTags { self.tags }
    #[inline] pub fn axn_kind(&self) -> AxonKind { self.axn_kind.clone() }
    #[inline] pub fn tar_slc_range(&self) -> &Range<u8> { &self.tar_slc_range }
}

impl fmt::Debug for SourceLayerInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("LayerInfo")
            .field("area_name", &self.area_name)
            .field("dims", &self.dims)
            .field("tags", &self.tags.to_string())
            .field("axn_kind", &self.axn_kind)
            .field("tar_slc_range", &self.tar_slc_range)
            .finish()
    }
}