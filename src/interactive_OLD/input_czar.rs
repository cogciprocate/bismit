use std::iter;
use std::ops::{ Range };
use std::cmp;
// use std::collections::{ HashMap };
use rand::{ self, /*ThreadRng,*/ Rng };

use cmn::{ self, CorticalDims, Sdr };
use ocl::{ self };
use cortex::{ Cortex };
use encode:: { IdxReader };
use input_source::{ InputGanglion };
use super::motor_state;
use microcosm::world::{ World };
use microcosm::entity::{ EntityBody, EntityKind, /*EntityBrain, Mobile*/ };
// use microcosm::worm::{ WormBrain };
use microcosm::common::{ Location, /*Peek, Scent, WORM_SPEED, TAU*/ };


pub const WORLD_TURN_FACTOR: f32 				= 9f32;	// (originally 3)


//pub const PARAM_RANDOM_COUNTER: bool = true;


pub struct InputCzar {
	dims: CorticalDims,
	counter: usize,
	counter_range: Range<usize>,
	random_counter: bool,
	toggle_dirs: bool,
	introduce_noise: bool,
	ttl_count: usize,
	reset_count: usize,
	rng: rand::XorShiftRng,
	input_sources: Vec<InputSource>,
	//optical_vec_kind: InputKind,
	pub vec_optical: Vec<u8>,
	pub vec_motor: Vec<u8>,
	pub vec_test_noise: Vec<u8>,
	//world: World,
	//worm: EntityBody,
	pub motor_state: motor_state::MotorState,
}

impl InputCzar {
	pub fn new(dims: CorticalDims, input_sources: Vec<InputSource>, counter_range: Range<usize>, random_counter: bool, toggle_dirs: bool, introduce_noise: bool) -> InputCzar {

		let area = dims.columns();

		let motor_state = motor_state::MotorState::new();

		let mut vec_motor: Vec<u8> = iter::repeat(0).take(cmn::SYNAPSE_SPAN_RHOMBAL_AREA as usize).collect();
		
		if toggle_dirs {
			vec_motor.clone_from_slice(&motor_state.cur_sdr(false));
		}

		let vec_test_noise = junk0_vec_init(cmn::SYNAPSE_SPAN_RHOMBAL_AREA, 0);

		InputCzar {
			dims: dims,
			counter: counter_range.end,
			counter_range: counter_range,
			random_counter: random_counter,
			toggle_dirs: toggle_dirs,
			introduce_noise: introduce_noise,
			ttl_count: 0,
			reset_count: 0,
			//next_turn_counter: 0,
			//next_turn_max: 0,
			rng: rand::weak_rng(),
			input_sources: input_sources,
			//optical_vec_kind: optical_vec_kind,
			vec_optical: iter::repeat(0).take(area as usize).collect(),
			vec_motor: vec_motor,
			vec_test_noise: vec_test_noise,
			//world: world,
			//worm: worm,
			motor_state: motor_state,
		}
	}

