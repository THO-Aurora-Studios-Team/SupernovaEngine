//! Supernova Core — Entity-Component-System framework, events, resources, and scheduling.
//!
//! The ECS uses a sparse-set storage model: each component type gets its own
//! sparse-set, giving O(1) insertion/removal and excellent iteration speed
//! with dense, contiguous arrays for cache-friendly queries.

pub mod app;
pub mod component;
pub mod entity;
pub mod event;
pub mod query;
pub mod resource;
pub mod schedule;
pub mod system;
pub mod time;
pub mod world;

pub use app::App;
pub use component::Component;
pub use entity::{Entity, EntityIndex};
pub use event::{EventReader, EventWriter, Events};
pub use query::{Query, QueryBorrow, QueryIter, QueryMutIter};
pub use resource::Resource;
pub use schedule::{Schedule, SystemSet};
pub use system::{FnSystem, IntoSystem, System};
pub use time::Time;
pub use world::World;

/// Global entity-id type used throughout the engine.
pub use entity::Entity as EcsEntity;

#[macro_use]
extern crate log;
