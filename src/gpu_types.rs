use bevy::math::{Vec3, Vec4};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GpuVec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[allow(dead_code)]
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

#[derive(Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct GpuVec3 {
    inner: GpuVec4,
}

#[allow(dead_code)]
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

impl std::fmt::Debug for GpuVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}, {}, {} }}", self.x(), self.y(), self.z())
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GpuUVec4 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub w: u32,
}

#[allow(dead_code)]
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

impl std::fmt::Debug for GpuUVec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}, {}, {}, {} }}", self.x, self.y, self.z, self.w)
    }
}

/// GPU-aligned Vec3 wrapper using GpuU32Vec4 internally
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct GpuUVec3 {
    inner: GpuUVec4,
}

#[allow(dead_code)]
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

impl std::fmt::Debug for GpuUVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {}, {}, {} }}", self.x(), self.y(), self.z())
    }
}
