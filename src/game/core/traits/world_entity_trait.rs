use crate::garbagecs::entity_trait::TraitInstance;
use cgmath::{Vector3, Zero};

pub struct WorldEntityTrait {
	pub transform: Transform,
}

impl TraitInstance for WorldEntityTrait {}

pub struct Transform {
	pub translation: Vector3<f32>,
}

impl Default for Transform {
	fn default() -> Self {
		Self {
			translation: Vector3::zero(),
		}
	}
}
