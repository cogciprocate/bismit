use cortex::{ Cortex };
use dendrites::{ Dendrites };
use pyramidals::{ Pyramidals };
use columns::{ Columns };
use synapses::{ Synapses };
use common;

use std::io::{ self, Write, Stdout };
use std::fmt::{ Display, Debug, LowerHex, UpperHex };

pub fn print_pyrs(cortex: &mut Cortex) {
	let pyrs = &mut cortex.cells.pyrs;
	pyrs.confab();

	let mut pyr_idx = 0usize;
	let dens = &pyrs.dens;

	println!("\n");

	for pyr_depol in &pyrs.depols.vec {
		if *pyr_depol != 0 {
			//let pyr_out_col_id = pyr_idx % pyrs.width() as usize;
			let col_id = pyr_idx as isize & (common::SENSORY_CHORD_WIDTH - 1) as isize;
			print!("\n########## [P:[{}({})]:{cp}{:02X}{cd}] ##########", pyr_idx, col_id, pyr_depol, cp = common::C_PUR, cd = common::C_DEFAULT);
			shitty_print_dens(pyr_idx, dens);
		}
		pyr_idx += 1;
	}
	
	io::stdout().flush().unwrap();
}


fn shitty_print_dens(cel_idx: usize, dens: &Dendrites) {
	

	let den_idx_base = cel_idx << common::DENDRITES_PER_CELL_DISTAL_LOG2;
	let dens_per_cel = 1 << common::DENDRITES_PER_CELL_DISTAL_LOG2;

	let syns = &dens.syns;

	for den_i in den_idx_base..(den_idx_base + dens_per_cel) {
		if dens.states.vec[den_i] != 0 {
			//print!("[DEN:]", , );
			print!("\n%%%%% [{cd}D:[{}]{cg}:{cp}{:02X}]{cd} %%%%%", den_i, dens.states.vec[den_i], cp = common::C_PUR, cd = common::C_DEFAULT, cg = common::C_DGR);
			shitty_print_syns(cel_idx, den_i, &syns);
		}

	}

	//for den in vec

}

fn shitty_print_syns(cel_idx: usize, den_idx: usize, syns: &Synapses) {
	let syn_idx_base = den_idx << common::SYNAPSES_PER_DENDRITE_DISTAL_LOG2;
	let syns_per_den = 1 << common::SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	print!("\n");

	for syn_i in syn_idx_base..(syn_idx_base + syns_per_den) {
		if syns.states.vec[syn_i] != 0 {

				let width = syns.width() as isize;
				let col_id = cel_idx as isize & (common::SENSORY_CHORD_WIDTH - 1) as isize;
				let row_id = syns.src_row_ids.vec[syn_i] as isize;
				let col_ofs = syns.src_col_x_offs.vec[syn_i] as isize;

				let src_axn_addr = (width * row_id) + col_id + common::SYNAPSE_REACH as isize;

				let (src_axn_row, src_axn_col) = axn_coords(src_axn_addr, width);
				//print!("[width:{},col_id:{}]", width, col_id);

				print!("{cd}[[{cg}{}{cd}]{co}r:{},c:{}{cd}:{}({cp}{},{}{cd}){cd}:{cd}{:02X}{cd}]", syn_i, row_id, col_ofs, src_axn_addr, src_axn_row, src_axn_col, syns.states.vec[syn_i], cg = common::C_GRN, co = common::C_ORA, cp = common::C_PUR, cd = common::C_DEFAULT);

		}

	}

	print!("\n");
}

 



pub fn print_cols(cortex: &mut Cortex) {
	println!("Pyramidal synapse source test running...");

	let cols = &mut cortex.cells.cols;

	cols.confab();

	println!("\n");

	let col_idx_base = 0usize;
	let width = cortex.cells.width;

	let syns = &cols.syns;

	for col_i in 0..width as usize {
		if cols.states.vec[col_i] != 0 {
			//print!("[DEN:]", , );
			print!("\n########## [{cd}C:[{}]{cg}:{cp}{:02X}]{cd} ##########", col_i, cols.states.vec[col_i], cp = common::C_PUR, cd = common::C_DEFAULT, cg = common::C_DGR);
			shitty_print_col_syns(col_i, col_i, &syns);
		}
	}

	io::stdout().flush().unwrap();
}




fn shitty_print_col_syns(cel_idx: usize, den_idx: usize, syns: &Synapses) {
	let syn_idx_base = den_idx << common::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
	let syns_per_den = 1 << common::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;

	print!("\n");

	for syn_i in syn_idx_base..(syn_idx_base + syns_per_den) {
		if syns.states.vec[syn_i] != 0 {

				let width = syns.width() as isize;
				let col_id = cel_idx as isize & (common::SENSORY_CHORD_WIDTH - 1) as isize;
				let row_id = syns.src_row_ids.vec[syn_i] as isize;
				let col_ofs = syns.src_col_x_offs.vec[syn_i] as isize;

				let src_axn_addr = (width * row_id) + col_id + col_ofs + common::SYNAPSE_REACH as isize;

				let (src_axn_row, src_axn_col) = axn_coords(src_axn_addr, width);
				//print!("[width:{},col_id:{}]", width, col_id);

				print!("{cd}[[{cg}{}{cd}]{co}r:{},c:{}{cd}:{}({cp}{},{}{cd}){cd}:{cd}{:02X}{cd}]", syn_i, row_id, col_ofs, src_axn_addr, src_axn_row, src_axn_col, syns.states.vec[syn_i], cg = common::C_GRN, co = common::C_ORA, cp = common::C_PUR, cd = common::C_DEFAULT);

		}

	}

	print!("\n");
}



fn axn_coords(axn_addr: isize, width: isize) -> (isize, isize) {
	let axn_true = axn_addr - (common::SYNAPSE_REACH as isize);

	let axn_row = axn_true >> common::SENSORY_CHORD_WIDTH_LOG2;
	let axn_col = axn_true % width;

	(axn_row, axn_col)
}
