//! Supernova Math — high-performance math library built on glam.
//!
//! Provides vectors, matrices, quaternions, colors, AABBs, and transforms
//! with a clean, ergonomic API. All types are `#[repr(C)]` and `Pod` for
//! direct GPU upload.

pub mod aabb;
pub mod color;
pub mod mat;
pub mod quat;
pub mod transform;
pub mod vec;

pub use aabb::*;
pub use color::*;
pub use mat::*;
pub use quat::*;
pub use transform::*;
pub use vec::*;

// Re-export glam primitives for convenience
pub use glam::{
    IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec3A, Vec4,
};

/// Constant: π
pub const PI: f32 = std::f32::consts::PI;
/// Constant: 2π
pub const TAU: f32 = std::f32::consts::TAU;
/// Constant: π / 2
pub const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;
/// Constant: π / 4
pub const FRAC_PI_4: f32 = std::f32::consts::FRAC_PI_4;

/// Linear interpolation between `a` and `b` by `t` (0..=1).
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Smoothstep — Hermite interpolation.
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Clamp a value between `min` and `max`.
#[inline]
pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.clamp(min, max)
}

/// Degrees to radians.
#[inline]
pub fn to_radians(deg: f32) -> f32 {
    deg * (PI / 180.0)
}

/// Radians to degrees.
#[inline]
pub fn to_degrees(rad: f32) -> f32 {
    rad * (180.0 / PI)
}

/// Convert a direction vector to a yaw/pitch pair (in radians).
pub fn dir_to_yaw_pitch(dir: Vec3) -> (f32, f32) {
    let yaw = dir.x.atan2(dir.z);
    let pitch = (-dir.y).atan2((dir.x * dir.x + dir.z * dir.z).sqrt());
    (yaw, pitch)
}

/// Convert yaw/pitch (radians) to a direction vector.
pub fn yaw_pitch_to_dir(yaw: f32, pitch: f32) -> Vec3 {
    let cp = pitch.cos();
    Vec3::new(cp * yaw.sin(), -pitch.sin(), cp * yaw.cos())
}
