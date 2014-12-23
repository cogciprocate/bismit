use microcosm::entity::{ EntityBody, EntityKind, EntityBrain, Mobile };
use microcosm::worm::{ WormBrain };
use microcosm::common::{ Location, Peek, Scent };
use microcosm::world::{ World };
use sub_cortex::{ SubCortex };
use cortex::{ Cortex };
use chord::{ Chord };

use std::option::{ Option };


pub fn run() {
	let mut world: World = World::new();

	let worm =  EntityBody::new("worm".to_string(), EntityKind::Creature, Location::origin());
	let snake = EntityBody::new("snake".to_string(), EntityKind::Creature, Location::new(60f32, 60f32));

	let food = EntityBody::new("food".to_string(), EntityKind::Food, Location::new(50f32, 50f32));
	
	let poison = EntityBody::new("poison".to_string(), EntityKind::Poison, Location::new(-100f32, -50f32));

	let worm_uid = worm.uid;
	let snake_uid = snake.uid;

 
	world.entities().add(worm);
	world.entities().add(food);
	world.entities().add(snake);
	world.entities().add(poison);
	world.entities().add(EntityBody::new("food".to_string(), EntityKind::Food, Location::new(150f32, -200f32)));
	world.entities().add(EntityBody::new("food".to_string(), EntityKind::Food, Location::new(-150f32, -250f32)));
	world.entities().add(EntityBody::new("food".to_string(), EntityKind::Food, Location::new(550f32, -200f32)));
	world.entities().add(EntityBody::new("food".to_string(), EntityKind::Food, Location::new(-1150f32, -250f32)));
	world.entities().add(EntityBody::new("food".to_string(), EntityKind::Food, Location::new(0f32, 110f32)));
	world.entities().add(EntityBody::new("food".to_string(), EntityKind::Food, Location::new(-50f32, 0f32)));
	world.entities().add(EntityBody::new("food".to_string(), EntityKind::Food, Location::new(0f32, -50f32)));
	world.entities().add(EntityBody::new("food".to_string(), EntityKind::Food, Location::new(130f32, 0f32)));

	world.entities().print();

	world.peek_from(worm_uid);

	world.entities().get_mut(worm_uid).turn(0.25f32);

	//world.peek_from(worm_uid);



	let mut worm_brain = EntityBrain::new(worm_uid, &world);

	let mut snake_brain = SnakeBrain::new(snake_uid);


	let chord = render_peek(world.peek_from(worm_uid));
	chord.print();
	chord.unfold().print();

	for i in range(0u, 100000) {
		if worm_brain.act(&mut world) == Option::None {
			println!("Everything eaten after {} iterations.", i);
			break
		}
		
		snake_brain.act(&mut world);
	}

	//render_peek(world.peek_from(worm_uid)).print();

	//worm_brain.print();
	//world.entities().print();
}

pub struct SnakeBrain {
	pub cort: Cortex,
	pub subc: SubCortex,
	pub body_uid: uint,
}
impl SnakeBrain {
	pub fn new(body_uid: uint) -> SnakeBrain {
		SnakeBrain { 
			cort: Cortex::new(),
			subc: SubCortex::new(),
			body_uid: body_uid,
		}
	}

	pub fn act(&mut self, world: &mut World) {
		let scent_new: Scent = world.sniff_from(self.body_uid);
		render_peek(world.peek_from(self.body_uid));
	}
}

fn render_peek(box peek: Box<Peek>) -> Chord {
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
