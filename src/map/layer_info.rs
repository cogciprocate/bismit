use std::fmt;
use std::ops::{Range};

use map::{LayerScheme, AreaScheme, AreaSchemeList, LayerMapSchemeList, LayerMapScheme,
    LayerKind, LayerMapKind, AxonTopology, AxonDomain, AxonTags, InputTrack, LayerAddress,
    TuftSourceLayer, LayerTags, AxonSignature};
use cmn::{self, CorticalDims};
use subcortex::Subcortex;

const DEBUG_PRINT: bool = false;


// Assembles a list of source layers for this input layer.
//
// For each source area (aff, eff, or other) Store the layers
// with matching criteria (InputTrack and AxonTags) into a
// unified list.
fn matching_source_layers<'a>(area_sch: &'a AreaScheme, area_sch_list: &'a AreaSchemeList,
        layer_map_sch_list: &'a LayerMapSchemeList, input_filters: &'a Vec<AxonSignature>)
        -> Vec<(&'a LayerScheme, AxonSignature, Option<AxonTags>, &'a LayerMapScheme, &'a AreaScheme)>
{
    let mut matching_source_layers = Vec::with_capacity(16);

    for filter_sig in input_filters.iter() {
        debug_assert!(filter_sig.is_input());
        let candidate_areas: Vec<(&str, Option<Vec<(AxonTags, AxonTags)>>)> =
                match *filter_sig.track().unwrap()
        {
            InputTrack::Afferent => {
                area_sch.get_eff_areas().iter()
                    .map(|&an| (an, None)).collect()
            },
            InputTrack::Efferent => {
                area_sch.get_aff_areas().iter()
                    .map(|&an| (an, None)).collect()
            },
            InputTrack::Other => {
                area_sch.get_other_areas().clone()
            },
        };

        for (area_name, axon_tag_masqs) in candidate_areas {
            // Get the source area map scheme:
            let src_area_sch = area_sch_list.get_area_by_key(area_name)
                .expect("LayerInfo::new()");

            // Get the source layer map scheme associated with the source area:
            let src_lyr_map_sch = &layer_map_sch_list[src_area_sch.layer_map_name()];

            // Make a list of output layers with matching axon tags for this filter:
            let mut src_layers: Vec<(&LayerScheme, AxonSignature, Option<AxonTags>)> = Vec::with_capacity(8);

            match axon_tag_masqs {
                Some(masqs) => {
                    for (orig, repl) in masqs {
                        // If the replacement tag of a masquerade matches the
                        // current filter's axon tag, use the original tag to
                        // search for matching source layers. This is just
                        // performing the masquerade in reverse order with the
                        // same effect.
                        if repl == *filter_sig.tags() {
                            let matching_lyrs = src_lyr_map_sch.output_layers_with_axon_tags(&orig);

                            for matching_lyr in matching_lyrs.into_iter() {
                                src_layers.push((
                                    matching_lyr,
                                    (filter_sig.track().unwrap().clone(), repl.clone()).into(),
                                    Some(orig.clone())
                                ));
                            }
                        }
                    }
                },
                None => {
                    let matching_lyrs = src_lyr_map_sch.output_layers_with_axon_tags(filter_sig.tags());

                    for matching_lyr in matching_lyrs.into_iter() {
                        src_layers.push((matching_lyr, filter_sig.clone(), None));
                    }
                }
            }

            // Add the matching source layers to our list of sources:
            for (matching_lyr, sig, masq_orig_axn_tags) in src_layers.into_iter() {
                matching_source_layers.push((matching_lyr, sig, masq_orig_axn_tags,
                    src_lyr_map_sch, src_area_sch));
            }
        }
    }

    matching_source_layers.shrink_to_fit();
    matching_source_layers
}

// [FIXME]: Consolidate terminology and usage between source-layer layers (cellular)
// and source-area layers (axonal).
#[derive(Clone)]
pub struct LayerInfo {
    layer_addr: LayerAddress,
    name: &'static str,
    layer_tags: LayerTags,
    axon_domain: AxonDomain,
    slc_range: Option<Range<usize>>,
    sources: Vec<SourceLayerInfo>,
    layer_map_kind: LayerMapKind,
    axn_topology: AxonTopology,
    layer_scheme: LayerScheme,
    ttl_axn_count: u32,
    irregular_layer_dims: Option<CorticalDims>,
}

impl LayerInfo {
    /// [FIXME]: TODO: Create an error type enum just for map::Layer****.
    /// [FIXME]: TODO: Return result and get rid of panics, et al.
    /// [FIXME]: Refactor much of this into sub-functions.
    pub fn new(layer_id: usize, layer_scheme: &LayerScheme, plmap_kind: LayerMapKind,
            area_sch: &AreaScheme, area_sch_list: &AreaSchemeList,
            layer_map_sch_list: &LayerMapSchemeList, subcortex: &Subcortex, slc_total: u8)
            -> LayerInfo {
        let layer_scheme = layer_scheme.clone();
        let name = layer_scheme.name();
        let layer_tags = layer_scheme.layer_tags();
        let axon_domain = layer_scheme.axn_domain().clone();
        let axn_topology = layer_scheme.axn_topology();
        let mut sources: Vec<SourceLayerInfo> = Vec::with_capacity(8);

        let mut next_slc_idz = slc_total as usize;
        let mut ttl_axn_count = 0;

        let mut irregular_layer_dims: Option<CorticalDims> = None;
        let mut layer_debug: Vec<String> = Vec::new();
        let mut src_layer_debug: Vec<String> = Vec::new();

        if DEBUG_PRINT {
            layer_debug.push(format!("{mt}{mt}{mt}### LAYER: {:?}, next_slc_idz: {}, slc_total: {:?}",
                layer_tags, next_slc_idz, slc_total, mt = cmn::MT));
        }

        match axon_domain {
            /*=============================================================================
            ===============================================================================
            =============================================================================*/
            AxonDomain::Input(ref input_filters) => {
                // Make sure this layer is axonal (cellular layers must not also
                // be input layers):
                match *layer_scheme.kind() {
                    LayerKind::Axonal(_) => (),
                    _ => panic!("Error assembling LayerInfo for '{}'. Input layers \
                        must be 'AxonTopology::Axonal'.", name),
                }

                // Assemble a list of source layers for this input layer:
                let matching_source_layers = matching_source_layers(area_sch, area_sch_list,
                    layer_map_sch_list, input_filters);

                // Create a `SourceLayerInfo` for each matching layer:
                for (src_layer, sig, masq_orig_axn_tags,
                        src_lyr_map_sch, src_area_sch) in matching_source_layers.into_iter()
                {
                    let src_area_name = src_area_sch.name();
                    let src_area_id = src_area_sch.area_id();
                    let src_lyr_addr = LayerAddress::new(src_area_id, src_layer.layer_id());

                    /*=============================================================================
                    ===============================================================================
                    =============================================================================*/

                    let (src_layer_dims, src_layer_axn_topology) = match src_lyr_map_sch.kind() {
                        // If the source layer is subcortical, we will be relying
                        // on the `InputGenerator` associated with it to
                        // provide its dimensions.
                        &LayerMapKind::Subcortical => {
                            let subcortical_nucleus = subcortex.by_key(src_area_name)
                                .expect(&format!("LayerInfo::new(): Invalid input source key: \
                                    \"{}\"", src_area_name));

                            let sub_layer = subcortical_nucleus.layer(src_lyr_addr.clone())
                                .expect(&format!("LayerInfo::new(): Invalid addr: {:?}", src_lyr_addr));;

                            let sub_layer_dims = sub_layer.dims().clone();
                            assert!(sub_layer_dims.are_at_least(&CorticalDims::from((1, 1, 0))),
                                "Subcortical dims for area \"{}\" are zero.", src_area_name);

                            (sub_layer_dims, sub_layer.axon_topology())
                        },
                        // If the source layer is cortical, we will give the
                        // layer dimensions depending on the source layer's
                        // size.
                        &LayerMapKind::Cortical => {
                            let depth = src_layer.depth().unwrap_or(cmn::DEFAULT_OUTPUT_LAYER_DEPTH);

                            let src_axn_topology = match src_layer.kind() {
                                &LayerKind::Axonal(ref ak) => ak.clone(),
                                &LayerKind::Cellular(_) => AxonTopology::Spatial
                            };

                            (src_area_sch.dims().clone_with_depth(depth), src_axn_topology)

                        },
                    };

                    /*=============================================================================
                    ===============================================================================
                    =============================================================================*/

                    let tar_slc_range = next_slc_idz..(next_slc_idz + src_layer_dims.depth() as usize);

                    sources.push(SourceLayerInfo::new(src_lyr_addr, src_layer_dims.clone(),
                        src_layer.layer_tags(), src_layer_axn_topology, sig, masq_orig_axn_tags,
                        tar_slc_range.clone(), ));

                    if DEBUG_PRINT {
                        layer_debug.push(format!("{mt}{mt}{mt}{mt}{mt}{mt}### SOURCE_LAYER_INFO:\
                            (layer: '{}'): Adding source layer: \
                            src_area_name: '{}', src_layer.tags: '{}', src_lyr_map_sch.name: '{}', \
                            src_layer.name: '{}', tar_slc_range: '{:?}', depth: '{:?}'",
                            name, src_area_name, src_layer.layer_tags(), src_lyr_map_sch.name(),
                            src_layer.name(), tar_slc_range, src_layer.depth(), mt = cmn::MT));
                    }

                    src_layer_debug.push(format!("{mt}{mt}{mt}{mt}<{}>: {:?}: area: [\"{}\"], tags: {}",
                        src_layer.name(), tar_slc_range, src_area_name, src_layer.layer_tags(), mt = cmn::MT));

                    next_slc_idz += src_layer_dims.depth() as usize;
                    ttl_axn_count += src_layer_dims.cells();
                }

                // Double check that the total source layer axon count matches up:
                assert!(sources.iter().map(|sli| sli.dims().cells()).sum::<u32>() == ttl_axn_count);
            },
            /*=============================================================================
            ===============================================================================
            =============================================================================*/
            AxonDomain::Output(/*ref axon_tags*/ _) => {

                // If this is a subcortical layer we need to use the
                // dimensions set by the `SubcorticalNucleusLayer` instead of
                // the dimensions of the `SubcorticalNucleus` area.
                // Subcortical output layers have irregular layer sizes.
                let columns = match plmap_kind {
                    LayerMapKind::Subcortical => {
                        let area_sch_name = area_sch.name().to_owned();

                        let subcortical_nucleus = subcortex.by_key(&area_sch_name)
                            .expect(&format!("LayerInfo::new(): Invalid input source key: \
                                \"{}\"", area_sch.name()));

                        let sub_lyr_addr = LayerAddress::new(area_sch.area_id(), layer_id);

                        let sub_layer = subcortical_nucleus.layer(sub_lyr_addr)
                            .expect(&format!("LayerInfo::new(): Invalid addr: {:?}", sub_lyr_addr));

                        let sub_layer_dims = sub_layer.dims();
                        assert!(sub_layer_dims.are_at_least(&CorticalDims::from((1, 1, 0))),
                            "Subcortical dims for area \"{}\" are zero.", area_sch_name);

                        irregular_layer_dims = Some(sub_layer_dims.clone());
                        sub_layer_dims.columns()
                    },
                    LayerMapKind::Cortical => area_sch.dims().columns(),
                };

                // If the depth is not set, default to 0;
                let layer_depth = layer_scheme.depth().unwrap_or(0);
                next_slc_idz += layer_depth as usize;
                ttl_axn_count += columns * layer_depth as u32;
            },
            /*=============================================================================
            ===============================================================================
            =============================================================================*/
            AxonDomain::Local => {
                let columns = area_sch.dims().columns();
                let layer_depth = layer_scheme.depth().unwrap_or(0);
                next_slc_idz += layer_depth as usize;
                ttl_axn_count += columns * layer_depth as u32;
            }
        }

        let ttl_slc_range = (slc_total as usize)..next_slc_idz;
        let slc_range = if ttl_slc_range.len() > 0 { Some(ttl_slc_range) } else { None };
        sources.shrink_to_fit();

        println!("{mt}{mt}{mt}[{}] <{}>: {:?}: {}", layer_id, name, slc_range, layer_tags, mt = cmn::MT);

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
            debug_assert!(irr_dims.to_len() == ttl_axn_count as usize);
        }

        LayerInfo {
            layer_addr: LayerAddress::new(area_sch.area_id(), layer_id),
            name: name,
            layer_tags: layer_tags,
            axon_domain: axon_domain,
            slc_range: slc_range,
            sources: sources,
            layer_map_kind: plmap_kind,
            axn_topology: axn_topology,
            layer_scheme: layer_scheme,
            ttl_axn_count: ttl_axn_count,
            irregular_layer_dims: irregular_layer_dims,
        }
    }

    pub fn src_lyr_old(&self, area_id: usize, tar_slc_range: Range<usize>)
            -> Option<&SourceLayerInfo>
    {
        self.sources.iter().find(|sli| sli.area_id() == area_id &&
            sli.tar_slc_range == tar_slc_range)
    }

    pub fn src_lyr(&self, src_layer_addr: &LayerAddress) -> Option<&SourceLayerInfo> {
        self.sources.iter().find(|sli| sli.layer_addr() == src_layer_addr)
    }

    pub fn irregular_layer_dims(&self) -> Option<&CorticalDims> {
        self.irregular_layer_dims.as_ref()
    }

    pub fn cel_tft_src_lyrs(&self, tft_id: usize) -> &[TuftSourceLayer] {
        match *self.layer_scheme.kind() {
            LayerKind::Cellular(ref cell_scheme) => {
                let tft = cell_scheme.tft_schemes().get(tft_id).expect(&format!(
                    "Tuft with id: '{}' for layer: '{}' not found.", tft_id, self.layer_id()));

                tft.src_lyrs()
            },
            _ => panic!(format!("LayerScheme '{}' is not 'Cellular'.", self.name)),
        }
    }

    pub fn depth(&self) -> u8 {
        match self.slc_range {
            Some(ref r) => r.len() as u8,
            None => 0,
        }
    }

    #[inline] pub fn layer_addr(&self) -> &LayerAddress { &self.layer_addr }
    #[inline] pub fn layer_id(&self) -> usize { self.layer_addr.layer_id() }
    #[inline] pub fn name(&self) -> &'static str { self.name }
    #[inline] pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    #[inline] pub fn kind(&self) -> &LayerKind { self.layer_scheme.kind() }
    #[inline] pub fn axn_domain(&self) -> &AxonDomain { self.layer_scheme.axn_domain() }
    #[inline] pub fn is_input(&self) -> bool { self.layer_scheme.axn_domain().is_input() }
    #[inline] pub fn is_output(&self) -> bool { self.layer_scheme.axn_domain().is_output() }
    #[inline] pub fn sources(&self) -> &[SourceLayerInfo]  { &self.sources }
    #[inline] pub fn ttl_axn_count(&self) -> u32 { self.ttl_axn_count }
    #[inline] pub fn axn_topology(&self) -> AxonTopology { self.axn_topology.clone() }
    #[inline] pub fn layer_map_kind(&self) -> LayerMapKind { self.layer_map_kind.clone() }
    #[inline] pub fn slc_range(&self) -> Option<&Range<usize>> { self.slc_range.as_ref() }
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
            .field("layer_tags", &self.layer_tags.to_string())
            .field("slc_range", &self.slc_range)
            .field("sources", &self.sources)
            .field("layer_map_kind", &self.layer_map_kind)
            .field("axn_topology", &self.axn_topology)
            .field("layer_scheme", &self.layer_scheme)
            .field("ttl_axn_count", &self.ttl_axn_count)
            .field("irregular_layer_dims", &self.irregular_layer_dims)
            .finish()
    }
}



