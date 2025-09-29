#![allow(dead_code)]

use bevy::{
    math::{Dir3, UVec3, UVec4, Vec2, Vec3, Vec3A, Vec4, bounding::Aabb3d},
    transform::components::Transform,
};
use bevy_app_compute::prelude::ShaderType;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct GpuVec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl GpuVec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn z(&self) -> f32 {
        self.z
    }

    pub fn w(&self) -> f32 {
        self.w
    }

    pub fn from_slice(slice: &[f32]) -> Self {
        assert!(slice.len() >= 4);
        Self::new(slice[0], slice[1], slice[2], slice[3])
    }
}

impl std::fmt::Debug for GpuVec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}, {}, {}, {} }}", self.x, self.y, self.z, self.w)
    }
}

impl From<Vec4> for GpuVec4 {
    fn from(value: Vec4) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

#[derive(Clone, Copy, Zeroable, Pod, ShaderType)]
#[repr(C)]
pub struct GpuVec3 {
    inner: GpuVec4,
}

impl GpuVec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: GpuVec4::new(x, y, z, 0.0),
        }
    }

    pub fn x(&self) -> f32 {
        self.inner.x
    }

    pub fn y(&self) -> f32 {
        self.inner.y
    }

    pub fn z(&self) -> f32 {
        self.inner.z
    }

    pub fn from_slice(slice: &[f32]) -> Self {
        assert!(slice.len() >= 3);
        Self::new(slice[0], slice[1], slice[2])
    }
}

impl From<Vec3> for GpuVec3 {
    fn from(value: Vec3) -> Self {
        Self {
            inner: GpuVec4::new(value.x, value.y, value.z, 0.0),
        }
    }
}

impl From<Vec3A> for GpuVec3 {
    fn from(value: Vec3A) -> Self {
        Self {
            inner: GpuVec4::new(value.x, value.y, value.z, 0.0),
        }
    }
}

impl From<Dir3> for GpuVec3 {
    fn from(value: Dir3) -> Self {
        Self {
            inner: GpuVec4::new(value.x, value.y, value.z, 0.0),
        }
    }
}

impl std::fmt::Debug for GpuVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}, {}, {} }}", self.x(), self.y(), self.z())
    }
}

#[derive(Clone, Copy, Zeroable, Pod, ShaderType)]
#[repr(C)]
pub struct GpuVec2 {
    inner: GpuVec4,
}

impl GpuVec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            inner: GpuVec4::new(x, y, 0.0, 0.0),
        }
    }

    pub fn x(&self) -> f32 {
        self.inner.x
    }

    pub fn y(&self) -> f32 {
        self.inner.y
    }

    pub fn from_slice(slice: &[f32]) -> Self {
        assert!(slice.len() >= 2);
        Self::new(slice[0], slice[1])
    }
}

impl From<Vec2> for GpuVec2 {
    fn from(value: Vec2) -> Self {
        Self {
            inner: GpuVec4::new(value.x, value.y, 0.0, 0.0),
        }
    }
}

impl std::fmt::Debug for GpuVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}, {} }}", self.x(), self.y())
    }
}

#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct GpuUVec4 {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

impl GpuUVec4 {
    pub fn new(x: u32, y: u32, z: u32, w: u32) -> Self {
        Self { x, y, z, w }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn z(&self) -> u32 {
        self.z
    }

    pub fn w(&self) -> u32 {
        self.w
    }

    pub fn from_slice(slice: &[u32]) -> Self {
        assert!(slice.len() >= 4);
        Self::new(slice[0], slice[1], slice[2], slice[3])
    }
}

impl From<UVec4> for GpuUVec4 {
    fn from(value: UVec4) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl std::fmt::Debug for GpuUVec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}, {}, {}, {} }}", self.x, self.y, self.z, self.w)
    }
}

/// GPU-aligned Vec3 wrapper using GpuUVec4 internally
#[derive(Clone, Copy, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct GpuUVec3 {
    inner: GpuUVec4,
}

impl GpuUVec3 {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self {
            inner: GpuUVec4::new(x, y, z, 0),
        } // w as padding
    }

    pub fn x(&self) -> u32 {
        self.inner.x
    }

    pub fn y(&self) -> u32 {
        self.inner.y
    }

    pub fn z(&self) -> u32 {
        self.inner.z
    }

    pub fn from_slice(slice: &[u32]) -> Self {
        assert!(slice.len() >= 3);
        Self::new(slice[0], slice[1], slice[2])
    }
}

impl From<UVec3> for GpuUVec3 {
    fn from(value: UVec3) -> Self {
        Self {
            inner: GpuUVec4::new(value.x, value.y, value.z, 0),
        }
    }
}

impl std::fmt::Debug for GpuUVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}, {}, {} }}", self.x(), self.y(), self.z())
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Debug, ShaderType)]
#[repr(C)]
pub struct GpuBox3 {
    min: GpuVec3,
    max: GpuVec3,
}

impl GpuBox3 {
    /// Create a new Box3 from min and max points
    pub fn new(min: GpuVec3, max: GpuVec3) -> Self {
        Self { min, max }
    }

    pub fn min(&self) -> &GpuVec3 {
        &self.min
    }

    pub fn max(&self) -> &GpuVec3 {
        &self.max
    }

    /// Compute the size/extent of the box (max - min)
    pub fn size(&self) -> GpuVec3 {
        GpuVec3::new(
            self.max.x() - self.min.x(),
            self.max.y() - self.min.y(),
            self.max.z() - self.min.z(),
        )
    }

    /// Compute the center point of the box
    pub fn center(&self) -> GpuVec3 {
        let s = self.size();
        GpuVec3::new(
            self.min.x() + s.x() * 0.5,
            self.min.y() + s.y() * 0.5,
            self.min.z() + s.z() * 0.5,
        )
    }
}

impl From<Aabb3d> for GpuBox3 {
    fn from(value: Aabb3d) -> Self {
        Self {
            min: value.min.into(),
            max: value.max.into(),
        }
    }
}

#[derive(Clone, Copy, Pod, Zeroable, Debug, ShaderType)]
#[repr(C)]
pub struct GpuCamera {
    position: GpuVec3,
    forward: GpuVec3,
    right: GpuVec3,
    up: GpuVec3,
}

impl GpuCamera {
    pub fn new(position: GpuVec3, forward: GpuVec3, right: GpuVec3, up: GpuVec3) -> Self {
        Self {
            position,
            forward,
            right,
            up,
        }
    }

    pub fn position(&self) -> &GpuVec3 {
        &self.position
    }

    pub fn forward(&self) -> &GpuVec3 {
        &self.forward
    }

    pub fn right(&self) -> &GpuVec3 {
        &self.right
    }

    pub fn up(&self) -> &GpuVec3 {
        &self.up
    }
}

impl From<&Transform> for GpuCamera {
    fn from(transform: &Transform) -> Self {
        // Position of the camera
        let position = GpuVec3::from(transform.translation);

        // Orientation vectors
        let forward = GpuVec3::from(transform.forward());
        let right = GpuVec3::from(transform.right());
        let up = GpuVec3::from(transform.up());

        GpuCamera::new(position, forward, right, up)
    }
}
