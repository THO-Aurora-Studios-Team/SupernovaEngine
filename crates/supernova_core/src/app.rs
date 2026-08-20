//! App — the top-level engine builder and runner.

use crate::schedule::Schedule;
use crate::system::System;
use crate::time::Time;
use crate::world::World;

/// The application — holds the world, schedule, and configuration.
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub running: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedule: Schedule::new(),
            running: false,
        }
    }

    /// Add a system to the main schedule.
    pub fn add_system(&mut self, system: Box<dyn System>) -> &mut Self {
        self.schedule.add_system(system);
        self
    }

    /// Add a resource to the world.
    pub fn insert_resource<R: crate::Resource>(&mut self, resource: R) -> &mut Self {
        self.world.insert_resource(resource);
        self
    }

    /// Initialize and run the main loop.
    ///
    /// `callback` is called each frame with `(dt, &mut World)`. The caller
    /// is responsible for driving the event loop (e.g. winit's event loop)
    /// and calling `app.update(dt)` once per frame.
    pub fn run<F>(&mut self, mut callback: F)
    where
        F: FnMut(f32, &mut World, &mut Schedule),
    {
        self.running = true;
        // The actual frame loop is driven externally (by the platform layer).
        // This method is a convenience for headless/CLI usage.
        let mut last = std::time::Instant::now();
        while self.running {
            let now = std::time::Instant::now();
            let dt = (now - last).as_secs_f32() as f32;
            last = now;

            callback(dt, &mut self.world, &mut self.schedule);
            self.schedule.run(&mut self.world);
        }
    }

    /// Run one frame. Call this from your platform's event loop.
    pub fn update(&mut self, dt: f32) {
        // Store dt as a resource so systems can access it.
        self.world.insert_resource(Time { dt, elapsed: 0.0 });
        self.schedule.run(&mut self.world);
    }

    /// Stop the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
