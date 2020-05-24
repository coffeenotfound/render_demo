use crate::pots::object::ObjectSlotHeader;

#[derive(Copy, Clone, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct ObjectName {
	raw: u64,
}

impl ObjectName {
	pub fn as_raw(&self) -> u64 {
		self.raw
	}
	
	pub fn from_raw(raw: u64) -> Self {
		Self {raw}
	}
	
	pub fn null() -> Self {
		Self::from_raw(0u64)
	}
}

impl Default for ObjectName {
	fn default() -> Self {
		Self::null()
	}
}

#[derive(Copy, Clone, Eq)]
pub struct ObjectReference {
	name: ObjectName,
	allocation_address: ObjectAllocationAddress,
}

impl ObjectReference {
	pub fn new(name: ObjectName, allocation_address: ObjectAllocationAddress) -> Self {
		Self {name, allocation_address}
	}
	
	pub fn name(&self) -> &ObjectName {
		&self.name
	}
	
	pub fn allocation_address(&self) -> &ObjectAllocationAddress {
		&self.allocation_address
	}
}

#[derive(Copy, Clone, Eq)]
#[repr(transparent)]
pub struct ObjectAllocationAddress {
	pub raw_ptr: *const ObjectSlotHeader,
}
