// use std::iter;
use std::collections::HashMap;
// use std::collections::hash_map::IterMut;
// use std::hash::BuildHasherDefault;
// use twox_hash::XxHash;
use map::{self, LayerTags};
use cmn::{self, CorticalDims, TractFrameMut};
use ocl::{EventList};
use map::{AreaScheme, InputScheme, LayerMapScheme, LayerScheme, AxonKind};
use encode::{IdxStreamer, GlyphSequences};

// pub type ExternalSourceMap = HashMap<String, ExternalSource>;

/// A highway for input.
///
/// Returns a 3-array because I didn't want to bother with generics or enums
/// for the moment.
///
pub trait ExternalSourceTract {
    fn read_into(&mut self, tract_frame: &mut TractFrameMut, tags: LayerTags) 
        -> [usize; 3];
    fn cycle_next(&mut self);
}

#[allow(unused_variables)]
pub enum ExternalSourceKind {
    None,
    World,
    Stripes { stripe_size: usize, zeros_first: bool },
    Hexballs { edge_size: usize, invert: bool, fill: bool },
    Exp1,
    IdxStreamer(Box<ExternalSourceTract>),
    GlyphSequences(Box<GlyphSequences>),
    Custom(Box<ExternalSourceTract>),
}


pub struct ExternalSourceLayer {
    layer_name: &'static str,
    layer_tags: LayerTags,    
    axn_kind: AxonKind,
    dims: Option<CorticalDims>,
}

impl ExternalSourceLayer {
    pub fn set_dims(&mut self, dims: Option<CorticalDims>) {
        self.dims = dims;
    }

    pub fn name(&self) -> &'static str {
        self.layer_name
    }

    pub fn tags(&self) -> LayerTags {
        self.layer_tags
    }

    pub fn axn_kind(&self) -> AxonKind {
        self.axn_kind.clone()
    }

    pub fn dims(&self) -> Option<&CorticalDims> {
        self.dims.as_ref()
    }
}


/// An input source.
///
// [NOTE]: To implement multiple layers from a single input source:
// - Must pass layer count to the input 'generator' and have it accept a
//   multi-headed mutable slice when cycled.
pub struct ExternalSource {
    area_name: String,
    src_kind: ExternalSourceKind,
    // layers: HashMap<LayerTags, ExternalSourceLayer, BuildHasherDefault<XxHash>>,
    layers: HashMap<LayerTags, ExternalSourceLayer>,
}

