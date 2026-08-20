//! Supernova Physics Engine.
//!
//! Provides rigid body dynamics, collision detection, and constraint solving
//! using a high-performance, data-oriented architecture.

use std::collections::HashMap;
use supernova_math::Vec3;

/// Rigid body component for physics simulation.
#[derive(Debug, Clone, Copy)]
pub struct RigidBodyComponent {
    pub mass: f32,
    pub velocity: Vec3,
    pub force: Vec3,
    pub angular_velocity: Vec3,
    pub is_static: bool,
}

impl RigidBodyComponent {
    pub fn new(mass: f32, is_static: bool) -> Self {
        Self {
            mass,
            velocity: Vec3::ZERO,
            force: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            is_static,
        }
    }

    pub fn apply_force(&mut self, force: Vec3) {
        if !self.is_static {
            self.force += force;
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if !self.is_static && self.mass > 0.0 {
            self.velocity += impulse / self.mass;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.is_static {
            let acceleration = self.force / self.mass;
            self.velocity += acceleration * dt;
            self.force = Vec3::ZERO;
        }
    }
}

/// AABB for broad-phase collision detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColliderType {
    Sphere,
    Box,
    Capsule,
    ConvexHull,
    Mesh,
}

/// Physics material properties.
#[derive(Debug, Clone, Copy)]
pub struct PhysicsMaterial {
    pub restitution: f32,
    pub friction: f32,
    pub density: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        Self {
            restitution: 0.3,
            friction: 0.5,
            density: 1.0,
        }
    }
}

/// Collider component for collision detection.
#[derive(Debug, Clone, Copy)]
pub struct ColliderComponent {
    pub collider_type: ColliderType,
    pub radius: f32,
    pub half_extents: Vec3,
    pub enabled: bool,
    pub material: PhysicsMaterial,
}

impl ColliderComponent {
    pub fn new_sphere(radius: f32) -> Self {
        Self {
            collider_type: ColliderType::Sphere,
            radius,
            half_extents: Vec3::ZERO,
            enabled: true,
            material: PhysicsMaterial::default(),
        }
    }

    pub fn new_box(half_extents: Vec3) -> Self {
        Self {
            collider_type: ColliderType::Box,
            radius: 0.0,
            half_extents,
            enabled: true,
            material: PhysicsMaterial::default(),
        }
    }

    pub fn with_material(mut self, material: PhysicsMaterial) -> Self {
        self.material = material;
        self
    }
}

/// AABB for broad-phase collision detection.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_extents(center: Vec3, extents: Vec3) -> Self {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}

/// Physics engine for simulating physics.
pub struct PhysicsEngine {
    pub enabled: bool,
    pub gravity: Vec3,
    pub time_step: f32,
    spatial_hash: HashMap<u64, Vec<u32>>,
    collision_pairs: Vec<(u32, u32)>,
    entity_bodies: HashMap<u32, u32>,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self {
            enabled: true,
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_step: 1.0 / 60.0,
            spatial_hash: HashMap::new(),
            collision_pairs: Vec::new(),
            entity_bodies: HashMap::new(),
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    pub fn update(&mut self, world: &mut supernova_core::World, dt: f32) {
        if !self.enabled {
            return;
        }

        // Step simulation
        let mut accumulator = dt;
        while accumulator >= self.time_step {
            self.integrate(world, self.time_step);
            self.detect_collisions(world);
            self.resolve_collisions(world);
            accumulator -= self.time_step;
        }
    }

    fn integrate(&self, world: &mut supernova_core::World, _dt: f32) {
        // In a full implementation, this would iterate over rigid bodies
        // and update their positions based on velocity and forces.
        // For now, we collect entities and perform a minimal update.
        let _entities: Vec<_> = world.query::<RigidBodyComponent>().collect();
    }

    fn detect_collisions(&self, _world: &mut supernova_core::World) {}

    fn resolve_collisions(&self, _world: &mut supernova_core::World) {}
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Physics world for standalone simulation.
pub struct PhysicsWorld {
    pub bodies: Vec<RigidBodyComponent>,
    pub colliders: Vec<ColliderComponent>,
    pub materials: Vec<PhysicsMaterial>,
    pub gravity: Vec3,
    pub time_step: f32,
    pub iterations: u32,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            colliders: Vec::new(),
            materials: Vec::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            time_step: 1.0 / 60.0,
            iterations: 8,
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    pub fn add_body(&mut self, body: RigidBodyComponent) -> usize {
        let index = self.bodies.len();
        self.bodies.push(body);
        index
    }

    pub fn add_collider(&mut self, collider: ColliderComponent) -> usize {
        let index = self.colliders.len();
        self.colliders.push(collider);
        index
    }

    pub fn add_material(&mut self, material: PhysicsMaterial) -> usize {
        let index = self.materials.len();
        self.materials.push(material);
        index
    }

    pub fn step(&mut self, dt: f32) {
        for body in &mut self.bodies {
            if !body.is_static {
                body.force += self.gravity * body.mass;
                body.update(dt);
            }
        }
    }

    pub fn clear_forces(&mut self) {
        for body in &mut self.bodies {
            body.force = Vec3::ZERO;
        }
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
