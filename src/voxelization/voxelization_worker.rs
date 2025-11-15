#![allow(dead_code)]
use bevy::prelude::*;
use bevy_app_compute::prelude::{
    AppComputeWorker, AppComputeWorkerBuilder, ComputeShader, ComputeWorker, ShaderRef, ShaderType,
};
use bytemuck::{Pod, Zeroable};

use crate::gpu_types::{GpuBvhNode, GpuTriangle};

pub const SIZE: u32 = 128;
const WORKGROUP_SIZE: u32 = 8;

#[derive(Debug, strum::EnumString, strum::Display, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum VoxelVariables {
    VoxelTexture,
    Triangles,
    BvhNodes,
    VoxelUniforms,
}

#[derive(Clone, Copy, Zeroable, Pod, ShaderType)]
#[repr(C)]
pub struct VoxelUniforms {
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

        let voxel_uniforms = VoxelUniforms { size: SIZE };

        AppComputeWorkerBuilder::new(world)
            .add_empty_staging(
                VoxelVariables::VoxelTexture.as_ref(),
                (SIZE as u64).pow(3) * 4,
            )
            .add_empty_rw_storage(
                VoxelVariables::Triangles.as_ref(),
                4212 * std::mem::size_of::<GpuTriangle>() as u64,
            )
            .add_empty_rw_storage(
                VoxelVariables::BvhNodes.as_ref(),
                4096 * std::mem::size_of::<GpuBvhNode>() as u64,
            )
            .add_uniform(VoxelVariables::VoxelUniforms.as_ref(), &voxel_uniforms)
            .add_pass::<VoxelizationShader>(
                workgroups,
                &[
                    VoxelVariables::VoxelTexture.as_ref(),
                    VoxelVariables::Triangles.as_ref(),
                    VoxelVariables::BvhNodes.as_ref(),
                    VoxelVariables::VoxelUniforms.as_ref(),
                ],
            )
            .one_shot()
            .build()
    }
}