impl ExternalSource {
    // [FIXME] Determine (or have passed in) the layer depth corresponding to this source.
    pub fn new(pamap: &AreaScheme, plmap: &LayerMapScheme) -> ExternalSource {
        let p_layers: Vec<&LayerScheme> = plmap.layers().iter().map(|(_, pl)| pl).collect();

        assert!(pamap.get_input().layer_count() == p_layers.len(), "ExternalSource::new(): \
            Inputs for 'Protoarea' ({}) must equal layers in 'LayerMapScheme' ({}).",
            pamap.get_input().layer_count(), p_layers.len());

        // let mut layers = HashMap::with_capacity_and_hasher(4, BuildHasherDefault::default());
        let mut layers = HashMap::with_capacity(4);
        let mut layer_tags_list = Vec::with_capacity(4);

        for p_layer in p_layers.into_iter() {
            let layer_name = p_layer.name();
            let layer_tags = p_layer.tags();
            let axn_kind = p_layer.kind().axn_kind().expect("ExternalSource::new(): ExternalSource layer \
                must be 'LayerKind::Axonal(_)'.");
            let layer_depth = p_layer.depth().unwrap_or(cmn::DEFAULT_OUTPUT_LAYER_DEPTH);

            let dims = if layer_tags.contains(map::SPATIAL) {
                assert_eq!(axn_kind, AxonKind::Spatial);
                Some(pamap.dims.clone_with_depth(layer_depth))
            } else {
                assert!(layer_tags.contains(map::HORIZONTAL));
                assert_eq!(axn_kind, AxonKind::Horizontal);
                None
            };
            

            assert!(layer_tags.contains(map::OUTPUT), "ExternalSource::new(): External ('Thalamic') areas \
                must have a single layer with an 'OUTPUT' tag. [area: '{}', layer map: '{}']", 
                pamap.name(), plmap.name());     

            layer_tags_list.push(layer_tags);       

            layers.insert(layer_tags ,ExternalSourceLayer {
                layer_name: layer_name,
                layer_tags: layer_tags,
                axn_kind: axn_kind,
                dims: dims,
            });
        }

        let src_kind = match pamap.get_input().clone() {
            InputScheme::IdxStreamer { file_name, cyc_per, scale, loop_frames } => {
                assert_eq!(layers.len(), 1);
                let mut is = IdxStreamer::new(layers[&layer_tags_list[0]].dims()
                    .expect("ExternalSource::new(): Layer dims not set properly.").clone(), 
                    file_name, cyc_per, scale);

                if loop_frames > 0 {
                    is = is.loop_frames(loop_frames);
                }
                ExternalSourceKind::IdxStreamer(Box::new(is))
            },
            InputScheme::GlyphSequences { seq_lens, seq_count, scale, hrz_dims } => {
                let gs = GlyphSequences::new(&mut layers, seq_lens, seq_count, scale, hrz_dims);
                ExternalSourceKind::GlyphSequences(Box::new(gs))
            },
            InputScheme::None | InputScheme::Zeros => ExternalSourceKind::None,
            pi @ _ => panic!("\nExternalSource::new(): Input type: '{:?}' not yet supported.", pi),
        };

        ExternalSource {
            area_name: pamap.name.to_owned(),
            layers: layers,
            src_kind: src_kind,           
        }
    }

    /// Reads input data into a tract.
    pub fn read_into(&mut self, tags: LayerTags, tract: &mut [u8], _: &mut EventList) {
        let dims = self.layers[&tags].dims().expect(&format!("Dimensions don't exist for \
            external input area: \"{}\", tags: '{:?}' ", self.area_name, tags));

        debug_assert!(dims.to_len() == tract.len(), "Dimensional mismatch for external input \
            area: \"{}\", tags: '{:?}', layer dims: {:?}, tract len: {}", self.area_name, tags,
            dims, tract.len());

        let mut tf = TractFrameMut::new(tract, dims);

        // '.cycle()' returns a [usize; 3], not sure what we're going to do with it.
        let _ = match self.src_kind {
            ExternalSourceKind::IdxStreamer(ref mut es) | ExternalSourceKind::Custom(ref mut es) => {
                es.read_into(&mut tf, tags)
            },
            ExternalSourceKind::GlyphSequences(ref mut gs) => {
                gs.read_into(&mut tf, tags)
            },
            _ => [0; 3],
        };
    }

    pub fn cycle_next(&mut self) {
        match self.src_kind {
            ExternalSourceKind::IdxStreamer(ref mut es) | ExternalSourceKind::Custom(ref mut es) => {
                es.cycle_next()
            },
            ExternalSourceKind::GlyphSequences(ref mut gs) => {
                gs.cycle_next()
            },
            _ => (),
        }
    }

    pub fn layers(&mut self) -> &mut HashMap<LayerTags, ExternalSourceLayer> {
        &mut self.layers
    }

    pub fn layer(&self, tags: LayerTags) -> &ExternalSourceLayer {
        self.layers.get(&tags).expect(&format!("ExternalSource::layer(): Invalid tags: {:?}", tags))
    }

    pub fn layer_tags(&self) -> Vec<LayerTags> {
        let mut tags = Vec::with_capacity(self.layers.len());

        for (_, layer) in self.layers.iter() {
            tags.push(layer.tags());
        }
        tags
    }

    pub fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }
}
