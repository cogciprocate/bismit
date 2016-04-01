use cortex::Cortex;
use map::{ProtolayerMaps, ProtoareaMaps};



pub struct Tyro {
	cortex: Cortex,
}

impl Tyro {
	pub fn new(plmaps: ProtolayerMaps, pamaps: ProtoareaMaps) -> Tyro {
		let cortex = Cortex::new(plmaps, pamaps);

		Tyro {
			cortex: cortex,
		}
	}

	pub fn cortex(&self) -> &Cortex {
		&self.cortex
	}
}