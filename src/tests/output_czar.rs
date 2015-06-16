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
pub fn print_sense_only(cortex: &mut Cortex) {
	if false {
		print!("\nAXON STATES: ");
		cortex.area_mut("v1").axns.states.print_val_range(1 << 8, Some((1, 255)));
	}

	if false {
		print!("\nAXON REGION OUTPUT:");
		let mar = cortex.area_mut("v1").mcols.axn_output_range();
		cortex.area_mut("v1").axns.states.print((1 << 0) as usize, Some((1, 255)), Some(mar), true);
	}
	if false {
		print!("\nSPINY STELLATE SYNAPSE STRENGTHS:");
		cortex.area_mut("v1").psal_mut().dens.syns.strengths.print(1 << 0, None, Some((256, 288)), true);
	}
	if false{	
		print!("\nSPINY STELLATE SYNAPSE SOURCE SPINY STELLATE OFFSETS:");
		cortex.area_mut("v1").psal_mut().dens.syns.src_col_xy_offs.print(1 << 0, None, Some((256, 288)), true);
	}

	if false {
		print!("\nPYRAMIDAL DENDRITE SYNAPSE STRENGTHS:");
		cortex.area_mut("v1").ptal_mut().dens.syns.strengths.print(1 << 0, None, Some((256, 319)), true);
	}
}


