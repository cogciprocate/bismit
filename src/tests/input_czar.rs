use cmn;
use ocl;
use cortex::{ Cortex };
use super::motor_state;
use microcosm::world::{ World };
use microcosm::entity::{ EntityBody, EntityKind, EntityBrain, Mobile };
use microcosm::worm::{ WormBrain };
use microcosm::common::{ Location, Peek, Scent, WORM_SPEED, TAU };

use std::iter;
use std::ops::{ Range };
use rand::{ self, ThreadRng, Rng };

pub const WORLD_TURN_FACTOR: f32 				= 3f32;	


//pub const PARAM_RANDOM_COUNTER: bool = true;


pub struct InputCzar {
	counter: usize,
	counter_range: Range<usize>,
	random_counter: bool,
	toggle_dirs: bool,
	introduce_noise: bool,
	ttl_count: usize,
	reset_count: usize,
	//next_turn_counter: usize,
	//next_turn_max: usize,
	rng: rand::XorShiftRng,
	area: u32,
	optical_vec_kind: InputVecKind,
	pub vec_optical: Vec<u8>,
	pub vec_motor: Vec<u8>,
	pub vec_test_noise: Vec<u8>,
	world: World,
	worm: EntityBody,
	pub motor_state: motor_state::MotorState,
}

impl InputCzar {
	pub fn new(area: u32, optical_vec_kind: InputVecKind, counter_range: Range<usize>, random_counter: bool, toggle_dirs: bool, introduce_noise: bool) -> InputCzar {

		let mut world = World::new(area);

		let worm =  EntityBody::new("worm", EntityKind::Creature, Location::origin());
		world.entities().add(worm);
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, -220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, 220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, -220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, 220f32)));

		let mut motor_state = motor_state::MotorState::new();

		let mut vec_motor: Vec<u8> = iter::repeat(0).take(cmn::SYNAPSE_SPAN_LIN as usize).collect();
		
		if toggle_dirs {
			vec_motor.clone_from_slice(&motor_state.cur_sdr(false));
		}

		let vec_test_noise = junk0_vec_init(cmn::SYNAPSE_SPAN_LIN, 0);

		InputCzar {
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
			area: area,
			optical_vec_kind: optical_vec_kind,
			vec_optical: iter::repeat(0).take(area as usize).collect(),
			vec_motor: vec_motor,
			vec_test_noise: vec_test_noise,
			world: world,
			worm: worm,
			motor_state: motor_state,
		}
	}


	pub fn next(&mut self, cortex: &mut Cortex) {
		let remain_ticks = self.tick();

		if self.introduce_noise {
			/*if (self.ttl_count & 0x01) == 0x01 {
				//if (self.reset_count & 0x01) == 0x01 {
				self.vec_test_noise = test_vec_init(cmn::SYNAPSE_SPAN_LIN, 0);
			} else {
				self.vec_test_noise = test_vec_init(cmn::SYNAPSE_SPAN_LIN, 1);
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
		match self.optical_vec_kind {
			InputVecKind::World => {
				self.world.entities().get_mut(self.worm.uid).turn((WORLD_TURN_FACTOR/(self.area as f32)), self.motor_state.cur_turn());
				self.world.peek_from(self.worm.uid).unfold_into(&mut self.vec_optical, 0);
			},
			InputVecKind::Band_512 => {
				vec_band_512_fill(&mut self.vec_optical);
			},
		}


		/* ##### TEST_NOISE ##### */
		// nothing here yet

		self.sense(cortex);
	}

	pub fn sense(&self, cortex: &mut Cortex) {
		cortex.write_vec(0, "thal", &self.vec_optical);
		cortex.write_vec(0, "motor", &self.vec_motor);
		//cortex.write_vec(0, "test_noise", &self.vec_test_noise);
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

pub enum InputVecKind {
	World,
	Band_512,
}

pub fn vec_band_512_fill(vec: &mut Vec<u8>) {
	for i in 0..vec.len() {
		if (i & 512) == 0 {
			vec[i] = 0;
		} else {
			vec[i] = 1;
		}
	}
}


fn junk0_vec_init(sca: u32, vec_option: usize) -> Vec<ocl::cl_uchar> {

	//let vv1 = cmn::sparse_vec(2048, -128i8, 127i8, 6);
	//cmn::print_vec(&vv1, 1, false, Some(ops::Range{ start: -127, end: 127 }));

	//let mut vec1: Vec<i8> = cmn::shuffled_vec(1024, 0, 127);
	//let mut vec1: Vec<i8> = cmn::sparse_vec(2048, -128i8, 127i8, 8);

	//cmn::print_vec(&vec1, 1, false, Some(ops::Range{ start: -128, end: 127 }));

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
mod tests {
	
	#[test]
	fn test_input_czar() {
		let mut ic = super::InputCzar::new(1024, super::InputVecKind::Band_512, 0..5, false, false, false);
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
