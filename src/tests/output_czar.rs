use std::iter;

use super::input_czar;
use cortex::{ Cortex };
use synapses::{ Synapses };
use cmn;


const EMSG: &'static str = "tests::output_czar::";





/* PRINT_SENSE_ONLY() & PRINT_SENSE_AND_PRINT():
	- TODO:
		- [incomplete][priority: low] Roll up into integrated command line system and make each item togglable
*/
pub fn print_sense_only(cortex: &mut Cortex, area_name: &str) {
	if false {
		println!("AXON STATES: ");
		cortex.area_mut(area_name).axns.states.print_val_range(1 << 8, Some((1, 255)));
	}

	if false {
		println!("AXON REGION OUTPUT:");
		let mar = cortex.area_mut(area_name).mcols.axn_output_range();
		cortex.area_mut(area_name).axns.states.print((1 << 0) as usize, Some((1, 255)), Some(mar), true);
	}
	if false {
		println!("SPINY STELLATE SYNAPSE STRENGTHS:");
		cortex.area_mut(area_name).psal_mut().dens_mut().syns.strengths.print(1 << 0, None, Some((256, 288)), true);
	}
	if false{	
		println!("SPINY STELLATE SYNAPSE SOURCE SPINY STELLATE OFFSETS:");
		cortex.area_mut(area_name).psal_mut().dens_mut().syns.src_col_v_offs.print(1 << 0, None, Some((256, 288)), true);
	}

	if false {
		println!("PYRAMIDAL DENDRITE SYNAPSE STRENGTHS:");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.strengths.print(1 << 0, None, Some((256, 319)), true);
	}
}


