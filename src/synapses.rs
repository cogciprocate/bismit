use num;
use rand;
use std::mem;
use rand::distributions::{ Normal, IndependentSample, Range };
use rand::{ ThreadRng, Rng };
use num::{ Integer };
use std::default::{ Default };
use std::fmt::{ Display };

use cmn;
use ocl::{ self, Ocl, WorkSize, Envoy, CorticalDimensions };
use proto::areas::{ Protoareas };
use proto::regions::{ Protoregion, ProtoregionKind };
use proto::layer::{ Protolayer, ProtolayerKind };
use proto::cell::{ ProtocellKind, Protocell, DendriteKind };
use dendrites::{ Dendrites };
use axons::{ Axons };
use cortical_area:: { Aux };


/* Synapses: Smallest and most numerous unit in the cortex - the soldier behind it all
	- TODO:
		- [high priority] Testing: 
			- Top priority is checking for uniqueness and correct distribution frequency among src_slices and cols

		- [low priority] Optimization:
			- Obviously grow() and it's ilk need a lot of work

*/

const DEBUG_REGROWTH: bool = false;


pub struct Synapses {
	dims: CorticalDimensions, 
	protocell: Protocell,
	protoregion: Protoregion,
	den_kind: DendriteKind,
	cell_kind: ProtocellKind,
	since_decay: usize,
	kern_cycle: ocl::Kernel,
	//kern_regrow: ocl::Kernel,
	rng: rand::XorShiftRng,
	pub states: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_char>,
	pub src_slice_ids: Envoy<ocl::cl_uchar>,
	pub src_col_xy_offs: Envoy<ocl::cl_char>,
	//pub src_col_y_offs: Envoy<ocl::cl_char>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	//pub slice_pool: Envoy<ocl::cl_uchar>,  // BRING THIS BACK
}

impl Synapses {
	pub fn new(dims: CorticalDimensions, protocell: Protocell, den_kind: DendriteKind, cell_kind: ProtocellKind, 
					protoregion: &Protoregion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> Synapses {

		let syns_per_cel_l2: u8 = protocell.dens_per_cel_l2 + protocell.syns_per_den_l2;
		assert!(dims.per_cel_l2() as u8 == syns_per_cel_l2);

		//let syns_per_slice = dims.columns() << dims.per_cel_l2_left().expect("synapses.rs");
		let wg_size = cmn::SYNAPSES_WORKGROUP_SIZE;

		let syns_per_den_l2: u8 = protocell.syns_per_den_l2;

		//let slice_pool = Envoy::new(cmn::SYNAPSE_ROW_POOL_SIZE, 0, ocl); // BRING THIS BACK


		print!("\n            SYNAPSES::NEW(): new {:?} synapses with dims: {:?}", den_kind, dims);
		//println!("##### Synapses columns(): {}, per_slice(): {}", dims.columns(), dims.per_slice());

		let states = Envoy::<ocl::cl_uchar>::with_padding(32768, dims, 0, ocl);
		//let states = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);
		let strengths = Envoy::<ocl::cl_char>::new(dims, 0, ocl);
		let mut src_slice_ids = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);

		// SRC COL REACHES SHOULD BECOME CONSTANTS
		let mut src_col_xy_offs = Envoy::<ocl::cl_char>::shuffled(dims, -126, 126, ocl); 
		//let mut src_col_y_offs = Envoy::<ocl::cl_char>::shuffled(dims, -31, 31, ocl);

		let flag_sets = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);


		let mut kern_cycle = ocl.new_kernel("syns_cycle", 
			WorkSize::TwoDim(dims.depth() as usize, dims.columns() as usize))
			.lws(WorkSize::TwoDim(1 as usize, wg_size as usize))
			.arg_env(&axons.states)
			.arg_env(&src_col_xy_offs)
			.arg_env(&src_slice_ids)
			//.arg_env(&strengths)
			.arg_scl(syns_per_cel_l2)
			.arg_env(&aux.ints_0)
			//.arg_env(&aux.ints_1)
			.arg_env(&states)
		;

		//println!("\n### Defining kern_regrow with len: {} ###", dims.depth() as usize * dims as usize);

		/*let mut kern_regrow = ocl.new_kernel("syns_regrow", 
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slice() as usize))
			//.lws(WorkSize::TwoDim(1 as usize, wg_size as usize))
			.arg_env(&strengths)
			.arg_scl(syns_per_den_l2 as u32)
			.arg_scl_named(0u32, "rnd")
			//.arg_env(&aux.ints_0)
			//.arg_env(&aux.ints_1)
			.arg_env(&src_col_xy_offs)
			.arg_env(&src_slice_ids)
		;*/