	fn init_world(&self) -> (World, EntityBody) {
		let mut world = World::new(self.dims.columns());

		let worm =  EntityBody::new("worm", EntityKind::Creature, Location::origin());
		world.entities().add(worm);
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, -220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, 220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, -220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, 220f32)));

		(world, worm)
	}

	pub fn next(&mut self, cortex: &mut Cortex) -> usize {
		let remain_ticks = self.tick();
		let mut input_status: usize = 0;

		if self.introduce_noise {
			/*if (self.ttl_count & 0x01) == 0x01 {
				//if (self.reset_count & 0x01) == 0x01 {
				self.vec_test_noise = test_vec_init(cmn::SYNAPSE_SPAN_RHOMBAL_AREA, 0);
			} else {
				self.vec_test_noise = test_vec_init(cmn::SYNAPSE_SPAN_RHOMBAL_AREA, 1);
			}*/
		}

		/* ##### MOTOR ##### */
		if self.toggle_dirs {
			if remain_ticks == 0 {
				//print!("[> ctr = 0 <]");
				self.motor_state.switch();
			} else if remain_ticks == 1 {
				self.vec_motor.clone_from_slice(&self.motor_state.cur_sdr(true));
				//print!("[> ctr = 1 <]");
			}

		}


		/* ##### OPTICAL ##### */
		match self.input_sources[0].kind {
			InputKind::World => {
				// let turn_amt = WORLD_TURN_FACTOR / (self.dims.columns() as f32);
				// self.world.entities().get_mut(self.worm.uid).turn(turn_amt, self.motor_state.cur_turn());

				// if !self.toggle_dirs && remain_ticks == 0 {
				// 	self.world.entities().get_mut(self.worm.uid).head_north();
				// }
				// self.world.peek_from(self.worm.uid).unfold_into(&mut self.vec_optical, 0);
			},

			InputKind::Stripes { stripe_size, zeros_first } => {
				sdr_stripes(stripe_size, zeros_first, &mut self.vec_optical[..]);
			},

			InputKind::Hexballs { edge_size, invert, fill } => {
				sdr_hexballs(edge_size, invert, fill, self.dims, self.counter, &mut self.vec_optical[..]);
			},

			InputKind::Exp1 => {
				sdr_exp1(&mut self.vec_optical[..]);
			},

			InputKind::IdxReader(ref mut ir) => {
				input_status = ir.cycle(&mut self.vec_optical[..]);
				//input_status = 999;
			}

			//_ => (),
		}


		/* ##### TEST_NOISE ##### */
		// nothing here yet

		self.sense(cortex);
		return input_status;
	}

	pub fn sense(&self, cortex: &mut Cortex) {
		//cortex.write_input(self.input_sources[0].target_area_name, &self.vec_optical);
		//cortex.write_input("v2", &self.vec_optical); // *****
		//cortex.write_input("a1", &self.vec_optical); // *****
		//cortex.write(self.input_sources[0].target_area_name, "motor_in", &self.vec_motor);
		//cortex.write("v1", "test_noise", &self.vec_test_noise);
		//cortex.cycle_old("v1");
		cortex.cycle();
	}

	fn tick(&mut self) -> usize {
		self.ttl_count += 1;

		if self.counter > 0 {
			self.counter -= 1;
		}

		if self.counter == 0 {
			self.reset_counter();
			0
		} else {
			self.counter
		}
	}

	pub fn set_counter(&mut self, count: usize) {
		self.random_counter = false;
		self.counter_range.end = count;
		self.counter = count;
	}

	pub fn reset_counter(&mut self) {
		self.reset_count += 1;

		if self.random_counter {
			self.counter = self.rng.gen_range::<usize>(self.counter_range.start, self.counter_range.end);
		} else {
			self.counter = self.counter_range.end;
		}
	}

	pub fn counter(&self) -> usize {
		self.counter
	}
}

pub enum InputKind {
	World,
	Stripes { stripe_size: usize, zeros_first: bool },
	Hexballs { edge_size: usize, invert: bool, fill: bool },
	Exp1,
	IdxReader(Box<IdxReader>),
}

pub struct InputSource {
	kind: InputKind,
	target_area_name: &'static str,	
}

impl InputSource {
	pub fn new(kind: InputKind, target_area_name: &'static str) -> InputSource {
		InputSource {
			target_area_name: target_area_name,
			kind: kind,
		}
	}
}


/*pub fn vec_band_512_fill(vec: &mut Vec<u8>) {
	for i in 0..vec.len() {
		if (i & 512) == 0 {
			vec[i] = 0;
		} else {
			vec[i] = 1;
		}
	}
}*/

pub fn sdr_exp1(vec: &mut Sdr) {

	// for i in 0..vec.len() {
	// 	if (i >= 384 - 64) && (i < 384 + 64) {
	// 		vec[i] = (i - (384 - 64)) as u8;
	// 	} else {
	// 		vec[i] = 0;
	// 	}
	// }

	for i in 0..vec.len() {
		if i == 384 {
			vec[i] = 99;
		} else {
			vec[i] = 0;
		}
	}
}


