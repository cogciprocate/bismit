use cortex::{ Cortex };
use dendrites::{ Dendrites };
use pyramidals::{ Pyramidals };
use synapses::{ Synapses };
use common;

use std::io::{ self, Write, Stdout };
use std::fmt::{ Display, Debug, LowerHex, UpperHex };

pub fn run(cortex: &mut Cortex) {
	println!("Synapse source test running...");

	cortex.cells.pyrs.confab();
	cortex.cells.pyrs.dens.confab();
	cortex.cells.pyrs.dens.syns.confab();


	let pyrs = &cortex.cells.pyrs;
	let pyr_dens = &cortex.cells.pyrs.dens;
	let pyr_dens_syns = &cortex.cells.pyrs.dens.syns;

	shitty_print_pyrs(&pyrs, &pyr_dens, &pyr_dens_syns);

	io::stdout().flush().unwrap();

	//shitty_print_dens(&pyr_dens);
	

	// Print Active Dendrites Crudely




	// Print Active Dendrite Children Crudely
}

fn shitty_print_pyrs(pyrs: &Pyramidals, dens: &Dendrites, syns: &Synapses) {
	println!("\n");

	let mut pyr_idx = 0usize;

	//let width = pyrs.width;

	for pyr_depol in &pyrs.depols.vec {
		if *pyr_depol != 0 {
			let pyr_out_col_id = pyr_idx % pyrs.width() as usize;
			print!("\n[P:[{}({})]:{cp}{:02X}{cd}]", pyr_idx, pyr_out_col_id, pyr_depol, cp = common::C_PUR, cd = common::C_DEFAULT);
			shitty_print_dens(pyr_idx, dens, syns);
		}
		pyr_idx += 1;
	}

	//for den in vec

}

fn shitty_print_dens(pyr_idx: usize, dens: &Dendrites, syns: &Synapses) {
	let den_idx_base = pyr_idx << common::DENDRITES_PER_CELL_DISTAL_LOG2;
	let dens_per_cel = 1 << common::DENDRITES_PER_CELL_DISTAL_LOG2;

	for den_i in den_idx_base..(den_idx_base + dens_per_cel) {
		if dens.states.vec[den_i] != 0 {
			//print!("[DEN:]", , );
			print!("\n\t[{cd}D:[{}]{cg}:{cp}{:02X}]{cd}", den_i, dens.states.vec[den_i], cp = common::C_PUR, cd = common::C_DEFAULT, cg = common::C_DGR);
			shitty_print_syns(den_i, &syns);
		}

	}

	//for den in vec

}

fn shitty_print_syns(den_idx: usize, syns: &Synapses) {
	let syn_idx_base = den_idx << common::SYNAPSES_PER_DENDRITE_DISTAL_LOG2;
	let syns_per_den = 1 << common::SYNAPSES_PER_DENDRITE_DISTAL_LOG2;

	print!("\n\t\t");

	for syn_i in syn_idx_base..(syn_idx_base + syns_per_den) {
		if syns.states.vec[syn_i] != 0 {
				print!("{co}[S:[{cd}{}{co}]:{cp}{:02X}{co}]{cd}", syn_i, syns.states.vec[syn_i], co = common::C_ORA, cp = common::C_PUR, cd = common::C_DEFAULT);

		}

	}

	print!("\n");
}
