//! Supernova Core — Entity-Component-System framework, events, resources, and scheduling.
//!
//! The ECS uses a sparse-set storage model: each component type gets its own
//! sparse-set, giving O(1) insertion/removal and excellent iteration speed
//! with dense, contiguous arrays for cache-friendly queries.

pub mod entity;
pub mod world;
pub mod component;
pub mod query;
pub mod system;
pub mod event;
pub mod resource;
pub mod schedule;
pub mod app;
pub mod time;

pub use entity::{Entity, EntityIndex};
pub use world::World;
pub use component::Component;
pub use query::{Query, QueryIter, QueryMutIter, QueryBorrow};
pub use system::{System, IntoSystem, FnSystem};
pub use event::{EventReader, EventWriter, Events};
pub use resource::Resource;
pub use schedule::{Schedule, SystemSet};
pub use app::App;
pub use time::Time;

/// Global entity-id type used throughout the engine.
pub use entity::Entity as EcsEntity;

#[macro_use]
extern crate log;
