use crate::gpu_types::{GpuBox3, GpuCamera};
use bevy::{
    pbr::MaterialExtension,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, Clone, Debug, AsBindGroup, TypePath)]
pub struct RaymarchMaterialExtension {
    /// 3D texture with SDF values
    #[texture(100, dimension = "3d")]
    #[sampler(101)]
    pub voxel_texture: Handle<Image>,

    #[uniform(102)]
    pub camera: GpuCamera,

    #[uniform(103)]
    pub grid_size: u32,

    #[uniform(104)]
    pub grid_bounds: GpuBox3,

    #[uniform(105)]
    pub local_from_world: Mat4,

    #[uniform(106)]
    pub world_from_local: Mat4,
}

impl MaterialExtension for RaymarchMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/raymarcher.frag.wgsl".into()
    }
}
