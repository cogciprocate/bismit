


#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum CellType {
	Pyramidal,
	SpinyStellate,
	AspinyStellate,
}
// excitatory spiny stellate
// inhibitory aspiny stellate 

struct CellProperties {
	dens_per_cell: u32,
	syns_per_den: u32, 
}
