# [stealcell](https://github.com/AlexAegis/stealcell)

[![crates.io](https://img.shields.io/crates/v/stealcell.svg)](https://crates.io/crates/stealcell)
[![ci](https://github.com/AlexAegis/stealcell/actions/workflows/ci.yml/badge.svg)](https://github.com/AlexAegis/stealcell/actions/workflows/ci.yml)
[![codecov](https://codecov.io/github/AlexAegis/stealcell/graph/badge.svg?token=w1UtRLE5cc)](https://codecov.io/github/AlexAegis/stealcell)

An Option like type that lets you temporarily remove a value from somewhere to
retain mutable access on both.

## Example & Usage

```sh
cargo run --example stealcell_example
```

```rs
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
    stolen_thing.hello_world(&mut world);
    // If you skip this and let the stolen value drop, you get a panic!
    world.thing.return_stolen(stolen_thing);
}

```

## For Maintainers

See [contributing.md](https://github.com/AlexAegis/stealcell?tab=contributing-ov-file#contributing)
