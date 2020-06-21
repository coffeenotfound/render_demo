use std::sync::Arc;
use crate::garbagecs::prototype::EntityTrait;

pub struct PrototypeBuilder {
	traits: Vec<Arc<EntityTrait>>,
}

impl PrototypeBuilder { 
	pub fn new() -> Self {
		Self {
			traits: Vec::new(),
		}
	}
	
	pub fn with(&mut self, registered_trait: Arc<EntityTrait>) -> &mut Self {
		self.traits.push(registered_trait);
		self
	}
	
	pub(in super::super) fn build(self) -> (Vec<Arc<EntityTrait>>) {
		(self.traits)
	}
}
