//! World — the central ECS registry holding entities, components, resources, and events.

use std::any::{Any, TypeId};

use ahash::AHashMap;

use crate::component::ComponentStorage;
use crate::entity::{Entity, EntityIndex};
use crate::event::Events;
use crate::Resource;

/// Metadata for each entity slot.
#[derive(Clone, Copy, Debug)]
struct EntityMeta {
    generation: u32,
    alive: bool,
}

/// The world — central ECS data store.
///
/// Holds:
/// - Entity arena with generation-based reuse
/// - Per-type sparse-set component storage
/// - Global resources (singleton-like)
/// - Event buffers
pub struct World {
    entities: Vec<EntityMeta>,
    free_list: Vec<EntityIndex>,
    component_storages: AHashMap<TypeId, Box<ComponentStorage>>,
    component_type_names: AHashMap<TypeId, &'static str>,
    resources: AHashMap<TypeId, Box<dyn Any + Send + Sync>>,
    events: AHashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            free_list: Vec::new(),
            component_storages: AHashMap::new(),
            component_type_names: AHashMap::new(),
            resources: AHashMap::new(),
            events: AHashMap::new(),
        }
    }

    // ----------------------------------------------------------------------- //
    // Entity management
    // ----------------------------------------------------------------------- //

    /// Spawn a new entity. Returns its handle.
    pub fn spawn(&mut self) -> Entity {
        if let Some(index) = self.free_list.pop() {
            let meta = &mut self.entities[index as usize];
            meta.alive = true;
            Entity::new(index, meta.generation)
        } else {
            let index = self.entities.len() as EntityIndex;
            self.entities.push(EntityMeta {
                generation: 0,
                alive: true,
            });
            Entity::new(index, 0)
        }
    }

    /// Spawn an entity with a bundle of components.
    pub fn spawn_with<C: Bundle>(&mut self, bundle: C) -> Entity {
        let entity = self.spawn();
        bundle.insert(self, entity);
        entity
    }

    /// Despawn an entity and all its components.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        // Remove from all storages
        for storage in self.component_storages.values_mut() {
            storage.remove_entity(entity);
        }
        let meta = &mut self.entities[entity.index() as usize];
        meta.alive = false;
        meta.generation = meta.generation.wrapping_add(1);
        self.free_list.push(entity.index());
        true
    }

    /// Returns `true` if the entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities
            .get(entity.index() as usize)
            .map(|m| m.alive && m.generation == entity.generation())
            .unwrap_or(false)
    }

    /// Total number of alive entities.
    pub fn entity_count(&self) -> usize {
        self.entities.iter().filter(|m| m.alive).count()
    }

    /// All alive entities.
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities
            .iter()
            .enumerate()
            .filter(|(_, m)| m.alive)
            .map(|(i, m)| Entity::new(i as u32, m.generation))
    }

    // ----------------------------------------------------------------------- //
    // Component management
    // ----------------------------------------------------------------------- //

    /// Get or create the storage for component type `T`.
    fn storage<T: crate::Component>(&mut self) -> &mut ComponentStorage {
        let type_id = TypeId::of::<T>();
        self.component_type_names
            .entry(type_id)
            .or_insert_with(|| std::any::type_name::<T>());
        self.component_storages
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::new::<T>()))
    }

    /// Insert a component for `entity`.
    pub fn insert<T: crate::Component>(&mut self, entity: Entity, component: T) {
        self.storage::<T>().insert(entity, component);
    }

    /// Remove a component from `entity`.
    pub fn remove<T: crate::Component>(&mut self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.component_storages.get_mut(&type_id) {
            storage.remove(entity)
        } else {
            false
        }
    }

    /// Get a reference to a component.
    pub fn get<T: crate::Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.component_storages
            .get(&type_id)
            .and_then(|s| s.get::<T>(entity))
    }

    /// Get a mutable reference to a component.
    pub fn get_mut<T: crate::Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.component_storages
            .get_mut(&type_id)
            .and_then(|s| s.get_mut::<T>(entity))
    }

    /// Check if `entity` has component `T`.
    pub fn has<T: crate::Component>(&self, entity: Entity) -> bool {
        let type_id = TypeId::of::<T>();
        self.component_storages
            .get(&type_id)
            .map(|s| s.contains(entity))
            .unwrap_or(false)
    }

    /// Get the raw storage for component `T`.
    pub fn storage_ref<T: crate::Component>(&self) -> Option<&ComponentStorage> {
        self.component_storages
            .get(&TypeId::of::<T>())
            .map(|v| v.as_ref())
    }

    /// Get the raw mutable storage for component `T`.
    pub fn storage_mut<T: crate::Component>(&mut self) -> Option<&mut ComponentStorage> {
        self.component_storages
            .get_mut(&TypeId::of::<T>())
            .map(|v| v.as_mut())
    }

    /// Number of distinct component types currently registered.
    pub fn component_type_count(&self) -> usize {
        self.component_storages.len()
    }

    // ----------------------------------------------------------------------- //
    // Resources
    // ----------------------------------------------------------------------- //

    /// Insert a global resource (replaces if exists).
    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    /// Get a reference to a resource.
    pub fn resource<R: Resource>(&self) -> Option<&R> {
        self.resources
            .get(&TypeId::of::<R>())
            .and_then(|r| r.downcast_ref::<R>())
    }

    /// Get a mutable reference to a resource.
    pub fn resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .and_then(|r| r.downcast_mut::<R>())
    }

    /// Remove a resource.
    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources
            .remove(&TypeId::of::<R>())
            .and_then(|r| r.downcast::<R>().ok())
            .map(|b| *b)
    }

    /// Returns true if the resource exists.
    pub fn has_resource<R: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    // ----------------------------------------------------------------------- //
    // Events
    // ----------------------------------------------------------------------- //

    /// Send an event of type `E`.
    pub fn send_event<E: crate::event::Event>(&mut self, event: E) {
        let type_id = TypeId::of::<E>();
        let events = self
            .events
            .entry(type_id)
            .or_insert_with(|| Box::new(Events::<E>::new()));
        events.downcast_mut::<Events<E>>().unwrap().send(event);
    }

    /// Get the event buffer for event type `E`.
    pub fn events<E: crate::event::Event>(&self) -> Option<&Events<E>> {
        self.events
            .get(&TypeId::of::<E>())
            .and_then(|e| e.downcast_ref::<Events<E>>())
    }

    /// Get mutable access to the event buffer for event type `E`.
    pub fn events_mut<E: crate::event::Event>(&mut self) -> Option<&mut Events<E>> {
        let type_id = TypeId::of::<E>();
        if !self.events.contains_key(&type_id) {
            self.events.insert(type_id, Box::new(Events::<E>::new()));
        }
        self.events
            .get_mut(&type_id)
            .and_then(|e| e.downcast_mut::<Events<E>>())
    }

    // ----------------------------------------------------------------------- //
    // Queries
    // ----------------------------------------------------------------------- //

    /// Query for entities that have all the specified component types.
    /// Returns an iterator yielding `(Entity, &T0, &T1, ...)`.
    pub fn query<T0: crate::Component>(&self) -> crate::query::QueryIter<'_, T0> {
        crate::query::QueryIter::new(self)
    }

    /// Query for mutable access to a single component type.
    pub fn query_mut<T0: crate::Component>(&mut self) -> crate::query::QueryMutIter<'_, T0> {
        crate::query::QueryMutIter::new(self)
    }

    // ----------------------------------------------------------------------- //
    // Misc
    // ----------------------------------------------------------------------- //

    /// Clear all entities and components (resources and event buffers remain).
    pub fn clear_entities(&mut self) {
        for storage in self.component_storages.values_mut() {
            // Remove all entities from each storage
            let entities: Vec<Entity> = storage.entities().to_vec();
            for e in entities {
                storage.remove_entity(e);
            }
        }
        for meta in &mut self.entities {
            meta.alive = false;
            meta.generation = meta.generation.wrapping_add(1);
        }
        self.free_list.clear();
        // Re-add all indices to the free list
        for i in 0..self.entities.len() {
            self.free_list.push(i as u32);
        }
    }

    /// Total number of components across all types.
    pub fn total_components(&self) -> usize {
        self.component_storages.values().map(|s| s.len()).sum()
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

// --------------------------------------------------------------------------- //
// Bundle — a group of components inserted together
// --------------------------------------------------------------------------- //

/// A bundle of components that can be inserted into the world at once.
pub trait Bundle {
    fn insert(self, world: &mut World, entity: Entity);
}

// Implement Bundle for tuples of components
impl Bundle for () {
    fn insert(self, _world: &mut World, _entity: Entity) {}
}

macro_rules! impl_bundle {
    ($($T:ident),*) => {
        impl<$($T: crate::Component),*> Bundle for ($($T,)*) {
            #[allow(unused_variables)]
            fn insert(self, world: &mut World, entity: Entity) {
                let ($($T,)*) = self;
                $(
                    world.insert(entity, $T);
                )*
            }
        }
    };
}

impl_bundle!(A);
impl_bundle!(A, B);
impl_bundle!(A, B, C);
impl_bundle!(A, B, C, D);
impl_bundle!(A, B, C, D, E);
impl_bundle!(A, B, C, D, E, F);
impl_bundle!(A, B, C, D, E, F, G);
impl_bundle!(A, B, C, D, E, F, G, H);
impl_bundle!(A, B, C, D, E, F, G, H, I);
impl_bundle!(A, B, C, D, E, F, G, H, I, J);
impl_bundle!(A, B, C, D, E, F, G, H, I, J, K);
impl_bundle!(A, B, C, D, E, F, G, H, I, J, K, L);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_despawn() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.is_alive(e));
        assert!(world.despawn(e));
        assert!(!world.is_alive(e));
    }

    #[test]
    fn insert_get_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, 42i32);
        assert_eq!(world.get::<i32>(e), Some(&42));
        world.insert(e, 100i32);
        assert_eq!(world.get::<i32>(e), Some(&100));
    }

    #[test]
    fn remove_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert(e, 42i32);
        assert!(world.remove::<i32>(e));
        assert!(!world.has::<i32>(e));
    }

    #[test]
    fn entity_reuse() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.despawn(e1);
        let e2 = world.spawn();
        assert_ne!(e1, e2);
        assert_eq!(e1.index(), e2.index());
        assert_eq!(e2.generation(), e1.generation() + 1);
    }

    #[test]
    fn resource() {
        let mut world = World::new();
        world.insert_resource(String::from("hello"));
        assert_eq!(world.resource::<String>(), Some(&"hello".to_string()));
        world.resource_mut::<String>().unwrap().push_str(" world");
        assert_eq!(world.resource::<String>(), Some(&"hello world".to_string()));
    }

    #[test]
    fn bundle() {
        #[derive(Debug, PartialEq)]
        struct Pos(f32, f32, f32);
        #[derive(Debug, PartialEq)]
        struct Vel(f32, f32, f32);

        let mut world = World::new();
        let e = world.spawn_with((Pos(1.0, 2.0, 3.0), Vel(4.0, 5.0, 6.0)));
        assert_eq!(world.get::<Pos>(e), Some(&Pos(1.0, 2.0, 3.0)));
        assert_eq!(world.get::<Vel>(e), Some(&Vel(4.0, 5.0, 6.0)));
    }
}
