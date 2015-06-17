use std::ops::{ Range };
use std::collections::{ HashMap };
use std::iter;

use proto::areas::{ self, Protoareas, ProtoareasTrait, Protoarea };


pub struct Thalamus {
	concourse: Vec<u8>,
	index: Vec<AreaInfo>, // POSSIBLY CONVERT TO ARRAY FOR SIMPLICITY
	map: HashMap<&'static str, usize>,
}

impl Thalamus {
	pub fn new(protoareas: &Protoareas) -> Thalamus {
		let mut index = Vec::with_capacity(protoareas.len());
		let mut map = HashMap::with_capacity(protoareas.len());
		let mut cc_len = 0usize;
		let mut i = 0usize;

		for (&pa_name, pa) in protoareas {
			//print!("\nTHALAMUS::NEW(): Adding area: '{}'", pa_name);

			let pa_len = (pa.dims.width() * pa.dims.height()) as usize;

			index.push(
				AreaInfo {
					cc_range: cc_len..(cc_len + pa_len),
					protoarea: pa.clone(),
				}
			);

			cc_len += pa_len;

			assert!(index[i].protoarea.name == pa_name);

			map.insert(pa.name, i);

			i += 1;
		}

		let concourse: Vec<u8> = iter::repeat(0).take(cc_len).collect();

		//print!("\n\n##### THALAMUS::NEW(): \n\n    INDEX: {:?}\n\n    MAP: {:?}\n\n    CONCOURSE.LEN(): {}", index, map, concourse.len());


		Thalamus {
			concourse: concourse,
			index: index,
			map: map,
		}
	}

	pub fn something(){}
}

#[derive(PartialEq, Debug, Clone, Eq)]
struct AreaInfo {
	cc_range: Range<usize>,
	protoarea: Protoarea,
}



