use bevy::{math::bounding::Aabb3d, prelude::*, render::primitives::Aabb};
use bevy_app_compute::prelude::ShaderType;
use bytemuck::{Pod, Zeroable};

use super::vector_types::GpuVec3;

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

impl From<Aabb> for GpuBox3 {
    fn from(value: Aabb) -> Self {
        Self {
            min: value.min().into(),
            max: value.max().into(),
        }
    }
}

impl From<GpuBox3> for Aabb {
    fn from(value: GpuBox3) -> Self {
        let min = Vec3::from(*value.min());
        let max = Vec3::from(*value.max());
        Self::from_min_max(min, max)
    }
}