#[derive(Clone)]
pub struct SourceLayerInfo {
    layer_addr: LayerAddress,
    dims: CorticalDims,
    layer_tags: LayerTags,
    axn_topology: AxonTopology,
    input_sig: AxonSignature,
    masq_orig_axn_tags: Option<AxonTags>,
    // Absolute target slice range (not level-relative):
    tar_slc_range: Range<usize>,
}

impl SourceLayerInfo {
    #[inline]
    pub fn new(src_lyr_addr: LayerAddress, src_layer_dims: CorticalDims, src_layer_tags: LayerTags,
            src_axn_topology: AxonTopology, input_sig: AxonSignature,
            masq_orig_axn_tags: Option<AxonTags>,
            tar_slc_range: Range<usize>) -> SourceLayerInfo
    {
        assert!(input_sig.is_input());
        assert!(tar_slc_range.len() == src_layer_dims.depth() as usize);

        SourceLayerInfo {
            layer_addr: src_lyr_addr,
            dims: src_layer_dims,
            layer_tags: src_layer_tags,
            axn_topology: src_axn_topology,
            input_sig: input_sig,
            masq_orig_axn_tags: masq_orig_axn_tags,
            tar_slc_range: tar_slc_range,
        }
    }

    #[inline] pub fn area_id<'a>(&'a self) -> usize { self.layer_addr.area_id() }
    #[inline] pub fn layer_addr(&self) -> &LayerAddress { &self.layer_addr }
    #[inline] pub fn dims(&self) -> &CorticalDims { &self.dims }
    #[inline] pub fn axn_count(&self) -> u32 { self.dims().cells() }
    #[inline] pub fn layer_tags(&self) -> LayerTags { self.layer_tags }
    #[inline] pub fn axn_topology(&self) -> AxonTopology { self.axn_topology.clone() }
    #[inline] pub fn input_track(&self) -> &InputTrack { &self.input_sig.track().as_ref().unwrap() }
    #[inline] pub fn axn_tags(&self) -> &AxonTags { &self.input_sig.tags() }
    #[inline] pub fn input_sig(&self) -> &AxonSignature { &self.input_sig }
    #[inline] pub fn masq_orig_axn_tags(&self) -> Option<&AxonTags> { self.masq_orig_axn_tags.as_ref() }
    #[inline] pub fn tar_slc_range(&self) -> &Range<usize> { &self.tar_slc_range }
}

impl fmt::Debug for SourceLayerInfo {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("LayerInfo")
            .field("layer_addr", &self.layer_addr)
            .field("dims", &self.dims)
            .field("layer_tags", &self.layer_tags.to_string())
            .field("axn_topology", &self.axn_topology)
            .field("input_sig", &self.input_sig)
            .field("masq_orig_axn_tags", &self.masq_orig_axn_tags)
            .field("tar_slc_range", &self.tar_slc_range)
            .finish()
    }
}
