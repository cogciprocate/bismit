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
			- Top priority is checking for uniqueness and correct distribution frequency among src_slcs and cols

		- [low priority] Optimization:
			- Obviously grow() and it's ilk need a lot of work

*/

const DEBUG_NEW: bool = true;
const DEBUG_GROW: bool = true;
const DEBUG_REGROW_DETAIL: bool = false;


pub struct Synapses {
	layer_name: &'static str,
	dims: CorticalDimensions,
	syns_per_den_l2: u8,
	protocell: Protocell,
	protoregion: Protoregion,
	dst_src_slc_id_grps: Vec<Vec<u8>>,
	den_kind: DendriteKind,
	cell_kind: ProtocellKind,
	since_decay: usize,
	kernels: Vec<Box<ocl::Kernel>>,
	//kern_cycle: ocl::Kernel,
	//kern_regrow: ocl::Kernel,
	rng: rand::XorShiftRng,
	pub states: Envoy<ocl::cl_uchar>,
	pub strengths: Envoy<ocl::cl_char>,
	pub src_slc_ids: Envoy<ocl::cl_uchar>,
	pub src_col_xy_offs: Envoy<ocl::cl_char>,
	//pub src_col_y_offs: Envoy<ocl::cl_char>,
	pub flag_sets: Envoy<ocl::cl_uchar>,
	//pub slc_pool: Envoy<ocl::cl_uchar>,  // BRING THIS BACK (OPTIMIZATION)
}

impl Synapses {
	pub fn new(layer_name: &'static str, dims: CorticalDimensions, protocell: Protocell, den_kind: DendriteKind, cell_kind: ProtocellKind, 
					protoregion: &Protoregion, axons: &Axons, aux: &Aux, ocl: &Ocl) -> Synapses {

		let syns_per_grp_l2: u8 = protocell.dens_per_grp_l2 + protocell.syns_per_den_l2;
		assert!(dims.per_grp_l2() as u8 == syns_per_grp_l2);

		//let syns_per_slc = dims.columns() << dims.per_grp_l2_left().expect("synapses.rs");
		let wg_size = cmn::SYNAPSES_WORKGROUP_SIZE;
		let syns_per_den_l2: u8 = protocell.syns_per_den_l2;

		//let slc_pool = Envoy::new(cmn::SYNAPSE_ROW_POOL_SIZE, 0, ocl); // BRING THIS BACK

		let states = Envoy::<ocl::cl_uchar>::with_padding(32768, dims, 0, ocl);
		//let states = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);
		let strengths = Envoy::<ocl::cl_char>::new(dims, 0, ocl);
		let mut src_slc_ids = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);
		// SRC COL REACHES SHOULD BECOME CONSTANTS
		let mut src_col_xy_offs = Envoy::<ocl::cl_char>::shuffled(dims, -126, 126, ocl); 
		//let mut src_col_y_offs = Envoy::<ocl::cl_char>::shuffled(dims, -31, 31, ocl);
		let flag_sets = Envoy::<ocl::cl_uchar>::new(dims, 0, ocl);


		let dst_src_slc_id_grps = protoregion.dst_src_slc_id_grps(layer_name);
		assert!(dst_src_slc_id_grps.len() == dims.grps_per_cel() as usize);

		let mut kernels = Vec::with_capacity(dst_src_slc_id_grps.len());

		if DEBUG_NEW { print!("\n            SYNAPSES::NEW(): kind: {:?}, len: {}, dims: {:?}", den_kind, states.len(), dims); }

			// *****NEW WorkSize::ThreeDim(dims.depth() as usize, dims.width() as usize, dims.height() as usize))
			// *****NEW .lws(WorkSize::ThreeDim(1 as usize, wg_size as usize))

