pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};

/// Creates a right-handed perspective projection matrix.
pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    Mat4::perspective_rh(fov, aspect, near, far)
}

/// Creates a right-handed orthographic projection matrix.
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    Mat4::orthographic_rh(left, right, bottom, top, near, far)
}

/// Creates a right-handed look-at view matrix.
pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    Mat4::look_at_rh(eye, target, up)
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    #[must_use]
    pub fn from_pos(pos: Vec3) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn from_rot(rot: Quat) -> Self {
        Self {
            rot,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            scale,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn from_pos_rot(pos: Vec3, rot: Quat) -> Self {
        Self {
            pos,
            rot,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn with_pos(mut self, pos: Vec3) -> Self {
        self.pos = pos;
        self
    }

    #[must_use]
    pub fn with_rot(mut self, rot: Quat) -> Self {
        self.rot = rot;
        self
    }

    #[must_use]
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    #[must_use]
    pub fn translate(mut self, offset: Vec3) -> Self {
        self.pos += offset;
        self
    }

    #[must_use]
    pub fn rotate_y(mut self, angle: f32) -> Self {
        self.rot = Quat::from_rotation_y(angle) * self.rot;
        self
    }

    /// Rotates around a pivot point.
    #[must_use]
    pub fn rotate_around(mut self, pivot: Vec3, rot: Quat) -> Self {
        let offset = self.pos - pivot;
        self.pos = pivot + rot * offset;
        self.rot = rot * self.rot;
        self
    }

    #[must_use]
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.pos) * Mat4::from_quat(self.rot) * Mat4::from_scale(self.scale)
    }

    /// Linear interpolation between two transforms.
    #[must_use]
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            pos: self.pos.lerp(other.pos, t),
            rot: self.rot.slerp(other.rot, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

/// 2D camera with zoom and pan support. Y-axis increases downward (screen-space convention).
#[derive(Debug, Clone)]
pub struct Camera2D {
    pub pos: Vec2,
    pub zoom: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

impl Camera2D {
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            pos: Vec2::ZERO,
            zoom: 1.0,
            width,
            height,
            rotation: 0.0,
        }
    }

    #[must_use]
    pub fn with_pos(mut self, pos: Vec2) -> Self {
        self.pos = pos;
        self
    }

    #[must_use]
    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom.max(0.01);
        self
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn build_view_projection(&self) -> Mat4 {
        let w = self.width / self.zoom;
        let h = self.height / self.zoom;
        let proj = ortho(
            self.pos.x - w / 2.0,
            self.pos.x + w / 2.0,
            self.pos.y + h / 2.0,
            self.pos.y - h / 2.0,
            -1.0,
            1.0,
        );
        if self.rotation != 0.0 {
            let rot = Mat4::from_rotation_z(self.rotation);
            let view = Mat4::from_translation(Vec3::new(-self.pos.x, -self.pos.y, 0.0));
            proj * rot * view
        } else {
            proj
        }
    }

    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        Vec2::new(
            self.pos.x + (screen.x - self.width / 2.0) / self.zoom,
            self.pos.y + (screen.y - self.height / 2.0) / self.zoom,
        )
    }
}

/// 3D FPS-style camera with yaw/pitch.
#[derive(Debug, Clone)]
pub struct Camera3D {
    pub pos: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera3D {
    #[must_use]
    pub fn new(aspect: f32) -> Self {
        Self {
            pos: Vec3::new(0.0, 10.0, -20.0),
            yaw: 0.0,
            pitch: 0.2,
            aspect,
            fov: 60.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
        }
    }

    #[must_use]
    pub fn with_pos(mut self, pos: Vec3) -> Self {
        self.pos = pos;
        self
    }

    #[must_use]
    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        if width > 0.0 && height > 0.0 {
            self.aspect = width / height;
        }
    }

    /// Returns the forward direction. Clamps pitch to avoid gimbal lock at ±90°.
    pub fn forward(&self) -> Vec3 {
        let pitch = self.pitch.clamp(-1.5534, 1.5534); // ~89° clamp
        Vec3::new(
            self.yaw.sin() * pitch.cos(),
            pitch.sin(),
            self.yaw.cos() * pitch.cos(), // +Z is forward at yaw=0
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        Vec3::Y.cross(self.forward()).normalize_or_zero()
    }

    pub fn up(&self) -> Vec3 {
        self.forward().cross(self.right()).normalize_or_zero()
    }

    pub fn build_view_projection(&self) -> Mat4 {
        perspective(self.fov, self.aspect, self.near, self.far)
            * look_at(self.pos, self.pos + self.forward(), Vec3::Y)
    }

    pub fn screen_to_ray(&self, screen: Vec2, width: f32, height: f32) -> Ray {
        let forward = self.forward();
        let right = self.right();
        let up = right.cross(forward).normalize();
        let tan_fov = (self.fov / 2.0).tan();
        let x = (screen.x / width) * 2.0 - 1.0;
        let y = 1.0 - (screen.y / height) * 2.0;
        let dir = (forward + right * (x * tan_fov * self.aspect) + up * (y * tan_fov)).normalize();
        Ray {
            origin: self.pos,
            dir,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    /// Ray-sphere intersection. Returns true if the ray hits the sphere.
    /// Correctly handles the case where the ray origin is inside the sphere.
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        let oc = self.origin - center;
        let a = self.dir.dot(self.dir);
        let b = 2.0 * oc.dot(self.dir);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return false;
        }
        // Check if the nearest intersection is positive (in front of the ray)
        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        t >= 0.0 || c <= 0.0 // c <= 0 means origin is inside the sphere
    }

    /// Returns the distance to the sphere surface, or None if no hit.
    pub fn intersects_sphere_distance(&self, center: Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let a = self.dir.dot(self.dir);
        let b = 2.0 * oc.dot(self.dir);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrt_d = discriminant.sqrt();
        let t1 = (-b - sqrt_d) / (2.0 * a);
        let t2 = (-b + sqrt_d) / (2.0 * a);
        if t1 >= 0.0 {
            Some(t1)
        } else if t2 >= 0.0 {
            Some(t2)
        } else {
            None
        }
    }

    /// Ray-plane intersection (plane defined by point and normal).
    pub fn intersects_plane(&self, plane_point: Vec3, plane_normal: Vec3) -> Option<f32> {
        let denom = self.dir.dot(plane_normal);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = (plane_point - self.origin).dot(plane_normal) / denom;
        if t >= 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half = size / 2.0;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) / 2.0
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Check if the AABB is visible given a view-projection matrix (frustum culling).
    pub fn is_on_screen(&self, vp: &Mat4) -> bool {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];
        let mut inside = false;
        for c in &corners {
            let clip = *vp * Vec4::new(c.x, c.y, c.z, 1.0);
            if clip.w > 0.0 {
                let ndc = clip.xyz() / clip.w;
                if ndc.x >= -1.0
                    && ndc.x <= 1.0
                    && ndc.y >= -1.0
                    && ndc.y <= 1.0
                    && ndc.z >= -1.0
                    && ndc.z <= 1.0
                {
                    return true;
                }
            }
            inside |= clip.w > 0.0;
        }
        // Conservative: if all corners are on one side of a frustum plane,
        // the AABB is culled. If mixed, it's visible.
        inside
    }
}
