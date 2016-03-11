// use std::iter;
use std::collections::{HashMap};

use map::{self, LayerTags};
use cmn::{self, Sdr, CorticalDims};
use ocl::{EventList};
use proto::{ProtoareaMap, Protoinput, ProtolayerMap, Protolayer, AxonKind};
use encode::{IdxStreamer, GlyphSequences};

pub type InputSources = HashMap<(&'static str, LayerTags), InputSource>;

pub trait InputTract {
    fn cycle(&mut self, tract: &mut Sdr) -> usize;
}

pub enum InputSourceKind {
    None,
    World,
    Stripes { stripe_size: usize, zeros_first: bool },
    Hexballs { edge_size: usize, invert: bool, fill: bool },
    Exp1,
    IdxStreamer(Box<InputTract>),
    // IdxStreamerLoop(Box<InputTract>),
    GlyphSequences(Box<GlyphSequences>),
    Custom(Box<InputTract>),
}

pub struct InputSource {
    area_name: &'static str,
    layer_tags: LayerTags,
    kind: InputSourceKind,
    layer_name: &'static str,
    axn_kind: AxonKind,
    dims: CorticalDims,
    // source: Box<InputTract>,
}

impl InputSource {
    // [FIXME] Determine (or have passed in) the layer depth corresponding to this source.
    pub fn new(pamap: &ProtoareaMap, plmap: &ProtolayerMap) -> InputSource {
        let input = pamap.input.clone();

        let layers: Vec<&Protolayer> = plmap.layers().iter().map(|(_, pl)| pl).collect();

        // To implement multiple layers from a single input source:
        // - Must pass layer count to the input 'generator' and have it accept a multi-headed
        //   mutable slice when cycled.
        // - Set the following assert to .len() == .len() (or remove).
        assert!(plmap.layers().len() == 1 && layers.len() == 1, "InputSource::new(): External \
            ('Thalamic') areas with layer maps having more (or less) than one layer are not yet \
            allowed. [area: '{}', layer map: '{}']", pamap.name(), plmap.name());

        let layer_name = layers[0].name();
        let layer_tags = layers[0].tags();
        let axn_kind = layers[0].kind().axn_kind().expect("InputSource::new(): InputSource layer \
            must be 'LayerKind::Axonal(_)'.");
        let layer_depth = layers[0].depth().unwrap_or(cmn::DEFAULT_OUTPUT_LAYER_DEPTH);
        let dims = pamap.dims.clone_with_depth(layer_depth);

        assert!(layer_tags.contains(map::OUTPUT), "InputSource::new(): External ('Thalamic') areas \
            must have a single layer with an 'OUTPUT' tag. [area: '{}', layer map: '{}']", 
            pamap.name(), plmap.name());

        let kind = match input.clone() {
            Protoinput::IdxStreamer { file_name, cyc_per, scale } => {
                let ir = IdxStreamer::new(&dims, file_name, 
                    cyc_per, scale);                
                InputSourceKind::IdxStreamer(Box::new(ir))
            },
            Protoinput::IdxStreamerLoop { file_name, cyc_per, scale, loop_frames } => {
                let ir = IdxStreamer::new(&dims, file_name, 
                    cyc_per, scale).loop_frames(loop_frames);                
                InputSourceKind::IdxStreamer(Box::new(ir))
            },
            Protoinput::GlyphSequences { seq_lens, seq_count, scale } => {
                let gs = GlyphSequences::new(&dims, seq_lens, seq_count, scale);
                // let gs = GlyphSequences::new(&dims, input);
                InputSourceKind::GlyphSequences(Box::new(gs))
            },
            Protoinput::None | Protoinput::Zeros => InputSourceKind::None,
            _ => panic!("\nInputSource::new(): Input type not yet supported."),
        };

        InputSource {
            area_name: pamap.name,
            layer_tags: layer_tags, 
            kind: kind,
            layer_name: layer_name,            
            axn_kind: axn_kind,
            dims: dims,            
        }
    }

    // [FIXME] Multiple output target areas disabled.
    pub fn cycle(&mut self, tract: &mut Sdr, _: &mut EventList) {
        // This is temp (mult out tar areas): DEPRICATING: 
        // debug_assert!(self.targets.len() == 1);

        // '.cycle()' returns a usize (iter or something), not sure what we're going to do with it.
        let _ = match self.kind {
            InputSourceKind::IdxStreamer(ref mut ig) |
            InputSourceKind::Custom(ref mut ig)
                => { ig.cycle(tract) },
            InputSourceKind::GlyphSequences(ref mut gs) => gs.cycle(tract),
            _ => 0,
        };
    }

    pub fn area_name(&self) -> &'static str {
        self.area_name
    }

    pub fn tags(&self) -> LayerTags {
        self.layer_tags
    }

    pub fn axn_kind(&self) -> AxonKind {
        self.axn_kind.clone()
    }

    pub fn layer_name(&self) -> &'static str {
        self.layer_name
    }

    // pub fn depth(&self) -> u8 {
    //     self.depth
    // }

    pub fn dims(&self) -> &CorticalDims {
        &self.dims
    }
}
