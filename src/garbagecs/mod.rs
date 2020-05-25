//! A really shitty, garbage-tier ecs implementation.
//! But it will work. It will work.
//!
//! Pronounced /gar-bæg/-ecs

mod prototype; pub use prototype::*;
mod entity_trait; pub use entity_trait::*;
mod entity_store; pub use entity_store::*;
mod prototype_registry; pub use prototype_registry::*;
