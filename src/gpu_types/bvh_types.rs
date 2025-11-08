use super::GpuBox3;

#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct GpuBvhNode {
    aabb: GpuBox3,

    /// child index or first triangle index
    entry_index: u32,

    /// 0 = internal node, >0 = leaf
    tri_count: u32,

    /// Padding for 16 byte alignment on the GPU
    _pad0: u32,
    _pad1: u32,
}

impl GpuBvhNode {
    pub fn new(aabb: GpuBox3, entry_index: u32, tri_count: u32) -> Self {
        Self {
            aabb,
            entry_index,
            tri_count,
            _pad0: 0,
            _pad1: 0,
        }
    }

    pub fn with_entry_index(&mut self, entry_index: u32) {
        self.entry_index = entry_index;
    }

    pub fn with_tri_count(&mut self, tri_count: u32) {
        self.tri_count = tri_count;
    }
}
