//! Main entry point for a Supernova Engine game.
//!
//! Shows how to set up the engine with a demo scene and main loop.

use supernova_engine::SupernovaEngine;
use supernova_math::Vec3;
use supernova_scene::Transform;

fn main() {
    println!("Supernova Engine v0.1.0");
    println!("========================================");
    println!("Initializing engine...");

    let mut engine = SupernovaEngine::new();
    engine.initialize();

    // Create a demo scene
    {
        let world = engine.world_mut();
        let _player = world.spawn_with((
            Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
        ));
    }

    println!("Engine initialized successfully.");
    println!("Starting main loop...");

    let start_time = std::time::Instant::now();
    let mut frame_count = 0u32;

    loop {
        engine.update(0.016);
        frame_count += 1;

        if start_time.elapsed().as_secs() >= 5 {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    println!("Rendered {} frames in 5 seconds", frame_count);
    engine.stop();
    println!("Engine stopped. Goodbye!");
}
