// use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, SyncSender, /*Receiver*/};
use std::collections::HashMap;
use std::fmt::Debug;
// use std::mem::{self, Discriminant};
use find_folder::Search;
use cmn::{self, CorticalDims, CmnResult, /*CmnError,*/ TractDims};
use ocl::{FutureWriteGuard};
use map::{AreaScheme, EncoderScheme, LayerMapScheme, LayerScheme, AxonTopology, LayerAddress,
    AxonDomain, AxonTags, AxonSignature};
use encode::{IdxStreamer, GlyphSequences, SensoryTract, ScalarSequence, ReversoScalarSequence,
    VectorEncoder, ScalarSdrGradiant};
use cmn::TractFrameMut;
use subcortex::{Thalamus, SubcorticalNucleus, SubcorticalNucleusLayer, TractSender};


#[derive(Debug)]
pub enum InputGeneratorFrame<'a> {
    Writer(FutureWriteGuard<u8>),
    Tract(TractFrameMut<'a>),
    F32Slice(&'a mut [f32]),
}


/// A highway for input.
///
pub trait InputGeneratorTract: Debug + Send {
    fn write_into(&mut self, frame: &mut TractFrameMut, addr: LayerAddress);
    fn cycle_next(&mut self);
}


#[allow(unused_variables)]
#[derive(Debug)]
pub enum InputGeneratorEncoder {
    None,
    World,
    Stripes { stripe_size: usize, zeros_first: bool },
    Hexballs { edge_size: usize, invert: bool, fill: bool },
    Exp1,
    GlyphSequences(Box<GlyphSequences>),
    SensoryTract(Box<SensoryTract>),
    VectorEncoder(Box<VectorEncoder>),
    Custom(Box<InputGeneratorTract>),
    CustomUnspecified,
}

impl InputGeneratorEncoder {
    /// Writes input data into a tract.
    pub fn write_into(&mut self, addr: LayerAddress, dims: TractDims, future_write: FutureWriteGuard<Vec<u8>>) {
        let mut data = future_write.wait().unwrap();
        let mut frame = TractFrameMut::new(data.as_mut_slice(), dims);

        match *self {
            InputGeneratorEncoder::Custom(ref mut es) => {
                es.write_into(&mut frame, addr)
            },
            InputGeneratorEncoder::GlyphSequences(ref mut es) => {
                es.write_into(&mut frame, addr)
            },
            InputGeneratorEncoder::SensoryTract(ref mut es) => {
                es.write_into(&mut frame, addr)
            },
            InputGeneratorEncoder::VectorEncoder(ref mut es) => {
                es.write_into(&mut frame, addr)
            },
            InputGeneratorEncoder::CustomUnspecified => {
                panic!("InputGenerator::write_into: Custom pathway not specified.")
            },
            _ => (),
        }
    }

    pub fn cycle_next(&mut self) {
        match *self {
            InputGeneratorEncoder::Custom(ref mut es) => {
                es.cycle_next()
            },
            InputGeneratorEncoder::GlyphSequences(ref mut es) => {
                es.cycle_next()
            },
            InputGeneratorEncoder::SensoryTract(ref mut es) => {
                es.cycle_next()
            },
            InputGeneratorEncoder::CustomUnspecified => {
                panic!("InputGenerator::cycle_next: Custom pathway not specified.")
            },
            _ => (),
        }
    }

    pub fn set_ranges(&mut self, ranges: Vec<(f32, f32)>) {
        match *self {
            InputGeneratorEncoder::VectorEncoder(ref mut v) => {
                v.set_ranges(&ranges).unwrap();
            }
            _ => unimplemented!(),
        }
    }

    // /// Returns a tract frame of an external source buffer, if available.
    // pub fn ext_frame_mut(&mut self) -> CmnResult<InputGeneratorFrame> {
    //     match self.encoder {
    //         InputGeneratorEncoder::SensoryTract(ref mut es) => {
    //             Ok(es.ext_frame_mut())
    //         },
    //         InputGeneratorEncoder::VectorEncoder(ref mut es) => {
    //             Ok(es.ext_frame_mut())
    //         },
    //         InputGeneratorEncoder::CustomUnspecified => {
    //             panic!("InputGenerator::write_into: Custom pathway not specified.")
    //         },
    //         _ => Err(CmnError::new(format!("InputGenerator::ext_frame_Mut(): No tract available for the source \
    //             kind: {:?}.", self.encoder))),
    //     }
    // }
}


// pub struct SubcorticalNucleusLayer {
//     name: &'static str,
//     addr: LayerAddress,
//     axn_sig: AxonSignature,
//     axn_topology: AxonTopology,
//     dims: Option<CorticalDims>,
// }

// impl SubcorticalNucleusLayer {
//     pub fn set_dims(&mut self, dims: Option<CorticalDims>) {
//         self.dims = dims;
//     }

//     pub fn name(&self) -> &'static str { self.name }
//     pub fn addr(&self) -> &LayerAddress { &self.addr }
//     pub fn axn_sig(&self) -> &AxonSignature { &self.axn_sig }
//     pub fn axn_tags(&self) -> &AxonTags { &self.axn_sig.tags() }
//     pub fn axn_topology(&self) -> AxonTopology { self.axn_topology.clone() }
//     pub fn dims(&self) -> Option<&CorticalDims> { self.dims.as_ref() }
// }


enum EncoderCmd {
    WriteInto {addr: LayerAddress, dims: TractDims, future_write: FutureWriteGuard<Vec<u8>> },
    Cycle,
    SetRanges(Vec<(f32, f32)>),
    SetEncoder(InputGeneratorEncoder),
    Exit,
}

// enum EncoderRes {

// }


pub struct InputGeneratorLayer {
    // axn_sig: AxonSignature,
    // axn_topology: AxonTopology,
    sub: SubcorticalNucleusLayer,
    pathway: Option<TractSender>,
}

impl InputGeneratorLayer {
    pub fn set_dims(&mut self, dims: CorticalDims) {
        self.sub.set_dims(dims);
    }

    pub fn axn_sig(&self) -> &AxonSignature {
        // &self.axn_sig
        match *self.sub.axon_domain() {
            AxonDomain::Output(ref sig) => sig,
            _ => panic!("InputGeneratorLayer::axn_sig: Input generator layers must be \
                AxonDomain::Output(..)."),
        }
    }

    pub fn axn_tags(&self) -> &AxonTags {
        &self.axn_sig().tags()
    }

    pub fn axn_topology(&self) -> AxonTopology {
        self.sub.axon_topology().clone()
    }

    pub fn sub(&self) -> &SubcorticalNucleusLayer {
        &self.sub
    }

    pub fn sub_mut(&mut self) -> &mut SubcorticalNucleusLayer {
        &mut self.sub
    }

    pub fn pathway(&self) -> Option<&TractSender> {
        self.pathway.as_ref()
    }
}



/// An input source.
///
// [NOTE (out of date)]: To implement multiple layers from a single input source:
// - Must pass layer count to the input 'generator' and have it accept a
//   multi-headed mutable slice when cycled.
pub struct InputGenerator {
    area_id: usize,
    area_name: String,
    // encoder: InputGeneratorEncoder,
    layers: HashMap<LayerAddress, InputGeneratorLayer>,
    // thalamic_pathways: Vec<>,
    tx: SyncSender<EncoderCmd>,
    _thread: Option<JoinHandle<()>>,
    // rx: Receiver<EncoderRes>,
    disabled: bool,
}

impl InputGenerator {
    // [FIXME]: Determine (or have passed in) the layer depth corresponding to this source.
    pub fn new(plmap: &LayerMapScheme, pamap: &AreaScheme) -> CmnResult<InputGenerator> {
        let p_layers: Vec<&LayerScheme> = plmap.layers().iter().map(|pl| pl).collect();

        // assert!(pamap.get_encoder().layer_count() == p_layers.len(), "InputGenerator::new(): \
        //     Inputs for the area scheme, \"{}\" ({}), must equal the layers in the layer map \
        //     scheme, '{}' ({}). Ensure `EncoderScheme::layer_count()` is set correctly for {:?}",
        //     pamap.name(), pamap.get_encoder().layer_count(), plmap.name(), p_layers.len(),
        //     pamap.get_encoder());

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
            // assert!(layer_tags.contains(map::OUTPUT), "InputGenerator::new(): External ('Thalamic') areas \
            //     must have a layer or layers with an 'OUTPUT' tag. [area: '{}', layer map: '{}']",
            //     pamap.name(), plmap.name());
            // assert!(p_layer.axon_domain().is_output(), "InputGenerator::new(): External areas \
            //     must currently be output layers. [area: '{}', layer: '{}']", pamap.name(), plmap.name());

            let lyr_axn_sig = match *p_layer.axn_domain() {
                AxonDomain::Output(ref axn_sig) => axn_sig.clone(),
                _ => return Err(format!("InputGenerator::new(): External areas \
                    must currently be output layers. [area: '{}', layer: '{}']", pamap.name(),
                    plmap.name()).into()),
            };

            lyr_addr_list.push(lyr_addr.clone());
            lyr_dims_list.push(dims.clone());
            lyr_axn_sigs_list.push(lyr_axn_sig.clone());

            // layers.insert(lyr_addr.clone(), SubcorticalNucleusLayer {
            //     name: lyr_name,
            //     addr: lyr_addr,
            //     axn_sig: lyr_axn_sig,
            //     axn_topology: axn_topology,
            //     dims: dims,
            // });

            let layer = InputGeneratorLayer {
                sub: SubcorticalNucleusLayer::new(lyr_name, lyr_addr, p_layer.axn_domain().clone(),
                    axn_topology, dims.unwrap_or(CorticalDims::new(0, 0, 0, None))),
                pathway: None,
            };

            layers.insert(lyr_addr.clone(), layer);
        }

        let mut disabled = false;

        let encoder = match *pamap.get_encoder() {
            EncoderScheme::IdxStreamer { ref file_name, cyc_per, scale, loop_frames } => {
                assert_eq!(layers.len(), 1);
                let mut is = IdxStreamer::new(layers[&lyr_addr_list[0]].sub.dims()
                    // .expect("InputGenerator::new(): Layer dims not set properly.")
                    .clone(),
                    file_name.clone(), cyc_per, scale);

                if loop_frames > 0 {
                    is = is.loop_frames(loop_frames);
                }
                InputGeneratorEncoder::Custom(Box::new(is))
            },
            EncoderScheme::GlyphSequences { seq_lens, seq_count, scale, hrz_dims } => {
                let label_file = Search::ParentsThenKids(3, 3).for_folder("tmp_data")
                    .expect("InputGenerator::new(): 'label file folder (tmp_data)'")
                    .join("train-labels-idx1-ubyte");
                let image_file = Search::ParentsThenKids(3, 3).for_folder("tmp_data")
                    .expect("InputGenerator::new(): 'image file folder (tmp_data)'")
                    .join("train-images-idx3-ubyte");
                let gs = GlyphSequences::new(&mut layers, seq_lens, seq_count, scale, hrz_dims,
                    label_file, image_file);
                InputGeneratorEncoder::GlyphSequences(Box::new(gs))
            },
            EncoderScheme::SensoryTract => {
                assert_eq!(layers.len(), 1);
                let st = SensoryTract::new(layers[&lyr_addr_list[0]].sub.dims()
                    // .expect("InputGenerator::new(): Layer dims not set properly.")
                    );
                InputGeneratorEncoder::SensoryTract(Box::new(st))
            },
            EncoderScheme::ScalarSequence { range, incr } => {
                let tract_dims = {
                    assert!(lyr_dims_list.len() == 1);
                    lyr_dims_list[0].unwrap().into()
                };

                InputGeneratorEncoder::Custom(Box::new(ScalarSequence::new(range, incr, &tract_dims)))
            },
            EncoderScheme::ScalarSdrGradiant { range, way_span, incr } => {
                let tract_dims = {
                    assert!(lyr_dims_list.len() == 1);
                    lyr_dims_list[0].unwrap().into()
                };

                InputGeneratorEncoder::Custom(Box::new(ScalarSdrGradiant::new(range, way_span, incr, &tract_dims)))
            },
            EncoderScheme::ReversoScalarSequence { range, incr } => {
                InputGeneratorEncoder::Custom(Box::new(
                    ReversoScalarSequence::new(range, incr, &lyr_addr_list)))
            },
            EncoderScheme::VectorEncoder { ref ranges } => {
                let tract_dims: Vec<_> = lyr_dims_list.iter().map(|d| d.unwrap().into()).collect();

                InputGeneratorEncoder::VectorEncoder(Box::new(try!(
                    VectorEncoder::new(ranges.clone(), &lyr_addr_list, &tract_dims)
                )))
            },
            EncoderScheme::Custom => {
                InputGeneratorEncoder::CustomUnspecified
            },
            EncoderScheme::None => {
                disabled = true;
                InputGeneratorEncoder::None
            }
            EncoderScheme::Subcortex => {
                disabled = true;
                InputGeneratorEncoder::None
            }
            EncoderScheme::Zeros => InputGeneratorEncoder::None,
            ref is @ _ => panic!("\nInputGenerator::new(): Input type: '{:?}' not yet supported.", is),
        };

        let (tx, rx) = mpsc::sync_channel(1);
        let thread_name = format!("InputGeneratorEncoder_{}", pamap.name());
        let thread_handle: JoinHandle<_> = thread::Builder::new().name(thread_name).spawn(move || {
            let mut encoder = encoder;
            let rx = rx;

            loop {
                match rx.recv().unwrap() {
                    EncoderCmd::WriteInto { addr, dims, future_write } =>
                        encoder.write_into(addr, dims, future_write),
                    EncoderCmd::Cycle => encoder.cycle_next(),
                    EncoderCmd::SetRanges(ranges) => encoder.set_ranges(ranges),
                    EncoderCmd::SetEncoder(e) => encoder = e,
                    EncoderCmd::Exit => break,
                }
            }
        }).unwrap();

        // let thalamic_pathways = Vec::with_capacity(self.layers.len());

        Ok(InputGenerator {
            area_id: pamap.area_id(),
            area_name: pamap.name().to_owned(),
            layers: layers,
            // thalamic_pathways,
            // encoder: encoder,
            _thread: Some(thread_handle),
            tx: tx,
            disabled,
        })
    }

    // Specify a custom encoder tract. Input scheme must have been configured
    // as `EncoderScheme::Custom` in `AreaScheme`.
    pub fn set_encoder(&self, tract: Box<InputGeneratorTract>) {
        // match self.encoder {
        //     InputGeneratorEncoder::CustomUnspecified => (),
        //     _ => return CmnError::err("InputGenerator::specify_encoder(): Encoder already specified."),
        // }
        // self.encoder = InputGeneratorEncoder::Custom(tract);
        self.tx.send(EncoderCmd::SetEncoder(InputGeneratorEncoder::Custom(tract))).unwrap();
    }

    /// Writes input data into a tract.
    // pub fn write_into(&mut self, addr: &LayerAddress, mut frame: TractFrameMut, _: &mut EventList) {
    pub fn write_into(&self, addr: LayerAddress, future_write: FutureWriteGuard<Vec<u8>>) {
        if !self.disabled {
            let dims = self.layers[&addr].sub.dims()
                // .expect(&format!("Dimensions don't exist for external input area: \"{}\", \
                //     addr: '{:?}' ", self.area_name, addr))
                    .into();
            self.tx.send(EncoderCmd::WriteInto { addr, dims, future_write }).unwrap();
        }
    }

    pub fn cycle_next(&self) {
        if !self.disabled { self.tx.send(EncoderCmd::Cycle).unwrap(); }
    }

    pub fn set_encoder_ranges(&self, ranges: Vec<(f32, f32)>) {
        if !self.disabled { self.tx.send(EncoderCmd::SetRanges(ranges)).unwrap(); }
    }

    pub fn layers_mut(&mut self) -> &mut HashMap<LayerAddress, InputGeneratorLayer> {
        &mut self.layers
    }

    // pub fn layer(&self, addr: LayerAddress) -> &SubcorticalNucleusLayer {
    //     self.layers.get(&addr).expect(&format!("InputGenerator::layer(): Invalid addr: {:?}", addr))
    // }

    pub fn layer_addrs(&self) -> Vec<LayerAddress> {
        self.layers.iter().map(|(_, layer)| layer.sub.addr().clone()).collect()
    }

    pub fn area_id(&self) -> usize { self.area_id }
    pub fn area_name<'a>(&'a self) -> &'a str { &self.area_name }
    // pub fn encoder(&mut self) -> &mut InputGeneratorEncoderKind { &mut self.encoder_kind }
    pub fn is_disabled(&self) -> bool { self.disabled }
}

impl Drop for InputGenerator {
    fn drop(&mut self) {
        self.tx.send(EncoderCmd::Exit).unwrap();
        self._thread.take().unwrap().join().unwrap();
    }
}

impl SubcorticalNucleus for InputGenerator {
    fn create_pathways(&mut self, _thal: &mut Thalamus) {
        for _layer in self.layers.values_mut() {

        }
    }

    fn pre_cycle(&mut self, _thal: &mut Thalamus) {
        self.cycle_next()
    }

    fn post_cycle(&mut self, _thal: &mut Thalamus) {}


    fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer> {
        self.layers.get(&addr).map(|l| l.sub())
    }

    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }
}