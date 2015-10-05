// use std::iter;

use cmn::{ self, Sdr };

pub struct MotorState {
	turn_left: bool,  // change to turn_state
	sdr_left: Vec<u8>,
	sdr_right: Vec<u8>,
	left_str: &'static str,
	right_str: &'static str,
}

impl MotorState {
	pub fn new() -> MotorState {

		let turn_left = true;
		let sdr_left = cmn::gen_fract_sdr(50, 256);
		let sdr_right = cmn::gen_fract_sdr(129, 256);

		//let sdr_left = iter::repeat(0).take(256).collect();
		//let sdr_right = iter::repeat(0).take(256).collect();

		
		MotorState {
			turn_left: turn_left,
			sdr_left: sdr_left,
			sdr_right: sdr_right,
			left_str: "left",
			right_str: "right",
		}
	}

	pub fn switch(&mut self) {
		self.turn_left = !self.turn_left;
	}

	pub fn cur_turn(&self) -> bool {
		self.turn_left
	}

	pub fn cur_str(&self) -> &'static str {
		if self.turn_left {
			self.left_str
		} else {
			self.right_str
		}
	}

	pub fn cur_sdr(&self, rev: bool) -> &Sdr {
		if self.turn_left ^ rev {
			&self.sdr_left[..]
		} else {
			&self.sdr_right[..]
		}
	}
}


enum TurnState {
	Left,
	Right,
}