pub fn print_sense_and_print(cortex: &mut Cortex) {

	/* SPINY STELLATE, SPINY STELLATE SYNAPSE, SPINY STELLATE RAW STATES */
			/*if true {	
			print!("\nSPINY STELLATE SYNAPSE STATES: ");
			cortex.area_mut("v1").psal_mut().dens.syns.states.print(1 << 3, Some((1, 255)), None, true);
		}*/
	if true {	
		print!("\nSPINY STELLATE SYNAPSE STATES: ");
		cortex.area_mut("v1").psal_mut().dens.syns.states.print(1 << 10, Some((1, 255)), None, true);
	}
	if true {
		print!("\nSPINY STELLATE SYNAPSE SOURCE ROW IDS:");
		cortex.area_mut("v1").psal_mut().dens.syns.src_slice_ids.print(1 << 11, None, None, true);
	}
		if false {
			print!("\nSPINY STELLATE SYNAPSE SOURCE ROW IDS(0 - 1300):");
			cortex.area_mut("v1").psal_mut().dens.syns.src_slice_ids.print(1 << 0, None, Some((0, 1300)), true);
		}
	if true {	
		print!("\nSPINY STELLATE SYNAPSE SOURCE SPINY STELLATE OFFSETS: ");
		cortex.area_mut("v1").psal_mut().dens.syns.src_col_xy_offs.print(1 << 11, None, None, true);
	}
	if true {
		print!("\nSPINY STELLATE SYNAPSE STRENGTHS:");
		cortex.area_mut("v1").psal_mut().dens.syns.strengths.print(1 << 11, None, None, true);
	}
	if false {
		print!("\nSPINY STELLATE PEAK COL IDS: ");
		//cortex.area_mut("v1").mcols.iinn.spi_ids.print_val_range(1 << 0, Some((0, 255)));
	}
	if false {
		print!("\nSPINY STELLATE PEAK COL STATES: ");
		//cortex.area_mut("v1").mcols.iinn.states.print_val_range(1 << 0, Some((1, 255)));
	}
	if true {	
		print!("\nSPINY STELLATE DENDRITE STATES: ");
		cortex.area_mut("v1").psal_mut().dens.states.print_val_range(1 << 0, Some((1, 255)));
	}
	if true {	
		print!("\nSPINY STELLATE DENDRITE STATES RAW: ");
		cortex.area_mut("v1").psal_mut().dens.states_raw.print_val_range(1 << 8, Some((1, 255)));
	}
	/*if true {	
		print!("\nSPINY STELLATE STATES: ");
		cortex.area_mut("v1").psal_mut().dens.states.print_val_range(1 << 0, Some((1, 255)));
	}*/
	if false {	
		print!("\nSPINY STELLATE STATES RAW: ");
		cortex.area_mut("v1").psal_mut().dens.states_raw.print_val_range(1 << 0, Some((1, 255)));
	}



	/* PYRAMIDAL */
	if true {	
		print!("\nPYRAMIDAL SYNAPSE STATES: ");
		cortex.area_mut("v1").ptal_mut().dens.syns.states.print(1 << 16, Some((1, 255)), None, true);
	}	

		if false {	
			print!("\nPYRAMIDAL SYNAPSE STATES (all): ");
			cortex.area_mut("v1").ptal_mut().dens.syns.states.print(1 << 0, Some((0, 255)), None, true);
			//print!("\nPYRAMIDAL SYNAPSE STATES (524288 - 524588): ");
			//cortex.area_mut("v1").ptal_mut().dens.syns.states.print(1 << 1, Some((0, 255)), Some((524288, 524588)), true);
		}

	if true {	
		print!("\nPYRAMIDAL SYNAPSE SOURCE ROW IDS: ");
		cortex.area_mut("v1").ptal_mut().dens.syns.src_slice_ids.print(1 << 14, None, None, true);
	}

		if false {
			print!("\nPYRAMIDAL SYNAPSE SOURCE ROW IDS(0 - 1300):");
			cortex.area_mut("v1").ptal_mut().dens.syns.src_slice_ids.print(1 << 1, None, Some((0, 1300)), true);
		}

	if true {	
		print!("\nPYRAMIDAL SYNAPSE SOURCE SPINY STELLATE OFFSETS: ");
		cortex.area_mut("v1").ptal_mut().dens.syns.src_col_xy_offs.print(1 << 14, None, None, true);
	}

	if false {
		print!("\nPYRAMIDAL SYNAPSE STRENGTHS:");
		cortex.area_mut("v1").ptal_mut().dens.syns.strengths.print(1 << 14, None, None, true);
	}

		if true {
		print!("\nPYRAMIDAL SYNAPSE STRENGTHS:");
		cortex.area_mut("v1").ptal_mut().dens.syns.strengths.print(1 << 0, Some((7, 127)), None, true);
	}

	if true {	
		print!("\nPYRAMIDAL SYNAPSE FLAG SETS: ");
		cortex.area_mut("v1").ptal_mut().dens.syns.flag_sets.print(1 << 14, None, None, true);
	}

	if true {	
		print!("\nPYRAMIDAL DENDRITE STATES: ");
		cortex.area_mut("v1").ptal_mut().dens.states.print_val_range(1 << 8, Some((1, 255)));
	}
	if true {	
		print!("\nPYRAMIDAL DENDRITE STATES RAW: ");
		cortex.area_mut("v1").ptal_mut().dens.states_raw.print_val_range(1 << 8, Some((1, 255)));
	}
	if false {
		print!("\nPYRAMIDAL AXON OUTPUT:");
		let air = cortex.area_mut("v1").ptal_mut().axn_range();
		cortex.area_mut("v1").axns.states.print((1 << 0) as usize, Some((1, 255)), Some(air), false);
	}

	if true {
		print!("\nPYRAMIDAL FLAG SETS:");
		cortex.area_mut("v1").ptal_mut().flag_sets.print_val_range(1 << 6, Some((1, 255)));
	}
	if true {
		print!("\nPYRAMIDAL DEPOLARIZATIONS:");
		cortex.area_mut("v1").ptal_mut().soma_mut().print_val_range(1 << 0, Some((1, 255)));
	}

	if true {
		print!("\nPYRAMIDAL DEPOLARIZATIONS (cortical_area_2):");
		cortex.area_mut("cortical_area_2").ptal_mut().soma_mut().print_val_range(1 << 0, Some((1, 255)));
	}



	/* AUX (DEBUG) */
	if true {
		/*print!("\naux.ints_0: ");
		cortex.area_mut("v1").aux.ints_0.print((1 << 0) as usize, None, Some((0, 700)), false);*/
		//print!("\naux.ints_0: ");
		//cortex.area_mut("v1").aux.ints_0.print((1 << 0) as usize, None, Some((0, 42767)), false);
		print!("\naux.ints_0: ");
		cortex.area_mut("v1").aux.ints_0.print((1 << 0) as usize, None, None, false);
		
		//cortex.area_mut("v1").aux.ints_0.print((1 << 0) as usize, Some((0, 1023)), Some((1, 19783029)), false);
		print!("\naux.ints_1: ");
		cortex.area_mut("v1").aux.ints_1.print((1 << 0) as usize, None, None, false);
	}
	if false {
		print!("\naux.chars_0: ");
		cortex.area_mut("v1").aux.chars_0.print((1 << 0) as usize, Some((-128, 127)), None, true);
		print!("\naux.chars_1: ");
		cortex.area_mut("v1").aux.chars_1.print((1 << 0) as usize, Some((-128, 127)), None, true);
	}



	/* AXON STATES (ALL) */
	if true {
		print!("\nAXON STATES: ");
		cortex.area_mut("v1").axns.states.print((1 << 4) as usize, Some((1, 255)), None, true);

	}



	/* AXON REGION OUTPUT (L3) */
	if false {
		print!("\nAXON REGION OUTPUT (L3):");
		//cortex.area_mut("v1").axns.states.print((1 << 0) as usize, Some((1, 255)), Some((3000, 4423)));
		let mar = cortex.area_mut("v1").mcols.axn_output_range();
		cortex.area_mut("v1").axns.states.print(
			(1 << 0) as usize, Some((0, 255)), 
			Some(mar), 
			false
		);
	}

	print!("\n");

}
