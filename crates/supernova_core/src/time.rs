//! Time resource — updated each frame by the engine.

/// Frame timing information.
#[derive(Clone, Copy, Debug, Default)]
pub struct Time {
    /// Delta time in seconds.
    pub dt: f32,
    /// Total elapsed time in seconds.
    pub elapsed: f32,
}

// Time automatically implements Resource via the blanket impl.

impl Time {
    /// Delta time in milliseconds.
    pub fn dt_ms(&self) -> f32 {
        self.dt * 1000.0
    }

    /// Frames per second estimate.
    pub fn fps(&self) -> f32 {
        if self.dt > 0.0 {
            1.0 / self.dt
        } else {
            0.0
        }
    }
}
