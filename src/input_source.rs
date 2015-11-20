use std::iter;

use cmn::{ Sdr };
use ocl::{ EventList };
use proto::{ ProtoAreaMap, Protoinput };
use encode::{ IdxReader };


pub struct InputSource {
	area_name: &'static str,
	kind: InputSourceKind,
	targets: Vec<&'static str>, 
	ganglion: Vec<u8>,
}

impl InputSource {
	// [FIXME] Multiple output target areas disabled.
	pub fn new(proto_area_map: &ProtoAreaMap) -> InputSource {
		//let emsg = format!("\nInputSource::new(): No input source specified for area: '{}'", proto_area_map.name);
		let input = &proto_area_map.input;

		let (kind, targets, len) = match input {
			&Protoinput::IdxReader { file_name, cyc_per, scale } => {
				let ir = IdxReader::new(proto_area_map.dims.clone_with_depth(1), file_name, cyc_per, scale);				
				let len = ir.dims().cells();
				debug_assert!(proto_area_map.dims.columns() == len);

				( // RETURN TUPLE
					InputSourceKind::IdxReader(Box::new(ir)), 
					proto_area_map.aff_areas.clone(), 
					len
				)
			},

			&Protoinput::IdxReaderLoop { file_name, cyc_per, scale, loop_frames } => {
				let ir = IdxReader::new(proto_area_map.dims.clone_with_depth(1), file_name, cyc_per, scale)
					.loop_frames(loop_frames);
				let len = ir.dims().cells();
				debug_assert!(proto_area_map.dims.columns() == len);
				
				( // RETURN TUPLE
					InputSourceKind::IdxReader(Box::new(ir)), 
					proto_area_map.aff_areas.clone(), 
					len
				)
			},

			&Protoinput::None => (InputSourceKind::None, proto_area_map.aff_areas.clone(), 
				proto_area_map.dims.columns()),

			_ => panic!("\nInputSource::new(): Input type not yet supported."),
		};

		// [FIXME] Multiple output target areas disabled.
		assert!(targets.len() == 1, "Output to more or less than one area temporarily disabled. \
			Please create duplicate external source areas for now. Current source areas for '{}': {:?}.", 
			proto_area_map.name, targets);

		let ganglion = iter::repeat(0).take(len as usize).collect();

		InputSource {
			area_name: proto_area_map.name,
			kind: kind,
			targets: targets,
			ganglion: ganglion,			
		}
	}

	// [FIXME] Multiple output target areas disabled.
	pub fn next(&mut self, ganglion: &mut Sdr, events: &mut EventList) {
		// This is temp (mult out tar areas):
		debug_assert!(self.targets.len() == 1);
		// let dst_area_name = self.targets[0];

		// let mut ganglion = tract.ganglion_mut(self.area_name, map::AFFERENT_OUTPUT);

		match self.kind {
			InputSourceKind::IdxReader(ref mut ir) |
			InputSourceKind::IdxReaderLoop(ref mut ir)
				=> { let _ = ir.next(ganglion); },
				
			_ => (),
		}

		// for target in self.targets.iter() {
			// areas.get_mut(target).expect("InputSource::next(): Invalid area name, 'targets' mismatch error.")
			// 	.write_input(&self.ganglion, map::AFFERENT_INPUT);

			// println!("\n##### INPUTSOURCE::NEXT(): Writing ganglion with len: {} to area: '{}': \n{:?}", 
			// 	self.ganglion.len(), target, self.ganglion);
		// }
	}

	pub fn area_name(&self) -> &'static str {
		self.area_name
	}
}


pub enum InputSourceKind {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader(Box<IdxReader>),
	IdxReaderLoop(Box<IdxReader>),
	None,
}
