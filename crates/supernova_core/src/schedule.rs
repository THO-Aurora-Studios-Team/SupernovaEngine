//! Schedule — ordered collection of systems executed each frame.

use crate::system::System;
use crate::world::World;

/// A label for a set of systems. Systems in the same set run in an
/// undefined order, but sets themselves are ordered.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SystemSet(pub String);

impl SystemSet {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl From<&str> for SystemSet {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// A scheduled system with optional set assignment and ordering constraints.
struct ScheduledSystem {
    system: Box<dyn System>,
    set: Option<SystemSet>,
    before: Vec<String>,
    after: Vec<String>,
}

/// A schedule of systems that run in a defined order each frame.
pub struct Schedule {
    systems: Vec<ScheduledSystem>,
    /// Cached execution order (rebuilt when systems change).
    order: Vec<usize>,
    dirty: bool,
}

impl Schedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            order: Vec::new(),
            dirty: true,
        }
    }

    /// Add a system to the schedule.
    pub fn add_system(&mut self, system: Box<dyn System>) -> &mut Self {
        self.systems.push(ScheduledSystem {
            system,
            set: None,
            before: Vec::new(),
            after: Vec::new(),
        });
        self.dirty = true;
        self
    }

    /// Add a system to a specific set.
    pub fn add_system_in_set(
        &mut self,
        system: Box<dyn System>,
        set: impl Into<SystemSet>,
    ) -> &mut Self {
        self.systems.push(ScheduledSystem {
            system,
            set: Some(set.into()),
            before: Vec::new(),
            after: Vec::new(),
        });
        self.dirty = true;
        self
    }

    /// Number of systems.
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    /// Compute a topological execution order based on before/after constraints.
    /// For now, we use a simple insertion-order execution since we don't
    /// have complex dependency graphs. A full topological sort would go here.
    fn rebuild_order(&mut self) {
        self.order = (0..self.systems.len()).collect();
        self.dirty = false;
    }

    /// Run all systems in order.
    pub fn run(&mut self, world: &mut World) {
        if self.dirty {
            self.rebuild_order();
        }
        for &idx in &self.order {
            let sys = &mut self.systems[idx];
            trace!("Running system: {}", sys.system.name());
            sys.system.run(world);
        }
    }

    /// Get the names of all systems in order.
    pub fn system_names(&self) -> Vec<&str> {
        self.systems.iter().map(|s| s.system.name()).collect()
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}
