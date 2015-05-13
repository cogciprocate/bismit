use cmn;
use ocl::{ self, Ocl, WorkSize };
use ocl::{ Envoy };
use proto::areas::{ ProtoAreas, Width };
use proto::regions::{ ProtoRegion, ProtoRegionKind };
use proto::layer::{ ProtoLayer, ProtoLayerKind };
use proto::cell::{ CellKind, Protocell, DendriteKind };
use dendrites::{ Dendrites };
use axons::{ Axons };
use region_cells::{ Aux };

use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng, Rng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };


/* Synapses: Smallest and most numerous unit in the cortex - the soldier behind it all
	- TODO:
		- [high priority] Testing: 
			- Top priority is checking for uniqueness and correct distribution frequency among src_rows and cols

		- [low priority] Optimization:
			- Obviously grow() and it's ilk need a lot of work

*/

pub struct Synapses {
	depth: u8,
	width: u32,
	per_cell_l2: u32,
	per_den_l2: u32,
	den_kind: DendriteKind,
	cell_kind: CellKind,
	since_decay: usize,
	kern_cycle: ocl::Kernel,
	kern_regrow: ocl::Kernel,
	rng: rand::XorShiftRng,
	pub states: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_char>,
	pub src_row_ids: Envoy<ocl::cl_uchar>,
	pub src_col_x_offs: Envoy<ocl::cl_char>,
	//pub src_col_y_offs: Envoy<ocl::cl_char>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	pub row_pool: Envoy<ocl::cl_uchar>,
}

