use crate::garbagecs::entity_trait::TraitInstance;

pub struct TraitBuilder {
	factory: Box<dyn FnMut() -> Box<dyn TraitInstance>>,
}

impl TraitBuilder {
	pub fn new(factory: Box<dyn FnMut() -> Box<dyn TraitInstance>>) -> Self {
		Self {
			factory,
		}
	}
	
	pub(in super::super) fn build(self) -> (Box<dyn FnMut() -> Box<dyn TraitInstance>>) {
		(self.factory)
	}
}
