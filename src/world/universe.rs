use crate::world::World;
use crate::garbagecs::prototype::PrototypeRegistry;

pub struct Universe {
	pub prototype_registry: PrototypeRegistry,
	
	pub loaded_worlds: Vec<World>,
}

impl Universe {
	pub fn new() -> Self {
		Self {
			prototype_registry: PrototypeRegistry::new(),
			
			loaded_worlds: Vec::new(),
		}
	}
}
