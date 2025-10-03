use crate::gpu_types::{GpuBox3, GpuCamera};
use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
};

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
}

impl Material for RaymarchMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/raymarcher.frag.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        //TODO: Change to AlphaMode::Blend
        AlphaMode::Opaque
    }
}
