//! Demo scene for Supernova Engine.
//!
//! Demonstrates how to create entities, attach components, and build
//! a simple scene using the engine's ECS.

use supernova_core::World;
use supernova_math::Vec3;
use supernova_input::{InputHandler, Key};
use supernova_physics::{RigidBodyComponent, ColliderComponent};
use supernova_scene::Transform;

/// Player component
#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub speed: f32,
    pub jump_power: f32,
}

/// Cube tag component
#[derive(Debug, Clone, Copy)]
pub struct Cube {
    pub size: Vec3,
}

/// GameObject tag component
#[derive(Debug, Clone, Copy)]
pub struct GameObject {
    pub name: &'static str,
}

/// System: Update player movement based on input.
pub fn player_movement_system(world: &mut World, input: &InputHandler, dt: f32) {
    let entities: Vec<_> = world.query::<Player>().collect();

    for (entity, player) in entities {
        let mut velocity = Vec3::ZERO;

        if input.is_key_down(Key::W) {
            velocity.z -= player.speed;
        }
        if input.is_key_down(Key::S) {
            velocity.z += player.speed;
        }
        if input.is_key_down(Key::A) {
            velocity.x -= player.speed;
        }
        if input.is_key_down(Key::D) {
            velocity.x += player.speed;
        }

        if let Some(transform) = world.get_mut::<Transform>(entity) {
            transform.translation += velocity * dt;
        }
    }
}

/// System: Update physics simulation.
pub fn physics_system(world: &mut World) {
    const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

    let bodies: Vec<_> = world.query::<RigidBodyComponent>().collect();
    for (entity, body) in bodies {
        if !body.is_static {
            if let Some(rb) = world.get_mut::<RigidBodyComponent>(entity) {
                rb.force += GRAVITY * rb.mass;
                rb.update(0.016);
            }
        }
    }
}

/// Example scene creation function
pub fn create_demo_scene(world: &mut World) {
    // Create player entity
    let _player = world.spawn_with((
        GameObject { name: "Player" },
        Transform::from_position(Vec3::new(0.0, 0.0, 0.0)),
    ));

    // Create cube entity
    let _cube = world.spawn_with((
        GameObject { name: "Cube" },
        Transform::from_position(Vec3::new(2.0, 0.0, 0.0)),
        RigidBodyComponent::new(1.0, false),
        ColliderComponent::new_box(Vec3::new(0.5, 0.5, 0.5)),
    ));

    // Create floor (static ground)
    let _floor = world.spawn_with((
        GameObject { name: "Floor" },
        Transform::from_position(Vec3::new(0.0, -5.0, 0.0)),
        RigidBodyComponent::new(0.0, true),
        ColliderComponent::new_sphere(10.0),
    ));
}
