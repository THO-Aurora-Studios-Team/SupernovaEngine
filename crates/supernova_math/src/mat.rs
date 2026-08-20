//! Matrix utilities and extensions.

use glam::{Mat3, Mat4, Quat, Vec3};

/// Matrix extension methods.
pub trait Mat4Ext {
    /// Create a look-at view matrix (right-handed).
    fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4;
    /// Create an orthographic projection matrix.
    fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4;
    /// Create a perspective projection matrix (right-handed, depth 0..1).
    fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4;
    /// Decompose into translation, rotation (quat), and scale.
    fn decompose(self) -> (Vec3, Quat, Vec3);
    /// Get translation component.
    fn translation(self) -> Vec3;
    /// Get forward vector (-Z).
    fn forward(self) -> Vec3;
    /// Get right vector (+X).
    fn right(self) -> Vec3;
    /// Get up vector (+Y).
    fn up(self) -> Vec3;
}

impl Mat4Ext for Mat4 {
    #[inline]
    fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        Mat4::look_at_rh(eye, target, up)
    }

    #[inline]
    fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        Mat4::orthographic_rh(left, right, bottom, top, near, far)
    }

    #[inline]
    fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        Mat4::perspective_rh(fov_y, aspect, near, far)
    }

    #[inline]
    fn decompose(self) -> (Vec3, Quat, Vec3) {
        let scale = Vec3::new(
            self.x_axis.length(),
            self.y_axis.length(),
            self.z_axis.length(),
        );
        let translation = self.w_axis.truncate();
        let rot_mat = Mat3::from_mat4(self)
            * Mat3::from_diagonal(Vec3::new(1.0 / scale.x, 1.0 / scale.y, 1.0 / scale.z));
        let rotation = Quat::from_mat3(&rot_mat);
        (translation, rotation, scale)
    }

    #[inline]
    fn translation(self) -> Vec3 {
        self.w_axis.truncate()
    }

    #[inline]
    fn forward(self) -> Vec3 {
        -self.z_axis.truncate()
    }

    #[inline]
    fn right(self) -> Vec3 {
        self.x_axis.truncate()
    }

    #[inline]
    fn up(self) -> Vec3 {
        self.y_axis.truncate()
    }
}