pub fn sdr_hexballs(edge_size: usize, invert: bool, fill_hex: bool, dims: CorticalDims, counter: usize, vec: &mut Sdr) {
	let v_size = dims.v_size() as i64;
	let u_size = dims.u_size() as i64;
	let edge_size = edge_size as i64;
	let mut rng = rand::weak_rng();

	let (on, off) = if invert { 
		(0, 127)
	} else {
		(127,0)
	};

	for c in vec.iter_mut() {
		*c = off;
	}

	let hexagon_count = 9;
	let gap_factor = 3;
	let gap_extra = 0;
	let movement_factor = edge_size - 1;
	let movement_start_offset = (movement_factor * gap_factor) + 80;

	let first_hex_ofs = (counter as i64 * movement_factor) - movement_start_offset;
	let hex_spacing = (edge_size * gap_factor) + gap_extra;
	let (hc_init_v, hc_init_u) = (edge_size + first_hex_ofs, edge_size + first_hex_ofs);


	for i in 0..hexagon_count {

		//print!("[ball:{}]", i);

		let (v_id, u_id) = (hc_init_u + (i * hex_spacing), hc_init_v + (i * hex_spacing) + (i * 5) - 15);

		let v_ofs_z = 0 - edge_size;
		let v_ofs_n = edge_size + 1;

		for v_ofs in v_ofs_z..v_ofs_n {
			let v_ofs_inv = 0 - v_ofs;
			let u_ofs_z = cmp::max(0 - edge_size, v_ofs_inv - edge_size);
			let u_ofs_n = cmp::min(edge_size, v_ofs_inv + edge_size) + 1;
			//print!("[v_ofs:{}]", v_ofs);

			for u_ofs in u_ofs_z..u_ofs_n {
				let cell_write: bool = if fill_hex {
					true
				} else if v_ofs.abs() == edge_size || u_ofs.abs() == edge_size || (v_ofs + u_ofs).abs() == edge_size {
					true
				} else {
					false
				};

				let (col_id, valid) = gimme_a_valid_col_id(dims, v_id + v_ofs, u_id + u_ofs);

				if cell_write && valid {
					vec[col_id] = on & rng.gen::<u8>();
				}
				//print!("{} ", gimme_a_valid_col_id(dims, v_id + v_ofs, u_id + u_ofs));
			}

		}
	}
}

pub fn gimme_a_valid_col_id(dims: CorticalDims, v_id: i64, u_id: i64) -> (usize, bool) {
	let v_ok = (v_id < dims.v_size() as i64) && (v_id >= 0);
	let u_ok = (u_id < dims.u_size() as i64) && (u_id >= 0);

	if v_ok && u_ok {
		(((v_id * dims.u_size() as i64) + u_id) as usize, true)
	} else {
		(0, false)
	}
}


pub fn sdr_stripes(stripe_size: usize, zeros_first: bool, vec: &mut Sdr) {
	let (first, second) = if zeros_first { 
		(0, 255)
	} else {
		(255,0)
	};

	for i in 0..vec.len() {
		if (i & stripe_size) == 0 {
			vec[i] = first & i as u8;
		} else {
			vec[i] = second & i as u8;
		}
	}
}



struct SeqGen {
	step: usize,
	seq_eles: Vec<u8>,
}

impl SeqGen {
	pub fn new() -> SeqGen {
		SeqGen {
			step: 0,
			seq_eles: vec![9, 200, 50, 4],
		}
	}

	pub fn next(&mut self) -> u8 {
		if self.step >= self.seq_eles.len() {
			self.step = 0;
		} else {
			self.step += 1;
		}

		self.seq_eles[self.step]
	}
}


