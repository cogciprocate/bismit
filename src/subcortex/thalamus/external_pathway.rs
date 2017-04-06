use std::collections::HashMap;
use std::fmt::Debug;
use find_folder::Search;
use cmn::{self, CorticalDims, CmnResult, CmnError};
use ocl::{EventList};
use map::{AreaScheme, InputScheme, LayerMapScheme, LayerScheme, AxonTopology, LayerAddress,
    AxonDomain, AxonTags, AxonSignature};
use encode::{IdxStreamer, GlyphSequences, SensoryTract, ScalarSequence, ReversoScalarSequence,
    VectorEncoder, ScalarSdrGradiant};
use cmn::TractFrameMut;


#[derive(Debug)]
pub enum ExternalPathwayFrame<'a> {
    Tract(TractFrameMut<'a>),
    F32Slice(&'a mut [f32]),
}


/// A highway for input.
///
pub trait ExternalPathwayTract: Debug {
    fn write_into(&mut self, frame: &mut TractFrameMut, addr: &LayerAddress);
    fn cycle_next(&mut self);
}


#[allow(unused_variables)]
#[derive(Debug)]
pub enum ExternalPathwayEncoder {
    None,
    World,
    Stripes { stripe_size: usize, zeros_first: bool },
    Hexballs { edge_size: usize, invert: bool, fill: bool },
    Exp1,
    GlyphSequences(Box<GlyphSequences>),
    SensoryTract(Box<SensoryTract>),
    VectorEncoder(Box<VectorEncoder>),
    Other(Box<ExternalPathwayTract>),
    OtherUnspecified,
}


pub struct ExternalPathwayLayer {
    name: &'static str,
    addr: LayerAddress,
    axn_sig: AxonSignature,
    axn_topology: AxonTopology,
    dims: Option<CorticalDims>,
}

impl ExternalPathwayLayer {
    pub fn set_dims(&mut self, dims: Option<CorticalDims>) {
        self.dims = dims;
    }

    pub fn name(&self) -> &'static str { self.name }
    pub fn addr(&self) -> &LayerAddress { &self.addr }
    pub fn axn_sig(&self) -> &AxonSignature { &self.axn_sig }
    pub fn axn_tags(&self) -> &AxonTags { &self.axn_sig.tags() }
    pub fn axn_topology(&self) -> AxonTopology { self.axn_topology.clone() }
    pub fn dims(&self) -> Option<&CorticalDims> { self.dims.as_ref() }
}


/// An input source.
///
// [NOTE (out of date)]: To implement multiple layers from a single input source:
// - Must pass layer count to the input 'generator' and have it accept a
//   multi-headed mutable slice when cycled.
pub struct ExternalPathway {
    area_id: usize,
    area_name: String,
    encoder: ExternalPathwayEncoder,
    layers: HashMap<LayerAddress, ExternalPathwayLayer>,
}

impl ExternalPathway {
    // [FIXME]: Determine (or have passed in) the layer depth corresponding to this source.
    pub fn new(pamap: &AreaScheme, plmap: &LayerMapScheme) -> CmnResult<ExternalPathway> {
        let p_layers: Vec<&LayerScheme> = plmap.layers().iter().map(|pl| pl).collect();

        assert!(pamap.get_input().layer_count() == p_layers.len(), "ExternalPathway::new(): \
            Inputs for the area scheme, \"{}\" ({}), must equal the layers in the layer map \
            scheme, '{}' ({}). Ensure `InputScheme::layer_count()` is set correctly for {:?}",
            pamap.name(), pamap.get_input().layer_count(), plmap.name(), p_layers.len(),
            pamap.get_input());

        let mut layers = HashMap::with_capacity(4);
        let mut lyr_addr_list = Vec::with_capacity(4);
        let mut lyr_dims_list = Vec::with_capacity(4);
        let mut lyr_axn_sigs_list = Vec::with_capacity(4);

        for p_layer in p_layers.into_iter() {
            let lyr_name = p_layer.name();
            let lyr_addr = LayerAddress::new(pamap.area_id(), p_layer.layer_id());
            let axn_topology = p_layer.kind().axn_topology();
            let lyr_depth = p_layer.depth().unwrap_or(cmn::DEFAULT_OUTPUT_LAYER_DEPTH);

            let dims = match axn_topology {
                AxonTopology::Spatial => Some(pamap.dims().clone_with_depth(lyr_depth)),
                AxonTopology::Horizontal => None,
                AxonTopology::None => None,
            };

            ////// [FIXME]: Determine if either of these checks is still necessary or relevant:
            // assert!(layer_tags.contains(map::OUTPUT), "ExternalPathway::new(): External ('Thalamic') areas \
            //     must have a layer or layers with an 'OUTPUT' tag. [area: '{}', layer map: '{}']",
            //     pamap.name(), plmap.name());
            // assert!(p_layer.axon_domain().is_output(), "ExternalPathway::new(): External areas \
            //     must currently be output layers. [area: '{}', layer: '{}']", pamap.name(), plmap.name());

            let lyr_axn_sig = match *p_layer.axn_domain() {
                AxonDomain::Output(ref axn_sig) => axn_sig.clone(),
                _ => return Err(format!("ExternalPathway::new(): External areas \
                    must currently be output layers. [area: '{}', layer: '{}']", pamap.name(),
                    plmap.name()).into()),
            };

            lyr_addr_list.push(lyr_addr.clone());
            lyr_dims_list.push(dims.clone());
            lyr_axn_sigs_list.push(lyr_axn_sig.clone());

            layers.insert(lyr_addr.clone(), ExternalPathwayLayer {
                name: lyr_name,
                addr: lyr_addr,
                axn_sig: lyr_axn_sig,
                axn_topology: axn_topology,
                dims: dims,
            });
        }

        let encoder = match *pamap.get_input() {
            InputScheme::IdxStreamer { ref file_name, cyc_per, scale, loop_frames } => {
                assert_eq!(layers.len(), 1);
                let mut is = IdxStreamer::new(layers[&lyr_addr_list[0]].dims()
                    .expect("ExternalPathway::new(): Layer dims not set properly.").clone(),
                    file_name.clone(), cyc_per, scale);

                if loop_frames > 0 {
                    is = is.loop_frames(loop_frames);
                }
                ExternalPathwayEncoder::Other(Box::new(is))
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
                ExternalPathwayEncoder::GlyphSequences(Box::new(gs))
            },
            InputScheme::SensoryTract => {
                assert_eq!(layers.len(), 1);
                let st = SensoryTract::new(layers[&lyr_addr_list[0]].dims()
                    .expect("ExternalPathway::new(): Layer dims not set properly."));
                ExternalPathwayEncoder::SensoryTract(Box::new(st))
            },
            InputScheme::ScalarSequence { range, incr } => {
                let tract_dims = {
                    assert!(lyr_dims_list.len() == 1);
                    lyr_dims_list[0].unwrap().into()
                };

                ExternalPathwayEncoder::Other(Box::new(ScalarSequence::new(range, incr, &tract_dims)))
            },
            InputScheme::ScalarSdrGradiant { range, way_span, incr } => {
                let tract_dims = {
                    assert!(lyr_dims_list.len() == 1);
                    lyr_dims_list[0].unwrap().into()
                };

                ExternalPathwayEncoder::Other(Box::new(ScalarSdrGradiant::new(range, way_span, incr, &tract_dims)))
            },
            InputScheme::ReversoScalarSequence { range, incr } => {
                ExternalPathwayEncoder::Other(Box::new(
                    ReversoScalarSequence::new(range, incr, &lyr_addr_list)))
            },
            InputScheme::VectorEncoder { ref ranges } => {
                let tract_dims: Vec<_> = lyr_dims_list.iter().map(|d| d.unwrap().into()).collect();

                ExternalPathwayEncoder::VectorEncoder(Box::new(try!(
                    VectorEncoder::new(ranges.clone(), &lyr_addr_list, &tract_dims)
                )))
            },
            InputScheme::Custom { .. } => {
                ExternalPathwayEncoder::OtherUnspecified
            },
            InputScheme::None { .. } => {
                ExternalPathwayEncoder::None
            }
            InputScheme::Zeros => ExternalPathwayEncoder::None,
            ref is @ _ => panic!("\nExternalPathway::new(): Input type: '{:?}' not yet supported.", is),
        };

        Ok(ExternalPathway {
            area_id: pamap.area_id(),
            area_name: pamap.name().to_owned(),
            layers: layers,
            encoder: encoder,
        })
    }

    /// Writes input data into a tract.
    ///
    /// **Should** return promptly... data should already be staged (* TODO: Process
    /// in a separate thread).
    pub fn write_into(&mut self, addr: &LayerAddress, mut frame: TractFrameMut, _: &mut EventList) {
        let dims = self.layers[addr].dims().expect(&format!("Dimensions don't exist for \
            external input area: \"{}\", addr: '{:?}' ", self.area_name, addr));

        debug_assert!(dims == frame.dims(), "Dimensional mismatch for external input \
            area: \"{}\", addr: '{:?}', layer dims: {:?}, tract dims: {:?}", self.area_name, addr,
            dims, frame.dims());

        match self.encoder {
            ExternalPathwayEncoder::Other(ref mut es) => {
                es.write_into(&mut frame, addr)
            },
            ExternalPathwayEncoder::GlyphSequences(ref mut es) => {
                es.write_into(&mut frame, addr)
            },
            ExternalPathwayEncoder::SensoryTract(ref mut es) => {
                es.write_into(&mut frame, addr)
            },
            ExternalPathwayEncoder::VectorEncoder(ref mut es) => {
                es.write_into(&mut frame, addr)
            },
            ExternalPathwayEncoder::OtherUnspecified => {
                panic!("ExternalPathway::write_into: Custom pathway not specified.")
            },
            _ => (),
        }
    }

    /// Returns a tract frame of an external source buffer, if available.
    pub fn ext_frame_mut(&mut self) -> CmnResult<ExternalPathwayFrame> {
        match self.encoder {
            ExternalPathwayEncoder::SensoryTract(ref mut es) => {
                Ok(es.ext_frame_mut())
            },
            ExternalPathwayEncoder::VectorEncoder(ref mut es) => {
                Ok(es.ext_frame_mut())
            },
            ExternalPathwayEncoder::OtherUnspecified => {
                panic!("ExternalPathway::write_into: Custom pathway not specified.")
            },
            _ => Err(CmnError::new(format!("ExternalPathway::ext_frame_Mut(): No tract available for the source \
                kind: {:?}.", self.encoder))),
        }
    }

    pub fn cycle_next(&mut self) {
        match self.encoder {
            ExternalPathwayEncoder::Other(ref mut es) => {
                es.cycle_next()
            },
            ExternalPathwayEncoder::GlyphSequences(ref mut es) => {
                es.cycle_next()
            },
            ExternalPathwayEncoder::SensoryTract(ref mut es) => {
                es.cycle_next()
            },
            ExternalPathwayEncoder::OtherUnspecified => {
                panic!("ExternalPathway::write_into: Custom pathway not specified.")
            },
            _ => (),
        }
    }

    pub fn layers(&mut self) -> &mut HashMap<LayerAddress, ExternalPathwayLayer> {
        &mut self.layers
    }

    pub fn layer(&self, addr: LayerAddress) -> &ExternalPathwayLayer {
        self.layers.get(&addr).expect(&format!("ExternalPathway::layer(): Invalid addr: {:?}", addr))
    }

    pub fn layer_addrs(&self) -> Vec<LayerAddress> {
        self.layers.iter().map(|(_, layer)| layer.addr().clone()).collect()
    }

    // Specify a custom encoder tract. Input scheme must have been configured
    // `InputScheme::Custom` in `AreaScheme`.
    pub fn specify_encoder(&mut self, tract: Box<ExternalPathwayTract>) -> CmnResult<()> {
        match self.encoder {
            ExternalPathwayEncoder::OtherUnspecified => (),
            _ => return CmnError::err("ExternalPathway::specify_encoder(): Encoder already specified."),
        }

        self.encoder = ExternalPathwayEncoder::Other(tract);
        Ok(())
    }

    pub fn area_id(&self) -> usize { self.area_id }
    pub fn area_name<'a>(&'a self) -> &'a str { &self.area_name }
    pub fn encoder(&mut self) -> &mut ExternalPathwayEncoder { &mut self.encoder }
}
