use std::sync::Arc;
use crate::garbagecs::prototype::PrototypeBuilder;
use crate::garbagecs::entity_trait::{TraitInstance, TraitBuilder};

pub struct PrototypeRegistry {
	registered_prototypes: Vec<Arc<EntityPrototype>>,
	registered_traits: Vec<Arc<EntityTrait>>,
}

impl PrototypeRegistry {
	pub fn new() -> Self {
		Self {
			registered_prototypes: Vec::new(),
			registered_traits: Vec::new(),
		}
	}
	
	pub fn register_prototype(&mut self, builder: PrototypeBuilder) -> Arc<EntityPrototype> {
		// Build prototype
		let (traits) = builder.build();
		
		let proto = Arc::new(EntityPrototype {
			traits,
		});
		
		// Put into list
		self.registered_prototypes.push(Arc::clone(&proto));
		proto
	}
	
	pub fn register_trait(&mut self, builder: TraitBuilder) -> Arc<EntityTrait> {
		// Build prototype
		let (factory) = builder.build();
		
		let reg_trait = Arc::new(EntityTrait {
			factory,
		});
		
		// Put into list
		self.registered_traits.push(Arc::clone(&reg_trait));
		reg_trait
	}
}

pub struct EntityPrototype {
	traits: Vec<Arc<EntityTrait>>,
}

pub struct EntityTrait {
	factory: Box<dyn FnMut() -> Box<dyn TraitInstance>>,
}
