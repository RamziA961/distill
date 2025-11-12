use crate::gpu_types::{GpuBox3, GpuCamera};
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, Clone, Debug, AsBindGroup, TypePath)]
pub struct RaymarchMaterial {
    /// 3D texture with SDF values
    #[texture(0, dimension = "3d")]
    #[sampler(1)]
    pub voxel_texture: Handle<Image>,

    #[uniform(2)]
    pub camera: GpuCamera,

    #[uniform(3)]
    pub grid_size: u32,

    #[uniform(4)]
    pub grid_bounds: GpuBox3,
}

impl Material for RaymarchMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/raymarcher.frag.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
