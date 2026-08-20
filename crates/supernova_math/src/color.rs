//! Color types — linear RGBA, sRGB helpers, and palette generation.

use bytemuck::{Pod, Zeroable};

/// Linear-space RGBA color. `#[repr(C)]` for GPU upload.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a color from RGBA components.
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const WHITE:   Self = Self::rgb(1.0, 1.0, 1.0);
    pub const BLACK:   Self = Self::rgb(0.0, 0.0, 0.0);
    pub const RED:     Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN:   Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE:    Self = Self::rgb(0.0, 0.0, 1.0);
    pub const YELLOW:  Self = Self::rgb(1.0, 1.0, 0.0);
    pub const CYAN:    Self = Self::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::rgb(1.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create from 0-255 integer components.
    #[inline]
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
    }

    /// Create from 0-255 integer components with alpha.
    #[inline]
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Create from a hex string like "#ff8800" or "ff8800".
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        let val = u32::from_str_radix(hex, 16).ok()?;
        if hex.len() == 6 {
            Some(Self::rgb8(
                ((val >> 16) & 0xff) as u8,
                ((val >> 8) & 0xff) as u8,
                (val & 0xff) as u8,
            ))
        } else if hex.len() == 8 {
            Some(Self::rgba8(
                ((val >> 24) & 0xff) as u8,
                ((val >> 16) & 0xff) as u8,
                ((val >> 8) & 0xff) as u8,
                (val & 0xff) as u8,
            ))
        } else {
            None
        }
    }

    /// Convert to 0-255 u8 array.
    #[inline]
    pub fn to_rgba8(self) -> [u8; 4] {
        [
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
            (self.a * 255.0).round() as u8,
        ]
    }

    /// Linear interpolation between two colors.
    #[inline]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::rgba(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    /// Multiply (tint) two colors.
    #[inline]
    pub fn mul(self, other: Self) -> Self {
        Self::rgba(
            self.r * other.r,
            self.g * other.g,
            self.b * other.b,
            self.a * other.a,
        )
    }

    /// Convert from sRGB to linear.
    pub fn from_srgb(r: f32, g: f32, b: f32, a: f32) -> Self {
        fn to_lin(c: f32) -> f32 {
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        Self::rgba(to_lin(r), to_lin(g), to_lin(b), a)
    }

    /// Convert to sRGB (for display).
    pub fn to_srgb(self) -> [f32; 4] {
        fn to_srgb(c: f32) -> f32 {
            if c <= 0.0031308 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            }
        }
        [to_srgb(self.r), to_srgb(self.g), to_srgb(self.b), self.a]
    }

    /// HSL to RGB color.
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        if s == 0.0 {
            return Self::rgb(l, l, l);
        }
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        let h = h.rem_euclid(360.0) / 360.0;
        fn hue(p: f32, q: f32, t: f32) -> f32 {
            let mut t = t;
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
            if t < 1.0 / 2.0 { return q; }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
            p
        }
        Self::rgb(hue(p, q, h + 1.0 / 3.0), hue(p, q, h), hue(p, q, h - 1.0 / 3.0))
    }

    /// As a float array [r, g, b, a].
    #[inline]
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// As a glam Vec4.
    #[inline]
    pub fn to_vec4(self) -> glam::Vec4 {
        glam::Vec4::new(self.r, self.g, self.b, self.a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl From<[f32; 4]> for Color {
    fn from(v: [f32; 4]) -> Self {
        Self::rgba(v[0], v[1], v[2], v[3])
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        c.to_array()
    }
}
