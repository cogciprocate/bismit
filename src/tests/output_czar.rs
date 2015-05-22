use cortex::{ Cortex };

























/* PRINT_SENSE_ONLY() & PRINT_SENSE_AND_PRINT():
	- TODO:
		- [incomplete][priority: low] Roll up into integrated command line system and make each item togglable
*/
pub fn print_sense_only(cortex: &mut Cortex) {
	if false {
		print!("\nAXON STATES: ");
		cortex.region_cells.axns.states.print_val_range(1 << 8, Some((1, 255)));
	}

	if false {
		print!("\nAXON REGION OUTPUT:");
		cortex.region_cells.axns.states.print((1 << 0) as usize, Some((1, 255)), Some(cortex.region_cells.cols.axn_output_range()), true);
	}
	if false {
		print!("\nCOLUMN SYNAPSE STRENGTHS:");
		cortex.region_cells.cols.syns.strengths.print(1 << 0, None, Some((256, 288)), true);
	}
	if false{	
		print!("\nCOLUMN SYNAPSE SOURCE COLUMN OFFSETS:");
		cortex.region_cells.cols.syns.src_col_x_offs.print(1 << 0, None, Some((256, 288)), true);
	}

	if false {
		print!("\nPYRAMIDAL DENDRITE SYNAPSE STRENGTHS:");
		cortex.region_cells.pyrs.dens.syns.strengths.print(1 << 0, None, Some((256, 319)), true);
	}
}


