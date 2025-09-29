use bevy::prelude::*;
use bevy_app_compute::prelude::{
    AppComputeWorker, AppComputeWorkerBuilder, ComputeShader, ComputeWorker, ShaderRef, ShaderType,
};
use strum::{AsRefStr, EnumString};

pub const SIZE: u32 = 64;
const WORKGROUP_SIZE: u32 = 8;

#[derive(Debug, EnumString, AsRefStr)]
pub enum VoxelVariables {
    #[strum(serialize = "voxel_texture")]
    VoxelTexture,
    #[strum(serialize = "vertices")]
    Vertices,
    #[strum(serialize = "triangles")]
    Triangles,
    #[strum(serialize = "normals")]
    Normals,
    #[strum(serialize = "voxel_uniforms")]
    VoxelUniforms,
}

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, ShaderType)]
#[repr(C)]
pub struct VoxelUniforms {
    scale: Vec3,
    minimum: Vec3,
    size: u32,
}

#[derive(Default, TypePath)]
pub struct VoxelizationShader;

impl ComputeShader for VoxelizationShader {
    fn shader() -> ShaderRef {
        "shaders/voxelizer.compute.wgsl".into()
    }
}

#[derive(Resource)]
pub struct VoxelizationWorker;

impl ComputeWorker for VoxelizationWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let workgroups = [SIZE.div_ceil(WORKGROUP_SIZE); 3];
        info!(workgroups = ?workgroups);

        let voxel_uniforms = VoxelUniforms {
            scale: Vec3::splat(2.0),
            minimum: Vec3::splat(-1.0),
            size: SIZE,
        };

        AppComputeWorkerBuilder::new(world)
            .add_empty_staging(
                VoxelVariables::VoxelTexture.as_ref(),
                (SIZE * SIZE * SIZE * 4) as u64,
            )
            .add_empty_rw_storage(VoxelVariables::Vertices.as_ref(), 362 * 16)
            .add_empty_rw_storage(VoxelVariables::Triangles.as_ref(), 720 * 16)
            .add_uniform(VoxelVariables::VoxelUniforms.as_ref(), &voxel_uniforms)
            .add_pass::<VoxelizationShader>(
                workgroups,
                &[
                    VoxelVariables::VoxelTexture.as_ref(),
                    VoxelVariables::Vertices.as_ref(),
                    VoxelVariables::Triangles.as_ref(),
                    VoxelVariables::Normals.as_ref(),
                    VoxelVariables::VoxelUniforms.as_ref(),
                ],
            )
            .build()
    }
}