		//println!("\n### Test S1 ###");

		let mut syns = Synapses {
			dims: dims,
			protocell: protocell,
			protoregion: protoregion.clone(),
			//per_cell_l2: per_cell_l2,
			//per_den_l2: per_den_l2,
			den_kind: den_kind,
			cell_kind: cell_kind,
			since_decay: 0,
			kern_cycle: kern_cycle,
			//kern_regrow: kern_regrow,
			rng: rand::weak_rng(),
			states: states,
			strengths: strengths,
			src_slice_ids: src_slice_ids,
			src_col_xy_offs: src_col_xy_offs,
			//src_col_y_offs: src_col_y_offs,
			flag_sets: flag_sets,
			//slice_pool: slice_pool,  // BRING THIS BACK
		};

		syns.grow(true);
		//syns.refresh_slice_pool();

		syns
	}

	fn grow(&mut self, init: bool) {
		if DEBUG_REGROWTH && !init {
			print!("\nRG:{:?}: [PRE:(SLICE)(OFFSET)(STRENGTH)=>($:UNIQUE, ^:DUPL)=>POST:(..)(..)(..)]\n", self.den_kind);
		}

		assert!(
			(self.src_col_xy_offs.dims().per_slice() == self.src_slice_ids.dims().per_slice()) 
			&& ((self.src_slice_ids.dims().per_slice() == (self.dims().per_slice()))), 
			"[cortical_area::Synapses::init(): dims.columns() mismatch]"
		);

		self.confab();

		let syns_per_slice = self.dims.per_slice();
		//self.dims.columns() << self.dims.per_cel_l2;
		//let mut rng = rand::weak_rng();

		/* LOOP THROUGH ALL LAYERS */
		for (&layer_name, layer) in self.protoregion.layers().clone().iter() {
			/*let src_slice_ids_opt: Vec<u8> = match layer.kind {
				ProtolayerKind::Cellular(ref cell) => {
					if cell.cell_kind == self.cell_kind {
						self.protoregion.src_slice_ids(layer_name, self.den_kind)
					} else {
						continue
					}
				},
				_ => continue,
			};*/

			let src_slice_ids = match self.src_slice_ids(layer_name, layer) {
				Some(ssids) => ssids,
				None 		=> continue,
			};

			let slice_ids = self.protoregion.slice_ids(vec!(layer_name)).clone();
			let src_slice_ids_len: usize = src_slice_ids.len();

			assert!(src_slice_ids_len > 0, "Synapses must have at least one source slice");

			let kind_base_slice_pos = layer.kind_base_slice_pos;
			let src_slice_idx_range: Range<usize> = Range::new(0, src_slice_ids_len);
			let src_col_xy_offs_range: Range<i8> = Range::new(-126, 127);
			let strength_init_range: Range<i8> = Range::new(-3, 4);

			//println!("\n##### SYNAPSE DIMS: {:?}", self.dims);
			
			assert!(src_slice_ids_len <= (self.dims.per_cel().expect("synapses.rs")) as usize, "cortical_area::Synapses::init(): Number of source slices must not exceed number of synapses per cell.");

			if init {
				print!("\n    syns.init(): \"{}\" ({:?}): slice_ids: {:?}, src_slice_ids: {:?}", layer_name, self.den_kind, slice_ids, src_slice_ids);
			}

			/* LOOP THROUGH ROWS (WITHIN LAYER) */
			for slice_pos in kind_base_slice_pos..(kind_base_slice_pos + layer.depth) {
				//print!("\nDEBUG: slice_pos: {}", slice_pos);
				//print!("\nDEBUG: syns_per_slice: {}", syns_per_slice);

				let ei_start = syns_per_slice as usize * slice_pos as usize;
				//print!("\nDEBUG: ei_start: {}", ei_start);

				let ei_end = ei_start + syns_per_slice as usize;
				//print!("\nDEBUG: ei_end: {}", ei_end);
				//print!("\nDEBUG: src_slice_ids: {:?}", src_slice_ids);

				//print!("\n   Row {}: ei_start: {}, ei_end: {}, src_slice_ids: {:?}", slice_pos, ei_start, ei_end, src_slice_ids);

				/* LOOP THROUGH ENVOY VECTOR ELEMENTS (WITHIN ROW) */
				for i in ei_start..ei_end {
					if init || (self.strengths[i] <= cmn::SYNAPSE_STRENGTH_FLOOR) {

						self.regrow_syn(i, &src_slice_idx_range, &src_col_xy_offs_range,
							&strength_init_range, &src_slice_ids, init);

						//self.src_slice_ids[i] = src_slice_ids[src_slice_idx_range.ind_sample(&mut self.rng)];
						//self.src_col_xy_offs[i] = src_col_xy_offs_range.ind_sample(&mut self.rng);
						//self.strengths[i] = (self.src_col_xy_offs[i] >> 6) * strength_init_range.ind_sample(&mut self.rng);
					}
				}
			}
		}

		self.strengths.write();
		self.src_col_xy_offs.write();
		self.src_slice_ids.write();		
	}

	pub fn cycle(&self) {
		self.kern_cycle.enqueue();
	}

	pub fn regrow(&mut self) {
		//let rnd = self.rng.gen::<u32>();
		//self.kern_regrow.set_arg_scl_named("rnd", rnd);
		//self.kern_regrow.enqueue();

		self.grow(false);
	}

	pub fn confab(&mut self) {
		self.states.read();
		self.strengths.read();
		self.src_slice_ids.read();
		self.src_col_xy_offs.read();
	} 

	/*pub fn width(&self) -> u32 {
		self.width
	}*/

	pub fn src_slice_ids(&self, layer_name: &'static str, layer: &Protolayer) -> Option<Vec<u8>> {
		match layer.kind {
			ProtolayerKind::Cellular(ref cell) => {
				if cell.cell_kind == self.cell_kind {
					Some(self.protoregion.src_slice_ids(layer_name, self.den_kind))
				} else {
					None
				}
			},
			_ => None,
		}
	}

	pub fn den_kind(&self) -> DendriteKind {
		self.den_kind.clone()
	}

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

	// NEEDS SERIOUS OPTIMIZATION
	// Cache and sort by axn_idx (pre_compute, keep seperate list) for each dendrite
	fn regrow_syn(&mut self, 
				syn_idx: usize, 
				src_slice_idx_range: &Range<usize>, 
				src_col_xy_offs_range: &Range<i8>,
				strength_init_range: &Range<i8>,
				src_slice_ids: &Vec<u8>,
				init: bool,
	) {
		//let src_slice_idx_range: Range<usize> = Range::new(0, src_slice_ids.len());
		//let src_col_xy_offs_range: Range<i8> = Range::new(-127, 127);
		//let strength_init_range: Range<i8> = Range::new(-3, 4);

		//let src_slice_id
		//let src_col_x_off
		//let strength
		let mut print_str: String = String::with_capacity(10);

		let mut tmp_str = format!("[({})({})({})=>", self.src_slice_ids[syn_idx], self.src_col_xy_offs[syn_idx],  self.strengths[syn_idx]);
		print_str.push_str(&tmp_str);

		for i in 0..200 {
			self.src_slice_ids[syn_idx] = src_slice_ids[src_slice_idx_range.ind_sample(&mut self.rng)];
			self.src_col_xy_offs[syn_idx] = src_col_xy_offs_range.ind_sample(&mut self.rng);
			self.strengths[syn_idx] = (self.src_col_xy_offs[syn_idx] >> 6) * strength_init_range.ind_sample(&mut self.rng);

			if self.unique_src_addr(syn_idx) {
				print_str.push_str("$");
				break;
			} else {
				print_str.push_str("^");
			}
		}

		tmp_str = format!("=>({})({})({})] ", self.src_slice_ids[syn_idx], self.src_col_xy_offs[syn_idx],  self.strengths[syn_idx]);
		print_str.push_str(&tmp_str);

		if DEBUG_REGROWTH && !init {
			print!("{}", print_str);
		}
	}

	fn unique_src_addr(&self, syn_idx: usize) -> bool {
		let syns_per_den_l2 = self.protocell.syns_per_den_l2;
		let syn_idx_den_init: usize = (syn_idx >> syns_per_den_l2) << syns_per_den_l2;
		let syn_idx_den_n: usize = syn_idx_den_init + (1 << syns_per_den_l2);

		for i in syn_idx_den_init..syn_idx_den_n {
			if (self.src_slice_ids[syn_idx] == self.src_slice_ids[i]) 
				&& (self.src_col_xy_offs[syn_idx] == self.src_col_xy_offs[i])
				&& (i != syn_idx)
			{
				return false;
			}
		}

		true
	}





	/* REFRESH_ROW_POOL(): Pretty much being refactored into the new regrow
		- read
		- update cols and slices where str < whatever
		- write
		- piss off
	*/
	/*fn regrow_local(&mut self) {
		let src_slice_ids_len: usize = src_slice_ids.len();
		let src_slice_idx_range: Range<usize> = Range::new(0, src_slice_ids_len);

		for i in 0..cmn::SYNAPSE_ROW_POOL_SIZE {
			self.slice_pool[i] = src_slice_ids[src_slice_idx_range.ind_sample(&mut rng)];
		}
	}*/
}
