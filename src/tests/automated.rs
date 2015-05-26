
use super::input_czar::{ self, InputCzar, InputVecKind };
//use cortex::Cortex;
use proto::*;
//use proto::areas::{ ProtoAreas, ProtoAreasTrait };
//use proto::regions::{ ProtoRegions, ProtoRegion, ProtoRegionKind };
use synapses::{ Synapses };
use cortex::{ self, Cortex };
use cmn;
use ocl;

use std::ops::{ Range };
use std::iter;


const TOGGLE_DIRS: bool 				= true;
const INTRODUCE_NOISE: bool 			= false;
const COUNTER_RANGE: Range<usize>		= Range { start: 0, end: 10 };
const COUNTER_RANDOM: bool				= false;

pub fn define_prtrgns() -> ProtoRegions {
	ProtoRegions::new()
		.region(ProtoRegion::new(ProtoRegionKind::Sensory)
			.layer("thal", 1, layer::DEFAULT, Axonal(Spatial))
			.layer("out", 1, layer::COLUMN_OUTPUT, Axonal(Spatial))
			.layer("iv", 1, layer::COLUMN_INPUT, Protocell::new_spiny_stellate(vec!["thal", "thal", "thal", "motor"]))  // , "motor"
			.layer("iii", 4, layer::DEFAULT, Protocell::new_pyramidal(vec!["iii"]))
			.layer("motor", 1, layer::DEFAULT, Axonal(Horizontal))
			.freeze()
		)
}

pub fn define_prtareas() -> ProtoAreas {
	ProtoAreas::new().area("v1", 6, 6, ProtoRegionKind::Sensory)
}


/* IDEAS FOR TESTS:
	- set synapse src_ids, src_ofs, strs to 0
		- test some specific inputs and make sure that synapses are responding exactly


*/
#[test]
fn test_cortex() {
	let mut cortex = Cortex::new(define_prtrgns(), define_prtareas());
					/* 	 InputCzar::new(columns, vec_kind, counter_range, counter_random, toggle_dirs, introduce_noise) */
	//let mut input_czar = InputCzar::new(cmn::SENSORY_CHORD_COLUMNS, InputVecKind::Band_512, 0..10, false, false, false);
	//input_czar.next(&mut cortex);

	check_cycles(&mut cortex);

	cortex.release_components();
}



pub fn check_cycles(cortex: &mut Cortex) {
	let columns = cortex.cortical_area.dims.columns();
	let mut vec1 = iter::repeat(0).take(columns as usize).collect();

	input_czar::vec_band_512_fill(&mut vec1);
	cortex.write_vec(0, "thal", &vec1);
	cortex.write_vec(0, "iii", &vec1);

	cortex.cortical_area.mcols.cycle(false);
	cortex.cortical_area.pyrs.cycle();

		//#####  TRY THIS OUT SOMETIME  #####
	//let pyrs_input_len = cortex.cortical_area.pyrs.len();
	//let mut vec_pyrs = iter::repeat(0).take().collect();
	//input_czar::vec_band_512_fill(&mut vec_pyrs);
	//let pyr_axn_ranges = cortex.cortical_area.layer_input_ranges("iii", cortex.cortical_area.pyrs.dens.syns.den_kind());
	//write_to_axons(axn_range, vec1);

	let mut syns = &mut cortex.cortical_area.mcols.dens.syns;
	check_synapses(&mut syns);

	let mut syns = &mut cortex.cortical_area.pyrs.dens.syns;
	check_synapses(&mut syns);
	
}

fn check_synapses(syns: &mut Synapses) {
	let syns_per_cel_l2: usize = syns.dims.per_cel_l2_left().unwrap() as usize;
	let cels_per_band: usize = cmn::SYNAPSE_SPAN_LIN as usize;
	let syns_per_band: usize = cels_per_band << syns_per_cel_l2;
	let syn_actv_band_thresh = syns_per_band / 4;

	let mut cel_idz: usize = 0;
	let mut states_ttl: usize = 0;

	syns.confab();

	loop {
		if (cel_idz + cels_per_band) > syns.dims.cells() as usize {
			break;
		}

		states_ttl = 0;

		let syn_idz = cel_idz << syns_per_cel_l2;

		//println!("\nsyn_idz: {}, syns_per_cel: {}, syns_per_band: {}", syn_idz, 1 << syns_per_cel_l2, syns_per_band);

		for syn_idx in syn_idz..(syn_idz + syns_per_band) {
			states_ttl += (syns.states[syn_idx] >> 7) as usize;
		}

		if (cel_idz & 512) == 0 {
			assert!(states_ttl < syn_actv_band_thresh);
		} else {
			assert!(states_ttl > syn_actv_band_thresh);
		}

		print!("\nPASS [{} - {}], states_ttl: {}", cel_idz, (cel_idz + cels_per_band - 1), states_ttl);


		cel_idz += cels_per_band;
	}
}
