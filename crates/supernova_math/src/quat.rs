//! Quaternion utilities and extensions.

use glam::{Quat, Vec3};

/// Quaternion extension methods.
pub trait QuatExt {
    /// Create from Euler angles (XYZ order, in radians).
    fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Quat;
    /// Convert to Euler angles (XYZ order, in radians).
    fn to_euler(self) -> (f32, f32, f32);
    /// Spherical linear interpolation (shortest path).
    fn slerp_short(self, other: Quat, t: f32) -> Quat;
    /// Get the forward vector (-Z) after rotation.
    fn forward(self) -> Vec3;
    /// Get the right vector (+X) after rotation.
    fn right(self) -> Vec3;
    /// Get the up vector (+Y) after rotation.
    fn up(self) -> Vec3;
    /// Create a rotation that looks from `eye` toward `target` with `up`.
    fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Quat;
}

impl QuatExt for Quat {
    #[inline]
    fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Quat {
        Quat::from_euler(glam::EulerRot::XYZ, pitch, yaw, roll)
    }

    #[inline]
    fn to_euler(self) -> (f32, f32, f32) {
        self.to_euler(glam::EulerRot::XYZ)
    }

    #[inline]
    fn slerp_short(self, other: Quat, t: f32) -> Quat {
        let dot = self.dot(other);
        let other = if dot < 0.0 { -other } else { other };
        self.slerp(other, t)
    }

    #[inline]
    fn forward(self) -> Vec3 {
        self * -Vec3::Z
    }

    #[inline]
    fn right(self) -> Vec3 {
        self * Vec3::X
    }

    #[inline]
    fn up(self) -> Vec3 {
        self * Vec3::Y
    }

    #[inline]
    fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Quat {
        let forward = (target - eye).normalize_or_zero();
        if forward == Vec3::ZERO {
            return Quat::IDENTITY;
        }
        let rot = mat4_look_at(forward, up);
        Quat::from_mat3(&rot)
    }
}

fn mat4_look_at(forward: Vec3, up: Vec3) -> glam::Mat3 {
    let f = forward.normalize();
    let r = up.cross(f).normalize();
    let u = f.cross(r);
    glam::Mat3::from_cols(r, u, f).transpose()
}
