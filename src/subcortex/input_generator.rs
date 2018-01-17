use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, SyncSender};
use std::collections::HashMap;
use std::fmt::Debug;
use find_folder::Search;
use cmn::{self, CorticalDims, CmnResult, TractDims, TractFrameMut};
use ocl::{FutureWriteGuard};
use map::{LayerMapSchemeList, AreaSchemeList, EncoderScheme, LayerScheme, AxonTopology,
    LayerAddress, AxonDomain, AxonTags, AxonSignature};
use encode::{IdxStreamer, GlyphSequences, SensoryTract, ScalarSequence, ReversoScalarSequence,
    VectorEncoder, ScalarSdrGradiant};
use subcortex::{Thalamus, SubcorticalNucleus, SubcorticalNucleusLayer, TractSender};
use cortex::{WorkPool, CorticalAreas};


#[derive(Debug)]
pub enum InputGeneratorFrame<'a> {
    Writer(FutureWriteGuard<u8>),
    Tract(TractFrameMut<'a>),
    F32Slice(&'a mut [f32]),
}


/// A highway for input.
pub trait InputGeneratorTract: Debug + Send {
    fn write_into(&mut self, frame: &mut TractFrameMut, addr: LayerAddress);
    fn cycle_next(&mut self);
}


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
        let mut buffer = future_write.wait().expect("InputGeneratorEncoder::write_into");
        let mut frame = TractFrameMut::new(buffer.as_mut_slice(), dims);

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
}

enum EncoderCmd {
    WriteInto {addr: LayerAddress, dims: TractDims, future_write: FutureWriteGuard<Vec<u8>> },
    Cycle,
    SetRanges(Vec<(f32, f32)>),
    SetEncoder(InputGeneratorEncoder),
    Exit,
}


pub struct InputGeneratorLayer {
    sub: SubcorticalNucleusLayer,
    pathway: Option<TractSender>,
}

impl InputGeneratorLayer {
    pub fn set_dims(&mut self, dims: CorticalDims) {
        self.sub.set_dims(dims);
    }