fn junk0_vec_init(sca: u32, vec_option: usize) -> Vec<ocl::cl_uchar> {

	//let vv1 = cmn::sparse_vec(2048, -128i8, 127i8, 6);
	//ocl::fmt::print_vec(&vv1, 1, false, Some(ops::Range{ start: -127, end: 127 }));

	//let mut vec1: Vec<i8> = cmn::shuffled_vec(1024, 0, 127);
	//let mut vec1: Vec<i8> = cmn::sparse_vec(2048, -128i8, 127i8, 8);

	//ocl::fmt::print_vec(&vec1, 1, false, Some(ops::Range{ start: -128, end: 127 }));

	let mut vec1: Vec<ocl::cl_uchar> = Vec::with_capacity(sca as usize);

	//let mut vec1: Vec<ocl::cl_uchar> = iter::repeat(0).take(sc_area as usize).collect();
	/*for i in range(0, sca) {
		if i < sca >> 1 {
			vec1.push(64i8);
		} else {
			vec1.push(0i8);
		}
	}*/

	/* MAKE THIS A STRUCT OR SOMETHING */
	let sca_1_2 = sca >> 1;

	let sca_1_4 = sca >> 2;
	let sca_3_4 = sca - sca_1_4;

	let sca_1_8 = sca >> 3;
	let sca_3_8 = sca_1_2 - sca_1_8;
	let sca_5_8 = sca_1_2 + sca_1_8;

	let sca_1_16 = sca >> 4;

	//println!("##### sca_1_4: {}, sca_3_4: {} #####", sca_1_4, sca_3_4);
	/*for i in 0..sca {
		if i >= sca_3_8 + sca_1_16 && i < sca_5_8 - sca_1_16 {
		//if i >= sca_3_8 && i < sca_5_8 {
			vec1.push(0);
		} else {
			vec1.push(0);
		}
	}*/

	vec1.clear();

	if vec_option == 0 {
		vec1 = iter::repeat(0).take(sca as usize).collect();
	} else {
		for i in 0..sca {
			//if i >= sca_1_2 - (sca_1_16 / 2) && i < sca_1_2 + (sca_1_16 / 2) {
			//if ((i >= sca_1_4 - sca_1_16) && (i < sca_1_4 + sca_1_16)) || ((i >= sca_3_4 - sca_1_16) && (i < sca_3_4 + sca_1_16)) {
			//if i >= sca_3_8 && i < sca_5_8 {
			//if (i >= sca_1_2 - sca_1_16 && i < sca_1_2 + sca_1_16) || (i < sca_1_16) || (i >= (sca - sca_1_16)) {
			//if i >= sca_3_8 && i < sca_5_8 {
			//if i < sca_1_16 {
			if i >= sca_1_2 {
			//if i < sca_1_16 || i >= (sca - sca_1_16) {
				vec1.push(1);
			} else {
				vec1.push(0);
			}
		}
	}


	vec1

	/*if SHUFFLE_ONCE {
		cmn::shuffle_vec(&mut vec1);
		//chord1 = Chord::from_vec(&vec1);
	}*/

}


#[cfg(test)]
pub mod tests {
	use super::*;
	// use ocl::{ self };
	use cmn::{ CorticalDims };
	
	#[test]
	fn test_input_czar() {
		let dims = CorticalDims::new(32, 32, 1, 0, None);
		let mut ic = super::InputCzar::new(dims, 
			vec![InputSource::new(InputKind::Stripes { stripe_size: 512, zeros_first: false }, "v0")],
			0..5, false, false, false);
		//ic.set_counter(5);

		assert!(ic.counter == 5);

		ic.tick();

		assert!(ic.counter == 4);

		assert!(ic.tick() == 3, format!("(3) ic.counter == {}", ic.counter));
		assert!(ic.tick() == 2, format!("(2) ic.counter == {}", ic.counter));
		assert!(ic.tick() == 1, format!("(1) ic.counter == {}", ic.counter));
		assert!(ic.tick() == 0, format!("(0) ic.counter == {}", ic.counter));
		assert!(ic.tick() == 4, format!("(4) ic.counter == {}", ic.counter));
		assert!(ic.tick() == 3, format!("(3) ic.counter == {}", ic.counter));

	}
}