pub fn print_sense_and_print(cortex: &mut Cortex, area_name: &str) {

	/* SPINY STELLATE, SPINY STELLATE SYNAPSE, SPINY STELLATE RAW STATES */
			/*if true {	
			println!("SPINY STELLATE SYNAPSE STATES: ");
			cortex.area_mut(area_name).psal_mut().dens_mut().syns.states.print(1 << 3, Some((1, 255)), None, true);
		}*/
	if true {	
		println!("SPINY STELLATE SYNAPSE STATES: ");
		cortex.area_mut(area_name).psal_mut().dens_mut().syns.states.print(1 << 10, Some((1, 255)), None, true);
	}

	if true {
		println!("SPINY STELLATE SYNAPSE SOURCE ROW IDS:");
		cortex.area_mut(area_name).psal_mut().dens_mut().syns.src_slc_ids.print(1 << 11, None, None, true);
	} else if false {
		println!("SPINY STELLATE SYNAPSE SOURCE ROW IDS(0 - 1300):");
		cortex.area_mut(area_name).psal_mut().dens_mut().syns.src_slc_ids.print(1 << 0, None, Some((0, 1300)), true);
	}

	if true {	
		println!("SPINY STELLATE SYNAPSE SOURCE SPINY STELLATE OFFSETS: ");
		cortex.area_mut(area_name).psal_mut().dens_mut().syns.src_col_v_offs.print(1 << 11, None, None, true);
	}
	if true {
		println!("SPINY STELLATE SYNAPSE STRENGTHS:");
		cortex.area_mut(area_name).psal_mut().dens_mut().syns.strengths.print(1 << 11, None, None, true);
	}
	if false {
		println!("SPINY STELLATE PEAK COL IDS: ");
		//cortex.area_mut(area_name).mcols.iinn.spi_ids.print_val_range(1 << 0, Some((0, 255)));
	}
	if false {
		println!("SPINY STELLATE PEAK COL STATES: ");
		//cortex.area_mut(area_name).mcols.iinn.states.print_val_range(1 << 0, Some((1, 255)));
	}
	if true {	
		println!("SPINY STELLATE DENDRITE STATES: ");
		cortex.area_mut(area_name).psal_mut().dens_mut().states.print_val_range(1 << 0, Some((1, 255)));
	}
	if true {	
		println!("SPINY STELLATE DENDRITE STATES RAW: ");
		cortex.area_mut(area_name).psal_mut().dens_mut().states_raw.print_val_range(1 << 8, Some((1, 255)));
	}
	/*if true {	
		println!("SPINY STELLATE STATES: ");
		cortex.area_mut(area_name).psal_mut().dens_mut().states.print_val_range(1 << 0, Some((1, 255)));
	}*/
	if false {	
		println!("SPINY STELLATE STATES RAW: ");
		cortex.area_mut(area_name).psal_mut().dens_mut().states_raw.print_val_range(1 << 0, Some((1, 255)));
	}



	/* PYRAMIDAL */
	if true {	
		println!("PYRAMIDAL SYNAPSE STATES: ");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.states.print(1 << 12, Some((1, 255)), None, true);
	} else if false {	
		println!("PYRAMIDAL SYNAPSE STATES (all): ");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.states.print(1 << 0, Some((0, 255)), None, true);
		//println!("PYRAMIDAL SYNAPSE STATES (524288 - 524588): ");
		//cortex.area_mut(area_name).ptal_mut().dens_mut().syns.states.print(1 << 1, Some((0, 255)), Some((524288, 524588)), true);
	}

	if true {	
		println!("PYRAMIDAL SYNAPSE SOURCE SLICE IDS: ");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.src_slc_ids.print(1 << 11, None, None, true);
	} else if false {
		println!("PYRAMIDAL SYNAPSE SOURCE SLICE IDS(0 - 1300):");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.src_slc_ids.print(1 << 1, None, Some((0, 1300)), true);
	}

	if true {	
		println!("PYRAMIDAL SYNAPSE SOURCE COLUMN OFFSETS: ");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.src_col_v_offs.print(1 << 11, None, None, true);
	}

	if true {
		println!("PYRAMIDAL SYNAPSE STRENGTHS:");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.strengths.print(1 << 11, None, None, true);
	} else if false {
		println!("PYRAMIDAL SYNAPSE STRENGTHS:");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.strengths.print(1 << 0, Some((7, 127)), None, true);
	}

	if true {	
		println!("PYRAMIDAL SYNAPSE FLAG SETS: ");
		cortex.area_mut(area_name).ptal_mut().dens_mut().syns.flag_sets.print(1 << 14, None, None, true);
	}

	if true {	
		println!("PYRAMIDAL DENDRITE STATES: ");
		cortex.area_mut(area_name).ptal_mut().dens_mut().states.print_val_range(1 << 8, Some((1, 255)));
	}
	if true {	
		println!("PYRAMIDAL DENDRITE STATES RAW: ");
		cortex.area_mut(area_name).ptal_mut().dens_mut().states_raw.print_val_range(1 << 8, Some((1, 255)));
	}
	if false {
		println!("PYRAMIDAL AXON OUTPUT:");
		let air = cortex.area_mut(area_name).ptal_mut().axn_range();
		cortex.area_mut(area_name).axns.states.print((1 << 0) as usize, Some((1, 255)), Some(air), false);
	}

	if true {
		println!("PYRAMIDAL FLAG SETS:");
		cortex.area_mut(area_name).ptal_mut().flag_sets.print_val_range(1 << 6, Some((1, 255)));
	}
	if true {
		println!("PYRAMIDAL DEPOLARIZATIONS:");
		cortex.area_mut(area_name).ptal_mut().soma_mut().print_val_range(1 << 0, Some((1, 255)));
	}

	/*if true {
		println!("PYRAMIDAL DEPOLARIZATIONS (cortical_area_2):");
		cortex.area_mut("cortical_area_2").ptal_mut().soma_mut().print_val_range(1 << 0, Some((1, 255)));
	}
*/


	/* AUX (DEBUG) */
	if true {
		/*println!("aux.ints_0: ");
		cortex.area_mut(area_name).aux.ints_0.print((1 << 0) as usize, None, Some((0, 700)), false);*/
		//println!("aux.ints_0: ");
		//cortex.area_mut(area_name).aux.ints_0.print((1 << 0) as usize, None, Some((0, 42767)), false);
		println!("aux.ints_0: ");

		let view_radius = 1 << 24;
		cortex.area_mut(area_name).aux.ints_0.print((1 << 0) as usize, Some((0 - view_radius, view_radius)), None, true);
		
		//cortex.area_mut(area_name).aux.ints_0.print((1 << 0) as usize, Some((0, 1023)), Some((1, 19783029)), false);
		println!("aux.ints_1: ");
		cortex.area_mut(area_name).aux.ints_1.print((1 << 0) as usize, Some((0 - view_radius, view_radius)), None, true);
	}
	if false {
		println!("aux.chars_0: ");
		cortex.area_mut(area_name).aux.chars_0.print((1 << 0) as usize, Some((-128, 127)), None, true);
		println!("aux.chars_1: ");
		cortex.area_mut(area_name).aux.chars_1.print((1 << 0) as usize, Some((-128, 127)), None, true);
	}



	/* AXON STATES (ALL) */
	if false {
		println!("AXON STATES: ");
		cortex.area_mut(area_name).axns.states.print((1 << 8) as usize, Some((0, 255)), None, true);

	}



	/* AXON REGION OUTPUT (L3) */
	if false {
		println!("AXON REGION OUTPUT (L3):");
		//cortex.area_mut(area_name).axns.states.print((1 << 0) as usize, Some((1, 255)), Some((3000, 4423)));
		let mar = cortex.area_mut(area_name).mcols.axn_output_range();
		cortex.area_mut(area_name).axns.states.print(
			(1 << 0) as usize, Some((0, 255)), 
			Some(mar), 
			false
		);
	}

	println!("");

}
