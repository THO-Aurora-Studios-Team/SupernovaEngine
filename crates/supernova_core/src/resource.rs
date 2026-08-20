//! Resource trait — global singleton state stored in the World.
//!
//! Resources are accessed via [`World::resource`](crate::world::World::resource).
//! Any type that is `Any + Send + Sync` automatically implements `Resource`.

use std::any::Any;

/// Marker trait for resources — global singleton state.
///
/// No manual implementation is needed: any `Send + Sync` type is a `Resource`.
pub trait Resource: Any + Send + Sync {}

impl<T: Any + Send + Sync> Resource for T {}
