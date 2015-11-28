// use std::iter;
use std::collections::{ HashMap };

use map::{ self, LayerTags };
use cmn::{ self, Sdr, CorticalDims };
use ocl::{ EventList };
use proto::{ ProtoareaMap, Protoinput, ProtolayerMap, Protolayer, AxonKind };
use encode::{ IdxReader };

pub type InputSources = HashMap<(&'static str, LayerTags), InputSource>;

pub trait InputGanglion {
	fn cycle(&mut self, ganglion: &mut Sdr) -> usize;
}

pub struct InputSource {
	area_name: &'static str,
	layer_tags: LayerTags,
	kind: InputSourceKind,
	layer_name: &'static str,
	axn_kind: AxonKind,
	dims: CorticalDims,
	// source: Box<InputGanglion>,
}

impl InputSource {
	// [FIXME] Determine (or have passed in) the layer depth corresponding to this source.
	pub fn new(pamap: &ProtoareaMap, plmap: &ProtolayerMap) -> InputSource {
		let input = &pamap.input;

		let layers: Vec<&Protolayer> = plmap.layers().iter().map(|(_, pl)| pl).collect();

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

		let kind = match input {
			&Protoinput::IdxReader { file_name, cyc_per, scale } => {
				let ir = IdxReader::new(dims.clone(), file_name, 
					cyc_per, scale);				
				InputSourceKind::IdxReader(Box::new(ir))
			},

			&Protoinput::IdxReaderLoop { file_name, cyc_per, scale, loop_frames } => {
				let ir = IdxReader::new(dims.clone(), file_name, 
					cyc_per, scale).loop_frames(loop_frames);				
				InputSourceKind::IdxReader(Box::new(ir))
			},

			&Protoinput::None | &Protoinput::Zeros => InputSourceKind::None,

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
	pub fn cycle(&mut self, ganglion: &mut Sdr, events: &mut EventList) {
		// This is temp (mult out tar areas): DEPRICATING: 
		// debug_assert!(self.targets.len() == 1);

		match self.kind {
			InputSourceKind::IdxReader(ref mut ig) |
			InputSourceKind::Custom(ref mut ig)
				=> { let _ = ig.cycle(ganglion); },
				
			_ => (),
		}
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

	// pub fn depth(&self) -> u8 {
	// 	self.depth
	// }

	pub fn dims(&self) -> &CorticalDims {
		&self.dims
	}
}


pub enum InputSourceKind {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader(Box<InputGanglion>),
	// IdxReaderLoop(Box<InputGanglion>),
	Custom(Box<InputGanglion>),
	None,
}