		for syn_grp_i in 0..dst_src_slc_id_grps.len() {
			kernels.push(Box::new(
				ocl.new_kernel("syns_cycle", 
					WorkSize::ThreeDim(dims.depth() as usize, dims.height() as usize, dims.width() as usize))
					.lws(WorkSize::ThreeDim(1, 16, 16 as usize)) // TEMP UNTIL WE FIGURE OUT A WAY TO CALC THIS
					//WorkSize::ThreeDim(dims.columns() as usize, 1 as usize, dims.depth() as usize))
					//WorkSize::TwoDim(dims.columns() as usize, dims.depth() as usize))
					//.lws(WorkSize::TwoDim(1 as usize, wg_size as usize))
					.arg_env(&axons.states)
					.arg_env(&src_col_xy_offs)
					.arg_env(&src_slc_ids)
					//.arg_env(&strengths)
					.arg_scl(syn_grp_i as u32)
					.arg_scl(syns_per_grp_l2)
					.arg_env(&aux.ints_0)
					//.arg_env(&aux.ints_1)
					.arg_env(&states)
			))
		}

		//println!("\n### Defining kern_regrow with len: {} ###", dims.depth() as usize * dims as usize);

		/*let mut kern_regrow = ocl.new_kernel("syns_regrow", 
			WorkSize::TwoDim(dims.depth() as usize, dims.per_slc() as usize))
			//.lws(WorkSize::TwoDim(1 as usize, wg_size as usize))
			.arg_env(&strengths)
			.arg_scl(syns_per_den_l2 as u32)
			.arg_scl_named(0u32, "rnd")
			//.arg_env(&aux.ints_0)
			//.arg_env(&aux.ints_1)
			.arg_env(&src_col_xy_offs)
			.arg_env(&src_slc_ids)
		;*/

		let mut syns = Synapses {
			layer_name: layer_name,
			dims: dims,
			syns_per_den_l2: syns_per_den_l2,
			protocell: protocell,
			protoregion: protoregion.clone(),
			dst_src_slc_id_grps: dst_src_slc_id_grps,
			den_kind: den_kind,
			cell_kind: cell_kind,
			since_decay: 0,
			//kern_cycle: kern_cycle,
			kernels: kernels,
			//kern_regrow: kern_regrow,
			rng: rand::weak_rng(),
			states: states,
			strengths: strengths,
			src_slc_ids: src_slc_ids,
			src_col_xy_offs: src_col_xy_offs,
			//src_col_y_offs: src_col_y_offs,
			flag_sets: flag_sets,
			//slc_pool: slc_pool,  // BRING THIS BACK
		};

		syns.grow(true);
		//syns.refresh_slc_pool();

