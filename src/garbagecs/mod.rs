//! A really shitty, garbage-tier ecs implementation.
//! But it will work. It will work.
//!
//! Pronounced /gar-bæg/-ecs

pub mod prototype;
pub mod entity_trait;
pub mod system;

mod entity_store; pub use entity_store::*;