    pub fn axn_sig(&self) -> &AxonSignature {
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
//
// [NOTE (out of date)]: To implement multiple layers from a single input source:
// - Must pass layer count to the input 'generator' and have it accept a
//   multi-headed mutable slice when cycled.
pub struct InputGenerator {
    area_id: usize,
    area_name: String,
    layers: HashMap<LayerAddress, InputGeneratorLayer>,
    tx: SyncSender<EncoderCmd>,
    _thread: Option<JoinHandle<()>>,
    disabled: bool,
}

impl InputGenerator {
    pub fn new(layer_map_schemes: &LayerMapSchemeList, area_schemes: &AreaSchemeList,
            area_name: &str) -> CmnResult<InputGenerator> {
        let area_scheme = &area_schemes[area_name];
        let layer_map_scheme = &layer_map_schemes[area_scheme.layer_map_name()];
        let layer_schemes: Vec<&LayerScheme> = layer_map_scheme.layers().iter().map(|ls| ls).collect();

        let mut layers = HashMap::with_capacity(4);
        let mut lyr_addr_list = Vec::with_capacity(4);
        let mut lyr_dims_list = Vec::with_capacity(4);
        let mut lyr_axn_sigs_list = Vec::with_capacity(4);

        for layer_scheme in layer_schemes.into_iter() {
            let lyr_name = layer_scheme.name();
            let lyr_addr = LayerAddress::new(area_scheme.area_id(), layer_scheme.layer_id());
            let axn_topology = layer_scheme.kind().axn_topology();
            let lyr_depth = layer_scheme.depth().unwrap_or(cmn::DEFAULT_OUTPUT_LAYER_DEPTH);

            let dims = match axn_topology {
                AxonTopology::Spatial | AxonTopology::Nonspatial =>
                    Some(area_scheme.dims().clone_with_depth(lyr_depth)),
                // AxonTopology::Nonspatial => None,
                AxonTopology::None => panic!("InputGenerator::new: Invalid axon topology (None)."),
            };

            let lyr_axn_sig = match *layer_scheme.axon_domain() {
                AxonDomain::Output(ref axn_sig) => axn_sig.clone(),
                _ => return Err(format!("InputGenerator::new(): Input generator areas \
                    must contain only output layers. [area: '{}', layer: '{}']", area_scheme.name(),
                    layer_map_scheme.name()).into()),
            };

            lyr_addr_list.push(lyr_addr.clone());
            lyr_dims_list.push(dims.clone());
            lyr_axn_sigs_list.push(lyr_axn_sig.clone());

            let layer = InputGeneratorLayer {
                sub: SubcorticalNucleusLayer::new(lyr_name, lyr_addr, layer_scheme.axon_domain().clone(),
                    axn_topology, dims),
                pathway: None,
            };

            layers.insert(lyr_addr.clone(), layer);
        }

        let mut disabled = false;

        let encoder = match *area_scheme.get_encoder() {
            EncoderScheme::IdxStreamer { ref file_name, cyc_per, scale, loop_frames } => {
                assert_eq!(layers.len(), 1);
                let mut is = IdxStreamer::new(layers[&lyr_addr_list[0]].sub.dims().unwrap().clone(),
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
                let st = SensoryTract::new(layers[&lyr_addr_list[0]].sub.dims().unwrap());
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

        let (tx, rx) = mpsc::sync_channel(3);
        let thread_name = format!("InputGeneratorEncoder_{}", area_scheme.name());
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

        Ok(InputGenerator {
            area_id: area_scheme.area_id(),
            area_name: area_scheme.name().to_owned(),
            layers: layers,
            _thread: Some(thread_handle),
            tx: tx,
            disabled,
        })
    }

    // Specify a custom encoder tract. Input scheme must have been configured
    // as `EncoderScheme::Custom` in `AreaScheme`.
    pub fn set_encoder(&self, tract: Box<InputGeneratorTract>) {
        self.tx.send(EncoderCmd::SetEncoder(InputGeneratorEncoder::Custom(tract))).unwrap();
    }

    /// Writes input data into a tract.
    ///
    /// Blocks when the pathway (`TractSender`) `backpressure = true`.
    ///
    pub fn send_to_pathway(&self, layer: &InputGeneratorLayer, _work_pool: &mut WorkPool) {
        // println!("####### InputGenerator::send_to_pathway: self.disabled: {}", self.disabled);
        if !self.disabled {
            let pathway = layer.pathway.as_ref().expect("no pathway set");
            let future_write = match pathway.send().wait().unwrap() {
                Some(fw) => fw.write_u8(),
                None => panic!("tract wants to skip frame"),
            };

            self.tx.send(EncoderCmd::WriteInto {
                addr: layer.sub().addr(),
                dims: layer.sub.dims().unwrap().into(),
                future_write,
            }).unwrap();
        }
    }

    pub fn cycle_next(&self, _work_pool: &mut WorkPool) {
        if !self.disabled { self.tx.send(EncoderCmd::Cycle).unwrap(); }
    }

    pub fn set_encoder_ranges(&self, ranges: Vec<(f32, f32)>) {
        if !self.disabled { self.tx.send(EncoderCmd::SetRanges(ranges)).unwrap(); }
    }

    pub fn layers_mut(&mut self) -> &mut HashMap<LayerAddress, InputGeneratorLayer> {
        &mut self.layers
    }

    pub fn layer_addrs(&self) -> Vec<LayerAddress> {
        self.layers.iter().map(|(_, layer)| layer.sub.addr().clone()).collect()
    }

    pub fn area_id(&self) -> usize { self.area_id }
    pub fn area_name<'a>(&'a self) -> &'a str { &self.area_name }
    pub fn is_disabled(&self) -> bool { self.disabled }
}

impl Drop for InputGenerator {
    fn drop(&mut self) {
        self.tx.send(EncoderCmd::Exit).unwrap();
        self._thread.take().unwrap().join().unwrap();
    }
}

impl SubcorticalNucleus for InputGenerator {
    fn create_pathways(&mut self, thal: &mut Thalamus,
            _cortical_areas: &mut CorticalAreas) -> CmnResult<()> {
        for layer in self.layers.values_mut() {
            let tx = thal.input_pathway(layer.sub().addr(), true);
            layer.pathway = Some(tx);
        }
        Ok(())
    }

    fn pre_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            work_pool: &mut WorkPool) -> CmnResult<()> {
        for layer in self.layers.values() {
            self.send_to_pathway(layer, work_pool);
        }
        self.cycle_next(work_pool);
        Ok(())
    }

    fn post_cycle(&mut self, _thal: &mut Thalamus, _cortical_areas: &mut CorticalAreas,
            _work_pool: &mut WorkPool) -> CmnResult<()> {
        Ok(())
    }

    fn layer(&self, addr: LayerAddress) -> Option<&SubcorticalNucleusLayer> {
        self.layers.get(&addr).map(|l| l.sub())
    }

    fn area_name<'a>(&'a self) -> &'a str {
        &self.area_name
    }

    fn area_id(&self) -> usize {
        self.area_id
    }
}