pub fn print_sense_and_print(cortex: &mut Cortex) {

	/* COLUMN, COLUMN SYNAPSE, COLUMN RAW STATES */
	if true {	
		print!("\nCOLUMN STATES: ");
		cortex.region_cells.cols.states.print_val_range(1 << 0, Some((1, 255)));
	}
	if false {	
		print!("\nCOLUMN STATES RAW: ");
		cortex.region_cells.cols.states_raw.print_val_range(1 << 0, Some((1, 255)));
	}
	if true {	
		print!("\nCOLUMN SYNAPSE STATES: ");
		cortex.region_cells.cols.syns.states.print(1 << 10, Some((1, 255)), None, true);
	}

		/*if true {	
			print!("\nCOLUMN SYNAPSE STATES: ");
			cortex.region_cells.cols.syns.states.print(1 << 3, Some((1, 255)), None, true);
		}*/

	if true {
		print!("\nCOLUMN SYNAPSE SOURCE ROW IDS:");
		cortex.region_cells.cols.syns.src_row_ids.print(1 << 11, None, None, true);
	}
		if false {
			print!("\nCOLUMN SYNAPSE SOURCE ROW IDS(0 - 1300):");
			cortex.region_cells.cols.syns.src_row_ids.print(1 << 0, None, Some((0, 1300)), true);
		}
	if true {	
		print!("\nCOLUMN SYNAPSE SOURCE COLUMN OFFSETS: ");
		cortex.region_cells.cols.syns.src_col_x_offs.print(1 << 11, None, None, true);
	}
	if true {
		print!("\nCOLUMN SYNAPSE STRENGTHS:");
		cortex.region_cells.cols.syns.strengths.print(1 << 11, None, None, true);
	}
	if false {
		print!("\nCOLUMN PEAK COL IDS: ");
		cortex.region_cells.cols.peak_spis.spi_ids.print_val_range(1 << 0, Some((0, 255)));
	}
	if false {
		print!("\nCOLUMN PEAK COL STATES: ");
		cortex.region_cells.cols.peak_spis.states.print_val_range(1 << 0, Some((1, 255)));
	}



	/* PYRAMIDAL */
	if true {	
		print!("\nPYRAMIDAL SYNAPSE STATES: ");
		cortex.region_cells.pyrs.dens.syns.states.print(1 << 16, Some((1, 255)), None, true);
	}	

		if false {	
			print!("\nPYRAMIDAL SYNAPSE STATES (all): ");
			cortex.region_cells.pyrs.dens.syns.states.print(1 << 0, Some((0, 255)), None, true);
			//print!("\nPYRAMIDAL SYNAPSE STATES (524288 - 524588): ");
			//cortex.region_cells.pyrs.dens.syns.states.print(1 << 1, Some((0, 255)), Some((524288, 524588)), true);
		}

	if true {	
		print!("\nPYRAMIDAL SYNAPSE SOURCE ROW IDS: ");
		cortex.region_cells.pyrs.dens.syns.src_row_ids.print(1 << 14, None, None, true);
	}

		if false {
			print!("\nPYRAMIDAL SYNAPSE SOURCE ROW IDS(0 - 1300):");
			cortex.region_cells.pyrs.dens.syns.src_row_ids.print(1 << 1, None, Some((0, 1300)), true);
		}

	if true {	
		print!("\nPYRAMIDAL SYNAPSE SOURCE COLUMN OFFSETS: ");
		cortex.region_cells.pyrs.dens.syns.src_col_x_offs.print(1 << 14, None, None, true);
	}
	if true {
		print!("\nPYRAMIDAL SYNAPSE STRENGTHS:");
		cortex.region_cells.pyrs.dens.syns.strengths.print(1 << 14, None, None, true);
	}
	if true {	
		print!("\nPYRAMIDAL SYNAPSE SOURCE FLAG SETS: ");
		cortex.region_cells.pyrs.dens.syns.flag_sets.print(1 << 14, None, None, true);
	}
	if true {	
		print!("\nPYRAMIDAL DENDRITE STATES: ");
		cortex.region_cells.pyrs.dens.states.print_val_range(1 << 10, Some((1, 255)));
	}
	if false {	
		print!("\nPYRAMIDAL DENDRITE STATES RAW: ");
		cortex.region_cells.pyrs.dens.states_raw.print_val_range(1 << 12, Some((1, 255)));
	}
	if false {
		print!("\nPYRAMIDAL AXON OUTPUT:");
		cortex.region_cells.axns.states.print((1 << 0) as usize, Some((1, 255)), Some(cortex.region_cells.pyrs.axn_output_range()), false);
	}
	if true {
		print!("\nPYRAMIDAL DEPOLARIZATIONS:");
		cortex.region_cells.pyrs.preds.print_val_range(1 << 0, Some((1, 255)));
	}



	/* AUX (DEBUG) */
	if true {
		print!("\naux.ints_0: ");
		//cortex.region_cells.aux.ints_0.print((1 << 12) as usize, Some((0, 17000)), None, false);
		cortex.region_cells.aux.ints_0.print((1 << 0) as usize, Some((0, 1023)), Some((1, 19783029)), false);
		print!("\naux.ints_1: ");
		cortex.region_cells.aux.ints_1.print((1 << 0) as usize, None, None, false);
	}
	if false {
		print!("\naux.chars_0: ");
		cortex.region_cells.aux.chars_0.print((1 << 0) as usize, Some((-128, 127)), None, true);
		print!("\naux.chars_1: ");
		cortex.region_cells.aux.chars_1.print((1 << 0) as usize, Some((-128, 127)), None, true);
	}



	/* AXON STATES (ALL) */
	if false {
		print!("\nAXON STATES: ");
		cortex.region_cells.axns.states.print((1 << 4) as usize, Some((1, 255)), None, true);

	}



	/* AXON REGION OUTPUT (L3) */
	if false {
		print!("\nAXON REGION OUTPUT (L3):");
		//cortex.region_cells.axns.states.print((1 << 0) as usize, Some((1, 255)), Some((3000, 4423)));
		cortex.region_cells.axns.states.print(
			(1 << 0) as usize, Some((0, 255)), 
			Some(cortex.region_cells.cols.axn_output_range()), 
			false
		);
	}

	print!("\n");

}
