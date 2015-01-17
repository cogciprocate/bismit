use microcosm::entity::{ EntityBody, EntityKind, EntityBrain, Mobile };
use microcosm::worm::{ WormBrain };
use microcosm::common::{ Location, Peek, Scent, WORM_SPEED, TAU };
use microcosm::world::{ World };
use sub_cortex::{ SubCortex };
use cortex::{ Cortex };
use chord::{ Chord };
use ocl;
use std::clone::Clone;
//use std::ptr;

use std::option::{ Option };


pub fn run() {
	let mut world: World = World::new();

	let worm =  EntityBody::new("worm", EntityKind::Creature, Location::origin());
	let snake = EntityBody::new("snake", EntityKind::Creature, Location::new(60f32, 60f32));

	let food = EntityBody::new("food", EntityKind::Food, Location::new(50f32, 50f32));
	
	let poison = EntityBody::new("poison", EntityKind::Poison, Location::new(-100f32, -50f32));

	let worm_uid = worm.uid;
	let snake_uid = snake.uid;


	world.entities().add(worm);
	world.entities().add(food);
	world.entities().add(snake);
	world.entities().add(poison);
	//world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(150f32, -200f32)));
	//world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-150f32, -250f32)));
	//world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(550f32, -200f32)));
	//world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-1150f32, -250f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(0f32, 110f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(-50f32, 0f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(0f32, -50f32)));
	world.entities().add(EntityBody::new("food", EntityKind::Food, Location::new(130f32, 0f32)));

	world.entities().print();

	world.peek_from(worm_uid);

	world.entities().get_mut(worm_uid).turn(0.25f32);

	//world.peek_from(worm_uid);



	let mut worm_brain = EntityBrain::new(worm_uid, &world);

	let mut snake_brain = SnakeBrain::new(snake_uid);


	let chord = render_peek(world.peek_from(worm_uid));
	//chord.print();
	//chord.unfold().print();

	for i in range(0, 100000) {
		if worm_brain.act(&mut world) == Option::None {
			println!("Everything eaten after {} iterations.", i);
			break
		}
		
		snake_brain.act(&mut world);
		//break
	}

	//render_peek(world.peek_from(worm_uid)).print();

	//worm_brain.print();
	//world.entities().print();

	snake_brain.cort.cortex_segments[0].columns.synapses.print_values(&snake_brain.cort.ocl);

	snake_brain.cort.release_components();
}

pub struct SnakeBrain {
	pub cort: Cortex,
	pub subc: SubCortex,
	pub body_uid: usize,
}
impl SnakeBrain {
	pub fn new(body_uid: usize) -> SnakeBrain {
		SnakeBrain { 
			cort: Cortex::new(),
			subc: SubCortex::new(),
			body_uid: body_uid,
		}
	}

	pub fn act(&mut self, world: &mut World) {
		let scent: Scent = world.sniff_from(self.body_uid);
		let peek_chord = render_peek(world.peek_from(self.body_uid));
		self.cort.sense(0, &peek_chord);

		self.propel(world, 0.2f32, 0.1f32);

		if scent.sweet >= 1f32 {
			world.feed_entity(self.body_uid);
		}

		
	}

	fn propel(&mut self, world: &mut World, left: f32, right: f32) {

		let body = world.entities().get_mut(self.body_uid);

		let distance = WORM_SPEED;

		body.heading += (left - right) * WORM_SPEED;

		body.propel();

	}

}



trait SnakeCortex {
	fn sense_peek(&mut self, peek_chord: &Chord);
	fn release(&mut self);
}
impl SnakeCortex for Cortex {
	fn sense_peek(&mut self, pc: &Chord) {

		/*
		let mut output: Vec<u8> = Vec::with_capacity(peek_chord.len());
		for i in range(0u, output.capacity()) {
			output.push(Default::default());
		}
		*/



		//self.sense(0u, pc);

		/*

		let peek_chord = pc.unfold().notes.to_vec();

		let peek_chord_buff: ocl::cl_mem = ocl::new_write_buffer(&peek_chord, self.ocl.context);
		ocl::enqueue_write_buffer(&peek_chord, peek_chord_buff, self.ocl.command_queue);

		//let output_buff: ocl::cl_mem = ocl::new_read_buffer(&output, self.ocl.context);

		let sense_kernel_name = "sense";
		let sense_kernel: ocl::cl_kernel = ocl::new_kernel(self.ocl.program, sense_kernel_name);

		ocl::set_kernel_arg(0, peek_chord_buff, sense_kernel);
		//ocl::set_kernel_arg(1, output_buff, sense_kernel);

		ocl::enqueue_kernel(sense_kernel, self.ocl.command_queue, peek_chord.len());
		//ocl::enqueue_read_buffer(&test_out, test_out_buff, self.ocl.command_queue);

		ocl::release_mem_object(peek_chord_buff);
		ocl::release_kernel(sense_kernel);
		*/

	}

	fn release(&mut self) {

		self.release_components();
	}
}

fn render_peek(peek: Box<Peek>) -> Chord {
	let mut chord = Chord::new();

	for p in peek.peek.iter() {
		let (a, v) = *p;
		chord.note_gt(a, v);
	}

	chord
}

/*
fn renderScent(scent: Scent) -> Chord {

}
*/
