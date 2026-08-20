//! Transform — position, rotation, scale for 2D and 3D.

use crate::quat::QuatExt;
use glam::{Mat4, Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 3D transform with TRS (translation, rotation, scale).
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    #[inline]
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    #[inline]
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Self::IDENTITY
        }
    }

    /// Compute the local-to-world matrix.
    #[inline]
    pub fn to_matrix(self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_scale(self.scale)
    }

    /// Lerp between two transforms (uses slerp for rotation).
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }

    /// Translate by a local-space offset (affected by rotation).
    #[inline]
    pub fn translate_local(&mut self, offset: Vec3) {
        self.position += self.rotation * offset;
    }

    /// Translate by a world-space offset.
    #[inline]
    pub fn translate_world(&mut self, offset: Vec3) {
        self.position += offset;
    }

    /// Rotate by a quaternion in local space.
    #[inline]
    pub fn rotate_local(&mut self, rot: Quat) {
        self.rotation = self.rotation * rot;
    }

    /// Rotate by a quaternion in world space.
    #[inline]
    pub fn rotate_world(&mut self, rot: Quat) {
        self.rotation = rot * self.rotation;
    }

    /// Get the forward vector (-Z).
    #[inline]
    pub fn forward(self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    /// Get the right vector (+X).
    #[inline]
    pub fn right(self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up vector (+Y).
    #[inline]
    pub fn up(self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Look at a target point.
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let dir = target - self.position;
        if dir.length_squared() < 1e-12 {
            return;
        }
        self.rotation = <Quat as QuatExt>::look_at(self.position, target, up);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// 2D transform with position, rotation (z-axis), and scale.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::ONE,
    };

    #[inline]
    pub fn new(position: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    #[inline]
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }

    /// Compute the local-to-world 3x3 matrix as a Mat4 (z = 0 plane).
    #[inline]
    pub fn to_matrix(self) -> Mat4 {
        let (s, c) = self.rotation.sin_cos();
        Mat4::from_cols_array_2d(&[
            [c * self.scale.x, s * self.scale.x, 0.0, 0.0],
            [-s * self.scale.y, c * self.scale.y, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [self.position.x, self.position.y, 0.0, 1.0],
        ])
    }

    /// Lerp between two 2D transforms.
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            rotation: crate::lerp(self.rotation, other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}
