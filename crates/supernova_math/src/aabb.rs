//! Axis-Aligned Bounding Box — 2D and 3D variants.

use crate::vec::{Vec2Ext, Vec3Ext};
use glam::{Vec2, Vec3};

/// 2D AABB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb2 {
    #[inline]
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_point(p: Vec2) -> Self {
        Self { min: p, max: p }
    }

    #[inline]
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    #[inline]
    pub fn center(self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn size(self) -> Vec2 {
        self.max - self.min
    }

    #[inline]
    pub fn half_size(self) -> Vec2 {
        self.size() * 0.5
    }

    #[inline]
    pub fn contains(self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    #[inline]
    pub fn intersects(self, other: Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    #[inline]
    pub fn expanded(self, p: Vec2) -> Self {
        Self {
            min: self.min.min_component(p),
            max: self.max.max_component(p),
        }
    }

    #[inline]
    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min_component(other.min),
            max: self.max.max_component(other.max),
        }
    }

    #[inline]
    pub fn area(self) -> f32 {
        let s = self.size();
        s.x * s.y
    }

    #[inline]
    pub fn translate(self, by: Vec2) -> Self {
        Self {
            min: self.min + by,
            max: self.max + by,
        }
    }
}

/// 3D AABB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb3 {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb3 {
    #[inline]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_point(p: Vec3) -> Self {
        Self { min: p, max: p }
    }

    #[inline]
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    #[inline]
    pub fn center(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    #[inline]
    pub fn size(self) -> Vec3 {
        self.max - self.min
    }

    #[inline]
    pub fn half_size(self) -> Vec3 {
        self.size() * 0.5
    }

    #[inline]
    pub fn contains(self, p: Vec3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    #[inline]
    pub fn intersects(self, other: Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    #[inline]
    pub fn expanded(self, p: Vec3) -> Self {
        Self {
            min: self.min.min_component(p),
            max: self.max.max_component(p),
        }
    }

    #[inline]
    pub fn union(self, other: Self) -> Self {
        Self {
            min: self.min.min_component(other.min),
            max: self.max.max_component(other.max),
        }
    }

    #[inline]
    pub fn volume(self) -> f32 {
        let s = self.size();
        s.x * s.y * s.z
    }

    #[inline]
    pub fn translate(self, by: Vec3) -> Self {
        Self {
            min: self.min + by,
            max: self.max + by,
        }
    }

    /// Closest point on the AABB to `p`.
    pub fn closest_point(self, p: Vec3) -> Vec3 {
        Vec3::new(
            p.x.clamp(self.min.x, self.max.x),
            p.y.clamp(self.min.y, self.max.y),
            p.z.clamp(self.min.z, self.max.z),
        )
    }

    /// Signed distance to a point (negative inside).
    pub fn distance_to_point(self, p: Vec3) -> f32 {
        let closest = self.closest_point(p);
        (p - closest).length()
    }
}

impl Default for Aabb2 {
    fn default() -> Self {
        Self::new(Vec2::ZERO, Vec2::ZERO)
    }
}

impl Default for Aabb3 {
    fn default() -> Self {
        Self::new(Vec3::ZERO, Vec3::ZERO)
    }
}
