use std::iter;

use cortical_area:: { /*CorticalArea,*/ CorticalAreas };
// use cortex::{ Cortex };
use proto::{ layer, ProtoAreaMap, Protoinput };
use encode:: { IdxReader };


pub struct InputSource {
	kind: InputSourceKind,
	targets: Vec<&'static str>, 
	ganglion: Vec<u8>,
}

impl InputSource {
	pub fn new(proto_area_map: &ProtoAreaMap) -> InputSource {
		//let emsg = format!("\nInputSource::new(): No input source specified for area: '{}'", proto_area_map.name);
		let input = &proto_area_map.input;

		let (kind, targets, len) = match input {
			&Protoinput::IdxReader { file_name, repeats, scale } => {
				let ir = IdxReader::new(proto_area_map.dims.clone_with_depth(1), file_name, repeats, scale);
				let len = ir.dims().cells();
				( // RETURN TUPLE
					InputSourceKind::IdxReader(Box::new(ir)), 
					proto_area_map.aff_areas.clone(), 
					len
				)
			},

			&Protoinput::IdxReaderLoop { file_name, repeats, scale, loop_frames } => {
				let ir = IdxReader::new(proto_area_map.dims.clone_with_depth(1), file_name, repeats, scale)
					.loop_frames(loop_frames);
				let len = ir.dims().cells();
				( // RETURN TUPLE
					InputSourceKind::IdxReader(Box::new(ir)), 
					proto_area_map.aff_areas.clone(), 
					len
				)
			},

			&Protoinput::None => (InputSourceKind::None, vec![], 0),

			_ => panic!("\nInputSource::new(): Input type not yet supported."),
		};

		let ganglion = iter::repeat(0).take(len as usize).collect();

		InputSource {
			kind: kind,
			targets: targets,
			ganglion: ganglion,			
		}
	}

	pub fn next(&mut self, areas: &CorticalAreas) {		
		match self.kind {
			InputSourceKind::IdxReader(ref mut ir) => { let _ = ir.next(&mut self.ganglion[..]); },
			InputSourceKind::IdxReaderLoop(ref mut ir) => { let _ = ir.next(&mut self.ganglion[..]); },
			_ => (),
		}

		for target in self.targets.iter() {
			areas[target].write_input(&self.ganglion, layer::AFFERENT_INPUT);

			// println!("\n##### INPUTSOURCE::NEXT(): Writing ganglion with len: {} to area: '{}': \n{:?}", 
			// 	self.ganglion.len(), target, self.ganglion);
		}
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
