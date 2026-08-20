//! Vector utilities and extensions.

use glam::{Vec2, Vec3};

/// 2D vector extension methods.
pub trait Vec2Ext {
    /// Rotate this vector by `angle` radians.
    fn rotated(self, angle: f32) -> Vec2;
    /// Angle to another vector in radians.
    fn angle_to(self, other: Vec2) -> f32;
    /// Perpendicular vector (90° counter-clockwise).
    fn perp(self) -> Vec2;
    /// Signed area of the parallelogram formed by `self` and `other`.
    fn cross(self, other: Vec2) -> f32;
    /// Linear interpolation.
    fn lerp(self, other: Vec2, t: f32) -> Vec2;
    /// Component-wise min.
    fn min_component(self, other: Vec2) -> Vec2;
    /// Component-wise max.
    fn max_component(self, other: Vec2) -> Vec2;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn rotated(self, angle: f32) -> Vec2 {
        let (s, c) = angle.sin_cos();
        Vec2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }

    #[inline]
    fn angle_to(self, other: Vec2) -> f32 {
        other.y.atan2(other.x) - self.y.atan2(self.x)
    }

    #[inline]
    fn perp(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }

    #[inline]
    fn cross(self, other: Vec2) -> f32 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    fn lerp(self, other: Vec2, t: f32) -> Vec2 {
        self + (other - self) * t
    }

    #[inline]
    fn min_component(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[inline]
    fn max_component(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x.max(other.x), self.y.max(other.y))
    }
}

/// 3D vector extension methods.
pub trait Vec3Ext {
    /// Linear interpolation.
    fn lerp(self, other: Vec3, t: f32) -> Vec3;
    /// Component-wise min.
    fn min_component(self, other: Vec3) -> Vec3;
    /// Component-wise max.
    fn max_component(self, other: Vec3) -> Vec3;
    /// Returns true if all components are finite.
    fn is_finite(self) -> bool;
    /// Angle between two vectors in radians.
    fn angle_between(self, other: Vec3) -> f32;
}

impl Vec3Ext for Vec3 {
    #[inline]
    fn lerp(self, other: Vec3, t: f32) -> Vec3 {
        self + (other - self) * t
    }

    #[inline]
    fn min_component(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        )
    }

    #[inline]
    fn max_component(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        )
    }

    #[inline]
    fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    #[inline]
    fn angle_between(self, other: Vec3) -> f32 {
        let dot = self.dot(other);
        let mag = self.length() * other.length();
        if mag < 1e-12 {
            return 0.0;
        }
        (dot / mag).clamp(-1.0, 1.0).acos()
    }
}
