use super::GpuBox3;

#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GpuBvhNode {
    aabb: GpuBox3,

    entry_index: u32,
    exit_index: u32,
    shape_index: u32,

    /// Padding for 16 byte alignment on the GPU
    _pad: u32,
}

impl GpuBvhNode {
    pub fn new(aabb: GpuBox3, entry_index: u32, exit_index: u32, shape_index: u32) -> Self {
        Self {
            aabb,
            entry_index,
            exit_index,
            shape_index,
            _pad: 0,
        }
    }
}
