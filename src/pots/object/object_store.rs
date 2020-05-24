use std::sync::Arc;
use crate::pots::object::{ObjectName, ObjectNametable};
use crate::pots::prototype::{KindInfo};

pub struct ObjectStore {
	nametable: ObjectNametable,
	object_slabs: Vec<ObjectSlab>,
}

/// A table that stores pages of object data
/// for a single archetype.
pub struct ObjectSlab {
	/// The number of objects per page
	page_capacity: u32,
	pages: Vec<Box<ObjectSlabPage>>,
}

#[repr(C)]
struct ObjectSlabPage {
	pub data_array: [ObjectSlot]
}

#[repr(C)]
pub struct ObjectSlotHeader {
	pub resident_name: ObjectName,
	pub kind_ref: Arc<KindInfo>,
	pub mutex: parking_lot::Mutex<()>, // u8 sized
	pub slot_index: u32,
	// .. 3 byte padding ..
	// _: remaining actual trait data, aligned to u64
}
