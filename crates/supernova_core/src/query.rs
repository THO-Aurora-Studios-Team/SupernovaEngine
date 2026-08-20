//! Query system — iterate over entities matching a component filter.

use std::marker::PhantomData;

use crate::component::ComponentStorage;
use crate::entity::Entity;
use crate::world::World;
use crate::Component;

/// Immutable iterator over entities that have component `T`.
pub struct QueryIter<'w, T: Component> {
    storage: Option<&'w ComponentStorage>,
    index: usize,
    _marker: PhantomData<&'w T>,
}

impl<'w, T: Component> QueryIter<'w, T> {
    pub fn new(world: &'w World) -> Self {
        Self {
            storage: world.storage_ref::<T>(),
            index: 0,
            _marker: PhantomData,
        }
    }
}

impl<'w, T: Component> Iterator for QueryIter<'w, T> {
    type Item = (Entity, &'w T);

    fn next(&mut self) -> Option<Self::Item> {
        let storage = self.storage?;
        if self.index >= storage.len() {
            return None;
        }
        let entity = storage.entities()[self.index];
        let data = unsafe { storage.as_slice::<T>() };
        let item = (entity, &data[self.index]);
        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.storage.map_or(0, |s| s.len().saturating_sub(self.index));
        (remaining, Some(remaining))
    }
}

impl<'w, T: Component> ExactSizeIterator for QueryIter<'w, T> {
    fn len(&self) -> usize {
        self.storage.map_or(0, |s| s.len().saturating_sub(self.index))
    }
}

/// Mutable iterator over entities that have component `T`.
pub struct QueryMutIter<'w, T: Component> {
    data: *mut T,
    entities: *const Entity,
    count: usize,
    current: usize,
    _marker: PhantomData<(&'w T, &'w mut T)>,
}

impl<'w, T: Component> QueryMutIter<'w, T> {
    pub fn new(world: &'w mut World) -> Self {
        match world.storage_mut::<T>() {
            Some(storage) => {
                let count = storage.len();
                let entities_ptr = storage.entities().as_ptr();
                // SAFETY: exclusive access to world, so storage is exclusively borrowed
                let data_ptr = unsafe { storage.as_mut_slice::<T>().as_mut_ptr() };
                Self {
                    data: data_ptr,
                    entities: entities_ptr,
                    count,
                    current: 0,
                    _marker: PhantomData,
                }
            }
            None => Self {
                data: std::ptr::NonNull::dangling().as_ptr(),
                entities: std::ptr::NonNull::dangling().as_ptr(),
                count: 0,
                current: 0,
                _marker: PhantomData,
            },
        }
    }
}

impl<'w, T: Component> Iterator for QueryMutIter<'w, T> {
    type Item = (Entity, &'w mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            return None;
        }
        let idx = self.current;
        self.current += 1;
        // SAFETY: distinct indices, exclusive access granted at construction
        let entity = unsafe { *self.entities.add(idx) };
        let item = unsafe { &mut *self.data.add(idx) };
        Some((entity, item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count - self.current;
        (remaining, Some(remaining))
    }
}

impl<'w, T: Component> ExactSizeIterator for QueryMutIter<'w, T> {
    fn len(&self) -> usize {
        self.count - self.current
    }
}

unsafe impl<'w, T: Component + Send> Send for QueryMutIter<'w, T> {}
unsafe impl<'w, T: Component + Sync> Sync for QueryMutIter<'w, T> {}

/// A borrowed query handle that can be used to iterate multiple times.
pub struct QueryBorrow<'w, T: Component> {
    world: &'w World,
    _marker: PhantomData<T>,
}

impl<'w, T: Component> QueryBorrow<'w, T> {
    pub fn new(world: &'w World) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> QueryIter<'_, T> {
        QueryIter::new(self.world)
    }
}

/// A query descriptor — entry point for iterating components.
pub struct Query<T> {
    _marker: PhantomData<T>,
}

impl<T: Component> Query<T> {
    /// Immutable iteration.
    pub fn iter<'w>(world: &'w World) -> QueryIter<'w, T> {
        QueryIter::new(world)
    }

    /// Mutable iteration.
    pub fn iter_mut<'w>(world: &'w mut World) -> QueryMutIter<'w, T> {
        QueryMutIter::new(world)
    }
}
