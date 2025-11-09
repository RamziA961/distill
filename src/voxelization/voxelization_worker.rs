#![allow(dead_code)]

use crate::gpu_types::GpuBox3;
use bevy::{math::bounding::Aabb3d, prelude::*};
use bevy_app_compute::prelude::{
    AppComputeWorker, AppComputeWorkerBuilder, ComputeShader, ComputeWorker, ShaderRef, ShaderType,
};
use bytemuck::{Pod, Zeroable};
use strum::{AsRefStr, EnumString};

pub const SIZE: u32 = 64;
const WORKGROUP_SIZE: u32 = 8;

#[derive(Debug, EnumString, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum VoxelVariables {
    VoxelTexture,
    Triangles,
    BvhNodes,
    VoxelUniforms,
    MeshUniforms,
}

#[derive(Clone, Copy, Zeroable, Pod, ShaderType)]
#[repr(C)]
pub struct VoxelUniforms {
    size: u32,
}

#[derive(Clone, Copy, Zeroable, Pod, ShaderType)]
#[repr(C)]
pub struct MeshUniforms {
    aabb: GpuBox3,
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

        let voxel_uniforms = VoxelUniforms { size: SIZE };
        let mesh_uniforms = MeshUniforms {
            aabb: Aabb3d::new(Vec3::splat(0.0), Vec3::splat(1.0)).into(),
        };

        AppComputeWorkerBuilder::new(world)
            .add_empty_staging(
                VoxelVariables::VoxelTexture.as_ref(),
                (SIZE * SIZE * SIZE * 4) as u64,
            )
            .add_empty_rw_storage(VoxelVariables::Triangles.as_ref(), 1536 * 48)
            .add_empty_rw_storage(VoxelVariables::BvhNodes.as_ref(), 4096 * 32)
            .add_uniform(VoxelVariables::VoxelUniforms.as_ref(), &voxel_uniforms)
            .add_uniform(VoxelVariables::MeshUniforms.as_ref(), &mesh_uniforms)
            .add_pass::<VoxelizationShader>(
                workgroups,
                &[
                    VoxelVariables::VoxelTexture.as_ref(),
                    VoxelVariables::Triangles.as_ref(),
                    VoxelVariables::BvhNodes.as_ref(),
                    VoxelVariables::VoxelUniforms.as_ref(),
                    VoxelVariables::MeshUniforms.as_ref(),
                ],
            )
            .build()
    }
}
