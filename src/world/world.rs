use crate::garbagecs::EntityStore;

pub struct World {
	entity_store: EntityStore,
}

impl World {
	pub fn new() -> Self {
		Self {
			entity_store: EntityStore::new(),
		}
	}
}
