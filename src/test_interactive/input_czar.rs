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

pub const WORLD_TURN_FACTOR: f32 = 3f32;	


pub struct InputCzar {
	counter: usize,
	counter_range: Range<usize>,
	random_counter: bool,
	//next_turn_counter: usize,
	//next_turn_max: usize,
	rng: rand::XorShiftRng,
	sc_width: u32,
	pub vec_optical: Vec<u8>,
	pub vec_motor: Vec<u8>,
	world: World,
	worm: EntityBody,
	pub motor_state: motor_state::MotorState,
}

impl InputCzar {
	pub fn new(counter_range: Range<usize>, random_counter: bool) -> InputCzar {
		let sc_width = cmn::SENSORY_CHORD_WIDTH;

		let mut world = World::new(sc_width);

		let worm =  EntityBody::new("worm", EntityKind::Creature, Location::origin());
		world.entities().add(worm);
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, -220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(220f32, 220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, -220f32)));
		world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-220f32, 220f32)));

		let mut motor_state = motor_state::MotorState::new();

		//let mut vec_motor = Vec::with_capacity(cmn::SYNAPSE_SPAN as usize);
		let mut vec_motor: Vec<u8> = iter::repeat(0).take(cmn::SYNAPSE_SPAN as usize).collect();
		vec_motor.clone_from_slice(&motor_state.cur_sdr(false));

		InputCzar {
			counter: counter_range.end,
			counter_range: counter_range,
			random_counter: random_counter,
			//next_turn_counter: 0,
			//next_turn_max: 0,
			rng: rand::weak_rng(),
			sc_width: sc_width,
			vec_optical: iter::repeat(0).take(sc_width as usize).collect(),
			vec_motor: vec_motor,
			world: world,
			worm: worm,
			motor_state: motor_state,
		}
	}


	pub fn next(&mut self, cortex: &mut Cortex) {
		
		let remain_ticks = self.tick();

		if remain_ticks == 0 {
			//print!("[> ctr = 0 <]");
			self.motor_state.switch();
		} else if remain_ticks == 1 {
			self.vec_motor.clone_from_slice(&self.motor_state.cur_sdr(true));
			//print!("[> ctr = 1 <]");
		}

		self.world.entities().get_mut(self.worm.uid).turn((WORLD_TURN_FACTOR/(self.sc_width as f32)), self.motor_state.cur_turn());
		self.world.peek_from(self.worm.uid).unfold_into(&mut self.vec_optical, 0);

		self.sense(cortex);
	}

	pub fn sense(&self, cortex: &mut Cortex) {
		cortex.write_vec(0, "thal", &self.vec_optical);
		cortex.write_vec(0, "motor", &self.vec_motor);
		cortex.cycle();
	}

	fn tick(&mut self) -> usize {
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

/*
	turn_bomb_i += 1;

			if turn_bomb_i >= turn_bomb_n {
				//print!(" >- pow!:{} -< ", turn_bomb_i);
				motor_state.switch();
				turn_bomb_i = 0;
				turn_bomb_n = 6;
				//turn_bomb_n = (rng.gen::<u8>() as usize) << 1;
			}


turn_bomb_i += 1;

			if turn_bomb_i >= turn_bomb_n {
				//print!(" >- pow!:{} -< ", turn_bomb_i);
				motor_state.switch();
				turn_bomb_i = 0;
				turn_bomb_n = 6;
				//turn_bomb_n = (rng.gen::<u8>() as usize) << 1;
			}





*/


fn test_vec_init(cortex: &mut Cortex) -> Vec<ocl::cl_uchar> {

	//let vv1 = cmn::sparse_vec(2048, -128i8, 127i8, 6);
	//cmn::print_vec(&vv1, 1, false, Some(ops::Range{ start: -127, end: 127 }));

	//let mut vec1: Vec<i8> = cmn::shuffled_vec(1024, 0, 127);
	//let mut vec1: Vec<i8> = cmn::sparse_vec(2048, -128i8, 127i8, 8);

	//cmn::print_vec(&vec1, 1, false, Some(ops::Range{ start: -128, end: 127 }));
	let scw = cmn::SENSORY_CHORD_WIDTH;

	let mut vec1: Vec<ocl::cl_uchar> = Vec::with_capacity(scw as usize);

	//let mut vec1: Vec<ocl::cl_uchar> = iter::repeat(0).take(sc_width as usize).collect();
	/*for i in range(0, scw) {
		if i < scw >> 1 {
			vec1.push(64i8);
		} else {
			vec1.push(0i8);
		}
	}*/

	/* MAKE THIS A STRUCT OR SOMETHING */
	let scw_1_2 = scw >> 1;

	let scw_1_4 = scw >> 2;
	let scw_3_4 = scw - scw_1_4;

	let scw_1_8 = scw >> 3;
	let scw_3_8 = scw_1_2 - scw_1_8;
	let scw_5_8 = scw_1_2 + scw_1_8;

	let scw_1_16 = scw >> 4;

	//println!("##### scw_1_4: {}, scw_3_4: {} #####", scw_1_4, scw_3_4);
	/*for i in 0..scw {
		if i >= scw_3_8 + scw_1_16 && i < scw_5_8 - scw_1_16 {
		//if i >= scw_3_8 && i < scw_5_8 {
			vec1.push(0);
		} else {
			vec1.push(0);
		}
	}*/


	vec1.clear();
	for i in 0..scw {
		if i >= scw_1_2 - (scw_1_16 / 2) && i < scw_1_2 + (scw_1_16 / 2) {
		//if ((i >= scw_1_4 - scw_1_16) && (i < scw_1_4 + scw_1_16)) || ((i >= scw_3_4 - scw_1_16) && (i < scw_3_4 + scw_1_16)) {
		//if i >= scw_3_8 && i < scw_5_8 {
		//if (i >= scw_1_2 - scw_1_16 && i < scw_1_2 + scw_1_16) || (i < scw_1_16) || (i >= (scw - scw_1_16)) {
		//if i >= scw_3_8 && i < scw_5_8 {
		//if i < scw_1_16 {
		//if i < scw_1_16 || i >= (scw - scw_1_16) {
			vec1.push(1);
		} else {
			vec1.push(0);
		}
	}


	vec1

	/*if SHUFFLE_ONCE {
		cmn::shuffle_vec(&mut vec1);
		//chord1 = Chord::from_vec(&vec1);
	}*/

}


#[test]
fn test_input_czar() {
	let mut ic = InputCzar::new(0..5, false);
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