		syns
	}


	fn grow(&mut self, init: bool) {
		if DEBUG_GROW && DEBUG_REGROW_DETAIL && !init {
			print!("\nRG:{:?}: [PRE:(SLICE)(OFFSET)(STRENGTH)=>($:UNIQUE, ^:DUPL)=>POST:(..)(..)(..)]\n", self.den_kind);
		}

		assert!(
			(self.src_col_xy_offs.dims().per_slc() == self.src_slc_ids.dims().per_slc()) 
			&& ((self.src_slc_ids.dims().per_slc() == (self.dims().per_slc()))), 
			"[cortical_area::Synapses::init(): dims.columns() mismatch]"
		);

		self.strengths.read();
		self.src_slc_ids.read();
		self.src_col_xy_offs.read();


		//let syns_per_slc = self.dims.per_slc() as usize;
		//let grps_per_cel = self.dims.grps_per_cel() as usize;
		let syns_per_layer_grp = self.dims.per_slc_per_grp() as usize * self.dims.depth() as usize;
		//let slc_ids = self.protoregion.slc_ids(vec!(self.layer_name)).clone();
		let dst_src_slc_id_grps = self.dst_src_slc_id_grps.clone();
		let mut src_grp_i = 0usize;

		for src_slc_ids in dst_src_slc_id_grps {
			assert!(src_slc_ids.len() > 0, "Synapses must have at least one source slice.");
			assert!(src_slc_ids.len() <= (self.dims.per_cel()) as usize, "cortical_area::Synapses::init(): Number of source slcs must not exceed number of synapses per cell.");

			if init && DEBUG_GROW { }

			let src_slc_id_range: Range<usize> = Range::new(0, src_slc_ids.len());
			let src_col_xy_offs_range: Range<i8> = Range::new(-126, 127);
			let strength_init_range: Range<i8> = Range::new(-3, 4);

			let idz = syns_per_layer_grp * src_grp_i as usize;
			let idn = idz + syns_per_layer_grp as usize;

			if init && DEBUG_GROW {
				print!("\n                syns.init(): \"{}\" ({:?}): src_slc_ids: {:?}, syns_per_layer_grp:{}, idz:{}, idn:{}", self.layer_name, self.den_kind, src_slc_ids, syns_per_layer_grp, idz, idn);	
			}

			for i in idz..idn {
				if init || (self.strengths[i] <= cmn::SYNAPSE_STRENGTH_FLOOR) {
					self.regrow_syn(i, &src_slc_id_range, &src_col_xy_offs_range,
						&strength_init_range, &src_slc_ids, init);
				}
			}

			src_grp_i += 1;
		}

		self.strengths.write();
		self.src_slc_ids.write();
		self.src_col_xy_offs.write();	
	}

	fn regrow_syn(&mut self, 
				syn_idx: usize, 
				src_slc_idx_range: &Range<usize>, 
				src_col_xy_offs_range: &Range<i8>,
				strength_init_range: &Range<i8>,
				src_slc_ids: &Vec<u8>,
				init: bool,
	) {
		//let src_slc_idx_range: Range<usize> = Range::new(0, src_slc_ids.len());
		//let src_col_xy_offs_range: Range<i8> = Range::new(-127, 127);
		//let strength_init_range: Range<i8> = Range::new(-3, 4);

		//let src_slc_id
		//let src_col_x_off
		//let strength
		let mut print_str: String = String::with_capacity(10);

		let mut tmp_str = format!("[({})({})({})=>", self.src_slc_ids[syn_idx], self.src_col_xy_offs[syn_idx],  self.strengths[syn_idx]);
		print_str.push_str(&tmp_str);

		for i in 0..200 {
			self.src_slc_ids[syn_idx] = src_slc_ids[src_slc_idx_range.ind_sample(&mut self.rng)];
			self.src_col_xy_offs[syn_idx] = src_col_xy_offs_range.ind_sample(&mut self.rng);
			self.strengths[syn_idx] = (self.src_col_xy_offs[syn_idx] >> 6) * strength_init_range.ind_sample(&mut self.rng);

			if self.unique_src_addr(syn_idx) {
				print_str.push_str("$");
				break;
			} else {
				print_str.push_str("^");
			}
		}

		tmp_str = format!("=>({})({})({})] ", self.src_slc_ids[syn_idx], self.src_col_xy_offs[syn_idx],  self.strengths[syn_idx]);
		print_str.push_str(&tmp_str);

		if DEBUG_GROW && DEBUG_REGROW_DETAIL && !init {
			print!("{}", print_str);
		}
	}

	/* SRC_SLICE_IDS(): TODO: DEPRICATE */
	pub fn src_slc_ids(&self, layer_name: &'static str, layer: &Protolayer) -> Vec<u8> {
		
		//println!("\n##### SYNAPSES::SRC_SLICE_IDS({}): {:?}", layer_name, self.dst_src_slc_id_grps);

		match layer.kind {
			ProtolayerKind::Cellular(ref cell) => {
				if cell.cell_kind == self.cell_kind {
					self.protoregion.src_slc_ids(layer_name, self.den_kind)
				} else {
					panic!("Synapse::src_slc_ids(): cell_kind mismatch! ")
				}
			},

			_ => panic!("Synapse::src_slc_ids(): ProtolayerKind not Cellular! "),
		}
	}

	// NEEDS SERIOUS OPTIMIZATION
	// Cache and sort by axn_idx (pre_compute, keep seperate list) for each dendrite
	fn unique_src_addr(&self, syn_idx: usize) -> bool {
		let syns_per_den_l2 = self.protocell.syns_per_den_l2;
		let syn_idx_den_init: usize = (syn_idx >> syns_per_den_l2) << syns_per_den_l2;
		let syn_idx_den_n: usize = syn_idx_den_init + (1 << syns_per_den_l2);

		for i in syn_idx_den_init..syn_idx_den_n {
			if (self.src_slc_ids[syn_idx] == self.src_slc_ids[i]) 
				&& (self.src_col_xy_offs[syn_idx] == self.src_col_xy_offs[i])
				&& (i != syn_idx)
			{
				return false;
			}
		}

		true
	}
	

	pub fn cycle(&self) {
		for kern in self.kernels.iter() {
			kern.enqueue();
		}
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
		self.src_slc_ids.read();
		self.src_col_xy_offs.read();
	} 

	pub fn den_kind(&self) -> DendriteKind {
		self.den_kind.clone()
	}

	pub fn dims(&self) -> &CorticalDimensions {
		&self.dims
	}

}




