//! System trait and function-system adapter.

use crate::world::World;

/// A system is a function that operates on the world each frame.
pub trait System: Send + Sync {
    fn run(&mut self, world: &mut World);
    fn name(&self) -> &str;
}

/// Adapter that turns any `Fn(&mut World)` into a system.
pub struct FnSystem<F> {
    func: F,
    name: String,
}

impl<F> FnSystem<F>
where
    F: Fn(&mut World) + Send + Sync,
{
    pub fn new(name: impl Into<String>, func: F) -> Self {
        Self {
            func,
            name: name.into(),
        }
    }
}

impl<F> System for FnSystem<F>
where
    F: Fn(&mut World) + Send + Sync,
{
    fn run(&mut self, world: &mut World) {
        (self.func)(world);
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Trait for converting functions into boxed systems.
pub trait IntoSystem {
    fn into_system(self) -> Box<dyn System>;
}

impl<F> IntoSystem for F
where
    F: Fn(&mut World) + Send + Sync + 'static,
{
    fn into_system(self) -> Box<dyn System> {
        Box::new(FnSystem::new(std::any::type_name::<F>(), self))
    }
}

/// Helper to create a named system.
pub fn system<F: Fn(&mut World) + Send + Sync + 'static>(
    name: impl Into<String>,
    func: F,
) -> Box<dyn System> {
    Box::new(FnSystem::new(name, func))
}
