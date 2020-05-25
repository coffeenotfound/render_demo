use std::collections::HashMap;
use std::sync::Arc;
use crate::garbagecs::{Prototype, PrototypeId};

pub struct EntityStore {
	entity_slabs: HashMap<PrototypeId, EntitySlab>,
}

pub struct EntitySlab {
	entities: Vec<EntityData>,
}

struct EntityData {
	entity_id: EntityId,
	prototype_ref: Arc<Prototype>,
	trait_data: Vec<dyn Trait>,
}

#[repr(transparent)]
pub struct EntityId {
	raw: u64
}

pub struct EntityReference {
	pub(in super) raw_ref: Arc<EntityData>,
}
