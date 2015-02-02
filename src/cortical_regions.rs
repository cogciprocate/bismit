// CorticalArea for the specifics
// CorticalRegion to define shit

use ocl;

use std::collections::{ HashMap };
use std::num;

pub struct CorticalRegion {
	pub layers: Vec<CorticalLayer>,
}

impl CorticalRegion {
	pub fn width() -> usize {
		0
	}

	pub fn row_count(&self) -> (usize, usize) {
		let mut antecell_rows = 0us;
		let mut cell_rows = 0us;
		
		for layer in self.layers.iter() {
			for row in layer.rows.iter() {
				match *row {
					CorticalRowClass::Interregional(_) => antecell_rows += 1,
					CorticalRowClass::Interlaminar(_) => cell_rows += 1,
				}
			}
		}
		(antecell_rows, cell_rows)
	}
}



#[derive(PartialEq, Eq, Show, Clone, Hash)]
pub enum CorticalRegionType {
	Associational,
	Sensory,
	Motor,
}


pub fn define() -> CorticalRegions {
	use self::CorticalRowClass::*;
	use self::CorticalAxonScope::*;
	use self::CorticalCellType::*;

	let mut cort_regs: CorticalRegions = HashMap::new();

	let sensory_region = CorticalRegion {
		layers: vec![
			CorticalLayer {
				name: "Input",
				rows: vec![
					Interregional(Thalamocortical),
				]
			},
			CorticalLayer {
				name: "Local",
				rows: vec![
					Interlaminar(Pyramidal),
					Interlaminar(Pyramidal),
				]
			}
		]
	};
	cort_regs.insert(CorticalRegionType::Sensory, sensory_region);

	cort_regs
}


struct CorticalLayer {
	name: &'static str,
	rows: Vec<CorticalRowClass>,
}

impl CorticalLayer {
	pub fn height(&self) -> ocl::cl_uchar {
		num::cast(self.rows.len()).unwrap()
	}
}


#[derive(PartialEq, Show, Clone)]
pub enum CorticalCellType {
	Pyramidal,
	SpinyStellate,
	AspinyStellate,
}
// excitatory spiny stellate
// inhibitory aspiny stellate 


#[derive(PartialEq, Show, Clone)]
pub enum CorticalAxonScope {
	Corticocortical,
	Thalamocortical,
	Corticothalamic,
	Corticospinal,
}


#[derive(PartialEq, Show, Clone)]
pub enum CorticalRowClass {
	Interregional (CorticalAxonScope),
	Interlaminar (CorticalCellType),
}


pub type CorticalRegions = HashMap<CorticalRegionType, CorticalRegion>;
