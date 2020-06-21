use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::garbagecs::prototype::{PrototypeId, EntityPrototype};
use crate::garbagecs::entity_trait::TraitInstance;

pub struct EntityStore {
	entity_slabs: HashMap<PrototypeId, EntitySlab>,
}

impl EntityStore {
	pub fn new() -> Self {
		Self {
			entity_slabs: HashMap::new(),
		}
	}
}

pub struct EntitySlab {
	entities: Vec<Arc<Mutex<EntityData>>>,
}

pub(in super) struct EntityData {
	entity_id: EntityId,
	prototype_ref: Arc<EntityPrototype>,
	trait_data: Vec<Box<dyn TraitInstance>>,
}

#[repr(transparent)]
pub struct EntityId {
	raw: u64
}

pub struct EntityReference {
	pub(in super) raw_ref: Arc<Mutex<EntityData>>,
}
