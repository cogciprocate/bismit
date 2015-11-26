// use std::iter;

use cmn::{ self, Sdr };
use ocl::{ EventList };
use proto::{ ProtoareaMap, Protoinput };
use encode::{ IdxReader };

pub trait InputGanglion {
	fn next(&mut self, ganglion: &mut Sdr) -> usize;
}

pub struct InputSource {
	area_name: &'static str,
	kind: InputSourceKind,
	// source: Box<InputGanglion>,
	// targets: Vec<&'static str>,
	depth: u8,
}

impl InputSource {
	// [FIXME] Multiple output target areas disabled.
	// [FIXME] Depricate targets? Is knowing targets useful? - Thalamus now handles this.
	pub fn new(pamap: &ProtoareaMap) -> InputSource {
		let input = &pamap.input;

		let (kind, /*targets,*/ len, depth) = match input {
			&Protoinput::IdxReader { file_name, cyc_per, scale } => {
				let ir = IdxReader::new(pamap.dims.clone_with_depth(1), file_name, cyc_per, scale);				
				let len = ir.dims().cells();
				debug_assert!(pamap.dims.columns() == len);

				( // RETURN TUPLE
					InputSourceKind::IdxReader(Box::new(ir)), 
					/*pamap.aff_areas().clone(),*/ // DEPRICATE
					len,
					cmn::DEFAULT_OUTPUT_LAYER_DEPTH,
				)
			},

			&Protoinput::IdxReaderLoop { file_name, cyc_per, scale, loop_frames } => {
				let ir = IdxReader::new(pamap.dims.clone_with_depth(1), file_name, cyc_per, scale)
					.loop_frames(loop_frames);
				let len = ir.dims().cells();
				debug_assert!(pamap.dims.columns() == len);
				
				( // RETURN TUPLE
					InputSourceKind::IdxReader(Box::new(ir)), 
					/*pamap.aff_areas().clone(),*/ // DEPRICATE
					len,
					cmn::DEFAULT_OUTPUT_LAYER_DEPTH,
				)
			},

			&Protoinput::None | &Protoinput::Zeros => (InputSourceKind::None, 
				/*pamap.aff_areas().clone(),*/ pamap.dims.columns(), 
				cmn::DEFAULT_OUTPUT_LAYER_DEPTH),

			_ => panic!("\nInputSource::new(): Input type not yet supported."),
		};

		// [FIXME] Multiple output target areas disabled.
		// assert!(targets.len() == 1, "Output to more or less than one area temporarily disabled. \
		// 	Please create duplicate external source areas for now. Current source areas for '{}': {:?}.", 
		// 	pamap.name, targets);

		InputSource {
			area_name: pamap.name,
			kind: kind,
			depth: depth,
			// targets: targets,
		}
	}

	// [FIXME] Multiple output target areas disabled.
	pub fn next(&mut self, ganglion: &mut Sdr, events: &mut EventList) {
		// This is temp (mult out tar areas): DEPRICATING: 
		// debug_assert!(self.targets.len() == 1);

		match self.kind {
			InputSourceKind::IdxReader(ref mut ig) |
			InputSourceKind::Custom(ref mut ig)
				=> { let _ = ig.next(ganglion); },
				
			_ => (),
		}
	}

	pub fn area_name(&self) -> &'static str {
		self.area_name
	}

	pub fn depth(&self) -> u8 {
		self.depth
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
