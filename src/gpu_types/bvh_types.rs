use super::GpuBox3;
use bevy_app_compute::prelude::ShaderType;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug, Pod, Zeroable, ShaderType)]
#[repr(C)]
pub struct GpuBvhNode {
    aabb: GpuBox3,

    left_index: u32,
    right_index: u32,
    /// 0 = internal node, >0 = leaf
    triangle_count: u32,

    /// Padding for 16 byte alignment on the GPU
    _pad0: u32,
}

impl GpuBvhNode {
    pub fn new(aabb: GpuBox3, left_index: u32, right_index: u32, triangle_count: u32) -> Self {
        Self {
            aabb,
            left_index,
            right_index,
            triangle_count,
            _pad0: 0,
        }
    }

    pub fn aabb(&self) -> GpuBox3 {
        self.aabb
    }

    pub fn with_left_index(&mut self, left_index: u32) {
        self.left_index = left_index;
    }

    pub fn with_right_index(&mut self, right_index: u32) {
        self.right_index = right_index;
    }

    pub fn with_triangle_count(&mut self, triangle_count: u32) {
        self.triangle_count = triangle_count;
    }
}
