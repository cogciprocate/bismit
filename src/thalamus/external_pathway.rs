// use std::iter;
use std::collections::HashMap;
use std::fmt::Debug;
// use std::collections::hash_map::IterMut;
// use std::hash::BuildHasherDefault;
// use twox_hash::XxHash;
use find_folder::Search;
use cmn::{self, CorticalDims, CmnResult, CmnError};
use ocl::{EventList};
use map::{self, AreaScheme, InputScheme, LayerMapScheme, LayerScheme, AxonKind};
use encode::{IdxStreamer, GlyphSequences, SensoryTract, ScalarSequence, ReversoScalarSequence,
    VectorEncoder};
use cmn::TractFrameMut;
use map::LayerTags;


#[derive(Debug)]
pub enum ExternalPathwayFrame<'a> {
    Tract(TractFrameMut<'a>),
    F32Slice(&'a mut [f32]),
}


/// A highway for input.
///
/// Returns a 3-array because I didn't want to bother with generics or enums
/// for the moment.
///
pub trait ExternalPathwayTract: Debug {
    fn write_into(&mut self, frame: &mut TractFrameMut, tags: LayerTags)
        -> [usize; 3];
    fn cycle_next(&mut self);
}

#[allow(unused_variables)]
#[derive(Debug)]
pub enum ExternalPathwayKind {
    None,
    World,
    Stripes { stripe_size: usize, zeros_first: bool },
    Hexballs { edge_size: usize, invert: bool, fill: bool },
    Exp1,
    // IdxStreamer(Box<ExternalPathwayTract>),
    GlyphSequences(Box<GlyphSequences>),
    SensoryTract(Box<SensoryTract>),
    VectorEncoder(Box<VectorEncoder>),
    Other(Box<ExternalPathwayTract>),
}


pub struct ExternalPathwayLayer {
    layer_name: &'static str,
    layer_tags: LayerTags,
    axn_kind: AxonKind,
    dims: Option<CorticalDims>,
}

impl ExternalPathwayLayer {
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
pub struct ExternalPathway {
    area_name: String,
    src_kind: ExternalPathwayKind,
    // layers: HashMap<LayerTags, ExternalPathwayLayer, BuildHasherDefault<XxHash>>,
    layers: HashMap<LayerTags, ExternalPathwayLayer>,
}

impl ExternalPathway {
    // [FIXME] Determine (or have passed in) the layer depth corresponding to this source.
    pub fn new(pamap: &AreaScheme, plmap: &LayerMapScheme) -> CmnResult<ExternalPathway> {
        let p_layers: Vec<&LayerScheme> = plmap.layers().iter().map(|(_, pl)| pl).collect();

        assert!(pamap.get_input().layer_count() == p_layers.len(), "ExternalPathway::new(): \
            Inputs for 'AreaScheme' ({}) must equal layers in 'LayerMapScheme' ({}). Ensure \
            `InputScheme::layer_count()` is set correctly for {:?}",
            pamap.get_input().layer_count(), p_layers.len(), pamap.get_input());

        // let mut layers = HashMap::with_capacity_and_hasher(4, BuildHasherDefault::default());
        let mut layers = HashMap::with_capacity(4);
        let mut layer_tags_list = Vec::with_capacity(4);
        let mut layer_dims_list = Vec::with_capacity(4);

        for p_layer in p_layers.into_iter() {
            let layer_name = p_layer.name();
            let layer_tags = p_layer.tags();
            let axn_kind = p_layer.kind().axn_kind().expect("ExternalPathway::new(): ExternalPathway layer \
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

            assert!(layer_tags.contains(map::OUTPUT), "ExternalPathway::new(): External ('Thalamic') areas \
                must have a layer or layers with an 'OUTPUT' tag. [area: '{}', layer map: '{}']",
                pamap.name(), plmap.name());

            layer_tags_list.push(layer_tags);
            layer_dims_list.push(dims.clone());

            layers.insert(layer_tags, ExternalPathwayLayer {
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
                    .expect("ExternalPathway::new(): Layer dims not set properly.").clone(),
                    file_name, cyc_per, scale);

                if loop_frames > 0 {
                    is = is.loop_frames(loop_frames);
                }
                ExternalPathwayKind::Other(Box::new(is))
            },
            InputScheme::GlyphSequences { seq_lens, seq_count, scale, hrz_dims } => {
                let label_file = Search::ParentsThenKids(3, 3).for_folder("tmp_data")
                    .expect("ExternalPathway::new(): 'label file folder (tmp_data)'")
                    .join("train-labels-idx1-ubyte");
                let image_file = Search::ParentsThenKids(3, 3).for_folder("tmp_data")
                    .expect("ExternalPathway::new(): 'image file folder (tmp_data)'")
                    .join("train-images-idx3-ubyte");
                let gs = GlyphSequences::new(&mut layers, seq_lens, seq_count, scale, hrz_dims,
                    label_file, image_file);
                ExternalPathwayKind::GlyphSequences(Box::new(gs))
            },
            InputScheme::SensoryTract => {
                assert_eq!(layers.len(), 1);
                let st = SensoryTract::new(layers[&layer_tags_list[0]].dims()
                    .expect("ExternalPathway::new(): Layer dims not set properly."));
                ExternalPathwayKind::SensoryTract(Box::new(st))
            },
            InputScheme::ScalarSequence { range, incr } => {
                let tract_dims = {
                    assert!(layer_dims_list.len() == 1);
                    layer_dims_list[0].unwrap().into()
                };

                ExternalPathwayKind::Other(Box::new(ScalarSequence::new(range, incr, &tract_dims)))
            },
            InputScheme::ReversoScalarSequence { range, incr } => {
                // let layer_tags: Vec<_> = layers.iter().map(|(t, _)| t.clone()).collect();
                ExternalPathwayKind::Other(Box::new(
                    ReversoScalarSequence::new(range, incr, &layer_tags_list)))
            },
            InputScheme::VectorEncoder { ranges } => {
                let tract_dims: Vec<_> = layer_dims_list.iter().map(|d| d.unwrap().into()).collect();

                ExternalPathwayKind::VectorEncoder(Box::new(try!(
                    VectorEncoder::new(ranges, &layer_tags_list, &tract_dims)
                )))
            },
            InputScheme::None | InputScheme::Zeros => ExternalPathwayKind::None,
            is @ _ => panic!("\nExternalPathway::new(): Input type: '{:?}' not yet supported.", is),
        };

        Ok(ExternalPathway {
            area_name: pamap.name.to_owned(),
            layers: layers,
            src_kind: src_kind,
        })
    }

    /// Writes input data into a tract.
    ///
    /// **Should** return promptly... data should already be staged.
    pub fn write_into(&mut self, tags: LayerTags, mut frame: TractFrameMut, _: &mut EventList) {
        let dims = self.layers[&tags].dims().expect(&format!("Dimensions don't exist for \
            external input area: \"{}\", tags: '{:?}' ", self.area_name, tags));

        debug_assert!(dims == frame.dims(), "Dimensional mismatch for external input \
            area: \"{}\", tags: '{:?}', layer dims: {:?}, tract dims: {:?}", self.area_name, tags,
            dims, frame.dims());

        // '.cycle()' returns a [usize; 3], not sure what we're going to do with it.
        let _ = match self.src_kind {
            // ExternalPathwayKind::IdxStreamer(ref mut es) |
            ExternalPathwayKind::Other(ref mut es) => {
                es.write_into(&mut frame, tags)
            },
            ExternalPathwayKind::GlyphSequences(ref mut es) => {
                es.write_into(&mut frame, tags)
            },
            ExternalPathwayKind::SensoryTract(ref mut es) => {
                es.write_into(&mut frame, tags)
            },
            ExternalPathwayKind::VectorEncoder(ref mut es) => {
                es.write_into(&mut frame, tags)
            },
            _ => [0; 3],
        };
    }

    // pub fn frame<'f>(&'f self) -> Option<&'f mut [u8]> {
    //     None
    // }

    /// Returns a tract frame of an external source buffer, if available.
    pub fn ext_frame_mut(&mut self) -> CmnResult<ExternalPathwayFrame> {
        match self.src_kind {
            ExternalPathwayKind::SensoryTract(ref mut es) => {
                Ok(es.ext_frame_mut())
            },
            ExternalPathwayKind::VectorEncoder(ref mut es) => {
                Ok(es.ext_frame_mut())
            },
            _ => Err(CmnError::new(format!("ExternalPathway::ext_frame_Mut(): No tract available for the source \
                kind: {:?}.", self.src_kind))),
        }
    }

    pub fn cycle_next(&mut self) {
        match self.src_kind {
            // ExternalPathwayKind::IdxStreamer(ref mut es) |
            ExternalPathwayKind::Other(ref mut es) => {
                es.cycle_next()
            },
            ExternalPathwayKind::GlyphSequences(ref mut es) => {
                es.cycle_next()
            },
            ExternalPathwayKind::SensoryTract(ref mut es) => {
                es.cycle_next()
            },
            _ => (),
        }
    }

    pub fn layers(&mut self) -> &mut HashMap<LayerTags, ExternalPathwayLayer> {
        &mut self.layers
    }

    pub fn layer(&self, tags: LayerTags) -> &ExternalPathwayLayer {
        self.layers.get(&tags).expect(&format!("ExternalPathway::layer(): Invalid tags: {:?}", tags))
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
