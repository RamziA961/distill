use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
};

use crate::gpu_types::{GpuBox3, GpuCamera, GpuVec2};

#[derive(Asset, Clone, Debug, AsBindGroup, TypePath)]
pub struct RaymarchMaterial {
    #[storage(0, read_only)]
    pub voxel_texture: Handle<ShaderStorageBuffer>,

    #[uniform(1)]
    pub camera: GpuCamera,

    #[uniform(2)]
    pub grid_size: u32,

    #[uniform(3)]
    pub grid_bounds: GpuBox3,

    #[uniform(4)]
    pub screen_resolution: GpuVec2,
}

impl Material for RaymarchMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/raymarcher.frag.wgsl".into()
    }
}

