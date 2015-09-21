
use proto::{ Protoarea, Protoinput };
use encode:: { IdxReader };


pub struct InputSource {
	kind: InputSourceKind,
	targets: Vec<&'static str>,	
}

impl InputSource {
	pub fn new(protoarea: &Protoarea) -> InputSource {
		let emsg = format!("\nInputSource::new(): No input source specified for area: '{}'", protoarea.name);
		let input = protoarea.input.clone().expect(&emsg);

		let (kind, targets) = match input {
			Protoinput::IdxReader { file_name, repeats } => {
				let ir = IdxReader::new(protoarea.dims.clone(), file_name, repeats);
				(InputSourceKind::IdxReader(Box::new(ir)), protoarea.aff_areas.clone())
			}
			_ => panic!("\nInputSource::new(): Input type not supported."),
		};

		InputSource {
			kind: kind,
			targets: targets,			
		}
	}
}


pub enum InputSourceKind {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader(Box<IdxReader>),
	None,
}
