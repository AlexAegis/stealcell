use stealcell::StealCell;

struct Thing {
	value: usize,
}

impl Thing {
	/// A test function that requires mutable access on itself and za warudo
	fn hello_world(&mut self, world: &mut World) {
		println!("hello {}, my value is {}.", world.name, self.value);
	}
}

struct World {
	name: String,
	thing: StealCell<Thing>,
}

fn main() {
	let mut world = World {
		name: "world".to_string(),
		thing: StealCell::new(Thing { value: 1 }),
	};
	let mut stolen_thing = world.thing.steal();
	stolen_thing.as_mut().hello_world(&mut world); // `.get_mut()` only needed with `no_std`
	// If you skip this and let the stolen value drop, you get a panic!
	world.thing.return_stolen(stolen_thing);
}