impl Synapses {
	pub fn new(width: u32, depth: u8, per_cell_l2: u32, per_den_l2: u32, den_kind: DendriteKind, cell_kind: CellKind, 
					region: &ProtoRegion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> Synapses {

		let syns_per_row = width << per_cell_l2;
		let wg_size = cmn::SYNAPSES_WORKGROUP_SIZE;


		let row_pool = Envoy::new(cmn::SYNAPSE_ROW_POOL_SIZE, 1, 0, ocl);


		//print!("\nNew {:?} Synapses with: depth: {}, width: {}, per_cell_l2: {}, syns_per_row(row area): {}", den_kind, depth, width, per_cell_l2, syns_per_row);

		let states = Envoy::<ocl::cl_uchar>::new(syns_per_row, depth, 0, ocl);
		let strengths = Envoy::<ocl::cl_char>::new(syns_per_row, depth, 0, ocl);
		let mut src_row_ids = Envoy::<ocl::cl_uchar>::new(syns_per_row, depth, 0, ocl);

		// SRC COL REACHES SHOULD BECOME CONSTANTS
		let mut src_col_x_offs = Envoy::<ocl::cl_char>::shuffled(syns_per_row, depth, -126, 126, ocl); 
		//let mut src_col_y_offs = Envoy::<ocl::cl_char>::shuffled(syns_per_row, depth, -31, 31, ocl);

		let flag_sets = Envoy::<ocl::cl_uchar>::new(syns_per_row, depth, 0, ocl);


		let mut kern_cycle = ocl.new_kernel("syns_cycle", 
			WorkSize::TwoDim(depth as usize, width as usize))
			.lws(WorkSize::TwoDim(1 as usize, wg_size as usize))
			.arg_env(&axons.states)
			.arg_env(&src_col_x_offs)
			.arg_env(&src_row_ids)
			//.arg_env(&strengths)
			.arg_scl(per_cell_l2)
			.arg_env(&aux.ints_0)
			//.arg_env(&aux.ints_1)
			.arg_env(&states)
		;

		//println!("\n### Defining kern_regrow with len: {} ###", depth as usize * syns_per_row as usize);

		let mut kern_regrow = ocl.new_kernel("syns_regrow", 
			WorkSize::TwoDim(depth as usize, syns_per_row as usize))
			//.lws(WorkSize::TwoDim(1 as usize, wg_size as usize))
			.arg_env(&strengths)
			.arg_scl(per_den_l2)
			.arg_scl_named(0u32, "rnd")
			//.arg_env(&aux.ints_0)
			//.arg_env(&aux.ints_1)
			.arg_env(&src_col_x_offs)
			.arg_env(&src_row_ids)
		;

		//println!("\n### Test S1 ###");

		let mut syns = Synapses {
			width: width,
			depth: depth,
			per_cell_l2: per_cell_l2,
			per_den_l2: per_den_l2,
			den_kind: den_kind,
			cell_kind: cell_kind,
			since_decay: 0,
			kern_cycle: kern_cycle,
			kern_regrow: kern_regrow,
			rng: rand::weak_rng(),
			states: states,
			strengths: strengths,
			src_row_ids: src_row_ids,
			src_col_x_offs: src_col_x_offs,
			//src_col_y_offs: src_col_y_offs,
			flag_sets: flag_sets,
			row_pool: row_pool,
		};

		syns.grow(region, true);
		//syns.refresh_row_pool();

		syns
	}

	fn grow(&mut self, region: &ProtoRegion, init: bool) {
		assert!(
			(self.src_col_x_offs.width() == self.src_row_ids.width()) 
			&& ((self.src_row_ids.width() == (self.width << self.per_cell_l2))), 
			"[region_cells::Synapses::init(): width mismatch]"
		);

		self.confab();

		let syns_per_row = self.width << self.per_cell_l2;
		//let mut rng = rand::weak_rng();

		/* LOOP THROUGH ALL LAYERS */
		for (&layer_name, layer) in region.layers().iter() {
			let src_row_ids: Vec<u8> = match layer.kind {
				ProtoLayerKind::Cellular(ref cell) => {
					if cell.cell_kind == self.cell_kind {
						region.src_row_ids(layer_name, self.den_kind)
					} else {
						continue
					}
				},
				_ => continue,
			};

			let row_ids = region.row_ids(vec!(layer_name));
			let src_row_ids_len: usize = src_row_ids.len();

			assert!(src_row_ids_len > 0, "Synapses must have at least one source row");

			let kind_base_row_pos = layer.kind_base_row_pos;
			let src_row_idx_range: Range<usize> = Range::new(0, src_row_ids_len);
			let src_col_x_offs_range: Range<i8> = Range::new(-126, 127);
			let strength_init_range: Range<i8> = Range::new(-3, 4);
			
			assert!(src_row_ids_len <= (1 << self.per_cell_l2) as usize, "region_cells::Synapses::init(): Number of source rows must not exceed number of synapses per cell.");

			if init {
				print!("\n    syns.init(): \"{}\" ({:?}): row_ids: {:?}, src_row_ids: {:?}", layer_name, self.den_kind, row_ids, src_row_ids);
			}

			/* LOOP THROUGH ROWS (WITHIN LAYER) */
			for row_pos in kind_base_row_pos..(kind_base_row_pos + layer.depth) {
				//print!("\nDEBUG: row_pos: {}", row_pos);
				//print!("\nDEBUG: syns_per_row: {}", syns_per_row);

				let ei_start = syns_per_row as usize * row_pos as usize;
				//print!("\nDEBUG: ei_start: {}", ei_start);

				let ei_end = ei_start + syns_per_row as usize;
				//print!("\nDEBUG: ei_end: {}", ei_end);
				//print!("\nDEBUG: src_row_ids: {:?}", src_row_ids);

				//print!("\n   Row {}: ei_start: {}, ei_end: {}, src_row_ids: {:?}", row_pos, ei_start, ei_end, src_row_ids);

				/* LOOP THROUGH ENVOY VECTOR ELEMENTS (WITHIN ROW) */
				for i in ei_start..ei_end {
					if init || (self.strengths[i] <= cmn::SYNAPSE_STRENGTH_FLOOR) {

						self.regrow_syn(i, &src_row_idx_range, &src_col_x_offs_range,
							&strength_init_range, &src_row_ids);

						//self.src_row_ids[i] = src_row_ids[src_row_idx_range.ind_sample(&mut self.rng)];
						//self.src_col_x_offs[i] = src_col_x_offs_range.ind_sample(&mut self.rng);
						//self.strengths[i] = (self.src_col_x_offs[i] >> 6) * strength_init_range.ind_sample(&mut self.rng);
					}
				}
			}
		}

		self.strengths.write();
		self.src_col_x_offs.write();
		self.src_row_ids.write();		
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue();
	}

	pub fn regrow(&mut self, region: &ProtoRegion) {
		/*let rnd = self.rng.gen::<u32>();
		//print!("\nRegrowing with rnd: {}", rnd);
		self.kern_regrow.set_named_arg("rnd", rnd);
		self.kern_regrow.enqueue();*/

		self.grow(region, false);
	}

	pub fn confab(&mut self) {
		self.states.read();
		self.strengths.read();
		self.src_row_ids.read();
		self.src_col_x_offs.read();
	} 

	pub fn width(&self) -> u32 {
		self.width
	}


	// NEEDS SERIOUS OPTIMIZATION
	fn regrow_syn(&mut self, 
				syn_idx: usize, 
				src_row_idx_range: &Range<usize>, 
				src_col_x_offs_range: &Range<i8>,
				strength_init_range: &Range<i8>,
				src_row_ids: &Vec<u8>
	) {
		//let src_row_idx_range: Range<usize> = Range::new(0, src_row_ids.len());
		//let src_col_x_offs_range: Range<i8> = Range::new(-126, 127);
		//let strength_init_range: Range<i8> = Range::new(-3, 4);

		//let src_row_id
		//let src_col_x_off
		//let strength

		for i in 0..200 {
			self.src_row_ids[syn_idx] = src_row_ids[src_row_idx_range.ind_sample(&mut self.rng)];
			self.src_col_x_offs[syn_idx] = src_col_x_offs_range.ind_sample(&mut self.rng);
			self.strengths[syn_idx] = (self.src_col_x_offs[syn_idx] >> 6) * strength_init_range.ind_sample(&mut self.rng);

			if self.unique_src_addr(syn_idx) {
				break;
			} else {
				//print!("[nu]");
			}
		}
	}

	fn unique_src_addr(&self, syn_idx: usize) -> bool {
		let syn_idx_den_init: usize = (syn_idx >> self.per_den_l2) << self.per_den_l2;
		let syn_idx_den_n: usize = syn_idx_den_init + (1 << self.per_den_l2);

		for i in syn_idx_den_init..syn_idx_den_n {
			if (self.src_row_ids[syn_idx] == self.src_row_ids[i]) 
				&& (self.src_col_x_offs[syn_idx] == self.src_col_x_offs[i])
				&& (i != syn_idx)
			{
				return false;
			}
		}

		true
	}

	/* REFRESH_ROW_POOL(): Pretty much being refactored into the new regrow
		- read
		- update cols and rows where str < whatever
		- write
		- piss off
	*/
	/*fn regrow_local(&mut self) {
		let src_row_ids_len: usize = src_row_ids.len();
		let src_row_idx_range: Range<usize> = Range::new(0, src_row_ids_len);

		for i in 0..cmn::SYNAPSE_ROW_POOL_SIZE {
			self.row_pool[i] = src_row_ids[src_row_idx_range.ind_sample(&mut rng)];
		}
	}*/
}
