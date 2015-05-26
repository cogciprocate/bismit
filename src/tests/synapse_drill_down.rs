use cortex::{ Cortex };
use dendrites::{ Dendrites };
use pyramidals::{ Pyramidal };
use minicolumns::{ MiniColumns };
use synapses::{ Synapses };
use cmn;

use std::io::{ self, Write, Stdout };
use std::fmt::{ Display, Debug, LowerHex, UpperHex };

pub fn print_pyrs(cortex: &mut Cortex) {
	let pyrs = &mut cortex.cortical_area.pyrs;
	pyrs.confab();

	let columns = cortex.cortical_area.dims.columns();

	let mut pyr_idx = 0usize;
	let dens = &pyrs.dens;

	println!("\n");

	for pyr_pred in &pyrs.preds.vec {
		if *pyr_pred != 0 {
			//let pyr_out_col_id = pyr_idx % pyrs.dims.columns() as usize;
			let col_id = pyr_idx as isize & (columns - 1) as isize;
			print!("\n########## [P:[{}({})]:{cp}{:02X}{cd}] ##########", pyr_idx, col_id, pyr_pred, cp = cmn::C_PUR, cd = cmn::C_DEFAULT);
			shitty_print_dens(pyr_idx, dens);
		}
		pyr_idx += 1;
	}
	
	io::stdout().flush().unwrap();
}


fn shitty_print_dens(cel_idx: usize, dens: &Dendrites) {
	

	let den_idx_base = cel_idx << cmn::DENDRITES_PER_CELL_DISTAL_LOG2;
	let dens_per_cel = 1 << cmn::DENDRITES_PER_CELL_DISTAL_LOG2;

	let syns = &dens.syns;

	for den_i in den_idx_base..(den_idx_base + dens_per_cel) {
		if dens.states.vec[den_i] != 0 {
			//print!("[DEN:]", , );
			print!("\n%%%%% [{cd}D:[{}]{cg}:{cp}{:02X}]{cd} %%%%%", den_i, dens.states.vec[den_i], cp = cmn::C_PUR, cd = cmn::C_DEFAULT, cg = cmn::C_DGR);
			shitty_print_syns(cel_idx, den_i, &syns);
		}

	}

	//for den in vec

}

fn shitty_print_syns(cel_idx: usize, den_idx: usize, syns: &Synapses) {
	let syn_idx_base = den_idx << cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2;
	let syns_per_den = 1 << cmn::SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	print!("\n");

	for syn_i in syn_idx_base..(syn_idx_base + syns_per_den) {
		if syns.states.vec[syn_i] != 0 {

				let columns = syns.dims.columns() as isize;
				let col_id = cel_idx as isize & (cmn::SENSORY_CHORD_COLUMNS - 1) as isize;
				let slice_id = syns.src_slice_ids.vec[syn_i] as isize;
				let col_ofs = syns.src_col_xy_offs.vec[syn_i] as isize;

				let src_axn_addr = (columns * slice_id) + col_id + cmn::SYNAPSE_REACH_LIN as isize;

				let (src_axn_slice, src_axn_col) = axn_coords(src_axn_addr, columns);
				//print!("[columns:{},col_id:{}]", columns, col_id);

				print!("{cd}[[{cg}{}{cd}]{co}r:{},c:{}{cd}:{}({cp}{},{}{cd}){cd}:{cd}{:02X}{cd}]", syn_i, slice_id, col_ofs, src_axn_addr, src_axn_slice, src_axn_col, syns.states.vec[syn_i], cg = cmn::C_GRN, co = cmn::C_ORA, cp = cmn::C_PUR, cd = cmn::C_DEFAULT);

		}

	}

	print!("\n");
}

 



pub fn print_mcols(cortex: &mut Cortex) {
	println!("Pyramidal synapse source test running...");

	let mcols = &mut cortex.cortical_area.mcols;

	mcols.confab();

	println!("\n");

	let col_idx_base = 0usize;
	let columns = cortex.cortical_area.dims.columns();

	let syns = &mcols.dens.syns;

	for col_i in 0..columns as usize {
		if mcols.dens.states.vec[col_i] != 0 {
			//print!("[DEN:]", , );
			print!("\n########## [{cd}C:[{}]{cg}:{cp}{:02X}]{cd} ##########", col_i, mcols.dens.states.vec[col_i], cp = cmn::C_PUR, cd = cmn::C_DEFAULT, cg = cmn::C_DGR);
			shitty_print_col_syns(col_i, col_i, &syns);
		}
	}

	io::stdout().flush().unwrap();
}




fn shitty_print_col_syns(cel_idx: usize, den_idx: usize, syns: &Synapses) {
	let syn_idx_base = den_idx << cmn::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;
	let syns_per_den = 1 << cmn::SYNAPSES_PER_DENDRITE_PROXIMAL_LOG2;

	print!("\n");

	for syn_i in syn_idx_base..(syn_idx_base + syns_per_den) {
		if syns.states.vec[syn_i] != 0 {

				let columns = syns.dims.columns() as isize;
				let col_id = cel_idx as isize & (cmn::SENSORY_CHORD_COLUMNS - 1) as isize;
				let slice_id = syns.src_slice_ids.vec[syn_i] as isize;
				let col_ofs = syns.src_col_xy_offs.vec[syn_i] as isize;

				let src_axn_addr = (columns * slice_id) + col_id + col_ofs + cmn::SYNAPSE_REACH_LIN as isize;

				let (src_axn_slice, src_axn_col) = axn_coords(src_axn_addr, columns);
				//print!("[columns:{},col_id:{}]", columns, col_id);

				print!("{cd}[[{cg}{}{cd}]{co}r:{},c:{}{cd}:{}({cp}{},{}{cd}){cd}:{cd}{:02X}{cd}]", syn_i, slice_id, col_ofs, src_axn_addr, src_axn_slice, src_axn_col, syns.states.vec[syn_i], cg = cmn::C_GRN, co = cmn::C_ORA, cp = cmn::C_PUR, cd = cmn::C_DEFAULT);

		}

	}

	print!("\n");
}



fn axn_coords(axn_addr: isize, columns: isize) -> (isize, isize) {
	let axn_true = axn_addr - (cmn::SYNAPSE_REACH_LIN as isize);

	let axn_slice = axn_true >> cmn::SENSORY_CHORD_COLUMNS_LOG2;
	let axn_col = axn_true % columns;

	(axn_slice, axn_col)
}
