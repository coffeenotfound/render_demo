use std::collections::HashMap;
use crate::pots::object::{ObjectAllocationAddress, ObjectName, ObjectReference};

pub struct ObjectNametable {
	name_hashmap: HashMap<ObjectName, ObjectAllocationAddress>,
}

impl ObjectNametable {
	pub fn lookup(&self, name: ObjectName) -> Option<ObjectReference> {
//		self.name_hashmap.get(&name).map(|a| ObjectReference {name, allocation_address: *a})
		if let Some(address) = self.name_hashmap.get(&name) {
			Some(ObjectReference::new(name, *address))
		} else {
			None
		}
	}
}