/*	SYNAPSES::GROW(): This whole thing needs a massive amount of reworking
			- We no longer have one contiguous space
			- Tons of info in self.protocell we could use instead of protoregion calls
				- Look towards depricating calls to protoregion
*/
/*fn grow_old(&mut self, init: bool) {
	if DEBUG_GROW && DEBUG_REGROW_DETAIL && !init {
		print!("\nRG:{:?}: [PRE:(SLICE)(OFFSET)(STRENGTH)=>($:UNIQUE, ^:DUPL)=>POST:(..)(..)(..)]\n", self.den_kind);
	}

	assert!(
		(self.src_col_xy_offs.dims().per_slc() == self.src_slc_ids.dims().per_slc()) 
		&& ((self.src_slc_ids.dims().per_slc() == (self.dims().per_slc()))), 
		"[cortical_area::Synapses::init(): dims.columns() mismatch]"
	);

	self.strengths.read();
	self.src_slc_ids.read();
	self.src_col_xy_offs.read();

	let syns_per_slc = self.dims.per_slc();
	let layer_name = self.layer_name;
	// CLEAN THIS UP A BIT SOMEHOW...
	let layer = self.protoregion.get_layer(layer_name).expect("Synapses::grow()::emsg1").clone();

	let src_slc_ids = self.src_slc_ids(layer_name, &layer);

	let slc_ids = self.protoregion.slc_ids(vec!(layer_name)).clone();
	let src_slc_ids_len: usize = src_slc_ids.len();

	assert!(src_slc_ids_len > 0, "Synapses must have at least one source slc");

	//let kind_base_slc_pos = layer.kind_base_slc_pos; // BASED ON OLD SYSTEM
	let src_slc_idx_range: Range<usize> = Range::new(0, src_slc_ids_len);
	let src_col_xy_offs_range: Range<i8> = Range::new(-126, 127);
	let strength_init_range: Range<i8> = Range::new(-3, 4);
	
	assert!(src_slc_ids_len <= (self.dims.per_cel()) as usize, "cortical_area::Synapses::init(): Number of source slcs must not exceed number of synapses per cell.");

	if init && DEBUG_GROW {
		print!("\n#####    syns.init(): \"{}\" ({:?}): slc_ids: {:?}, src_slc_ids: {:?}", layer_name, self.den_kind, slc_ids, src_slc_ids);
	}

	// LOOP THROUGH ROWS (WITHIN LAYER) 
	for slc_pos in 0..layer.depth {

		let ei_start = syns_per_slc as usize * slc_pos as usize;

		let ei_end = ei_start + syns_per_slc as usize;

		if init && DEBUG_GROW {
			print!("\n   Row {}: syns_per_slc:{}, ei_start:{}, ei_end:{}, src_slc_ids:{:?}", slc_pos, syns_per_slc, ei_start, ei_end, src_slc_ids);
		}

		// LOOP THROUGH ENVOY VECTOR ELEMENTS (WITHIN ROW) 
		for i in ei_start..ei_end {
			if init || (self.strengths[i] <= cmn::SYNAPSE_STRENGTH_FLOOR) {

				self.regrow_syn(i, &src_slc_idx_range, &src_col_xy_offs_range,
					&strength_init_range, &src_slc_ids, init);

				//self.src_slc_ids[i] = src_slc_ids[src_slc_idx_range.ind_sample(&mut self.rng)];
				//self.src_col_xy_offs[i] = src_col_xy_offs_range.ind_sample(&mut self.rng);
				//self.strengths[i] = (self.src_col_xy_offs[i] >> 6) * strength_init_range.ind_sample(&mut self.rng);
			}
		}
	}

	self.strengths.write();
	self.src_slc_ids.write();
	self.src_col_xy_offs.write();
}*/












		/* LOOP THROUGH ALL LAYERS */
		/*for (&layer_name, layer) in self.protoregion.layers().clone().iter() {
			let src_slc_ids = match self.src_slc_ids(layer_name, layer) {
				Some(ss_ids) => ss_ids,
				None 		=> continue,
			};

			let slc_ids = self.protoregion.slc_ids(vec!(layer_name)).clone();
			let src_slc_ids_len: usize = src_slc_ids.len();

			assert!(src_slc_ids_len > 0, "Synapses must have at least one source slc");

			let kind_base_slc_pos = layer.kind_base_slc_pos;
			let src_slc_idx_range: Range<usize> = Range::new(0, src_slc_ids_len);
			let src_col_xy_offs_range: Range<i8> = Range::new(-126, 127);
			let strength_init_range: Range<i8> = Range::new(-3, 4);
			
			assert!(src_slc_ids_len <= (self.dims.per_cel().expect("synapses.rs")) as usize, "cortical_area::Synapses::init(): Number of source slcs must not exceed number of synapses per cell.");

			if init && DEBUG_GROW {
				print!("\n#####    syns.init(): \"{}\" ({:?}): slc_ids: {:?}, src_slc_ids: {:?}", layer_name, self.den_kind, slc_ids, src_slc_ids);
			}

			/* LOOP THROUGH ROWS (WITHIN LAYER) */
			for slc_pos in kind_base_slc_pos..(kind_base_slc_pos + layer.depth) {

				let ei_start = syns_per_slc as usize * slc_pos as usize;

				let ei_end = ei_start + syns_per_slc as usize;

				if init && DEBUG_GROW {
					print!("\n   Row {}: syns_per_slc:{}, ei_start:{}, ei_end:{}, src_slc_ids:{:?}", slc_pos, syns_per_slc, ei_start, ei_end, src_slc_ids);
				}

				/* LOOP THROUGH ENVOY VECTOR ELEMENTS (WITHIN ROW) */
				for i in ei_start..ei_end {
					if init || (self.strengths[i] <= cmn::SYNAPSE_STRENGTH_FLOOR) {

						self.regrow_syn(i, &src_slc_idx_range, &src_col_xy_offs_range,
							&strength_init_range, &src_slc_ids, init);

						//self.src_slc_ids[i] = src_slc_ids[src_slc_idx_range.ind_sample(&mut self.rng)];
						//self.src_col_xy_offs[i] = src_col_xy_offs_range.ind_sample(&mut self.rng);
						//self.strengths[i] = (self.src_col_xy_offs[i] >> 6) * strength_init_range.ind_sample(&mut self.rng);
					}
				}
			}
		}*/
