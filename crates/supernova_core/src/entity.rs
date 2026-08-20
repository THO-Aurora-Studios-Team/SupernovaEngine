//! Entity — a lightweight handle into the world.

use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

/// Index into the entity arena.
pub type EntityIndex = u32;

/// A 64-bit entity handle: 32-bit index + 32-bit generation.
///
/// The generation is incremented every time an entity is despawned and its
/// index recycled, so stale handles can be detected.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity(u64);

impl Entity {
    /// Create a raw entity from index + generation.
    #[inline]
    pub const fn new(index: EntityIndex, generation: u32) -> Self {
        Self((index as u64) | ((generation as u64) << 32))
    }

    #[inline]
    pub const fn index(self) -> EntityIndex {
        self.0 as u32
    }

    #[inline]
    pub const fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Placeholder used for "no entity".
    pub const PLACEHOLDER: Self = Self::new(u32::MAX, u32::MAX);
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}, {})", self.index(), self.generation())
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{}v{}", self.index(), self.generation())
    }
}

// --------------------------------------------------------------------------- //
// Global entity ID allocator for use outside the ECS world
// (e.g. networking, asset handles).
// --------------------------------------------------------------------------- //

static NEXT_ENTITY_ID: AtomicU32 = AtomicU32::new(0);

/// Allocate a unique 32-bit id. Useful for generating unique identifiers
/// outside the ECS world (e.g. asset handles, network ids).
pub fn next_id() -> u32 {
    NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_roundtrip() {
        let e = Entity::new(42, 7);
        assert_eq!(e.index(), 42);
        assert_eq!(e.generation(), 7);
    }

    #[test]
    fn entity_eq() {
        let a = Entity::new(1, 0);
        let b = Entity::new(1, 0);
        assert_eq!(a, b);
        let c = Entity::new(1, 1);
        assert_ne!(a, c);
    }
}
