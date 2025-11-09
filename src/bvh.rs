use crate::gpu_types::{GpuBvhNode, GpuTriangle};
use bevy::prelude::*;

mod bevy_mesh_integration;
mod bvh_builder;

pub struct BvhPlugin;

impl Plugin for BvhPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, bvh_system);
    }
}

#[derive(Component, Debug)]
pub struct BvhData {
    pub nodes: Vec<GpuBvhNode>,
    pub triangles: Vec<GpuTriangle>,
}

fn bvh_system(
    mut commands: Commands,
    mesh_handles: Query<(Entity, &Mesh3d), Without<BvhData>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, mesh_handle) in mesh_handles.iter() {
        let mesh = if let Some(mesh) = meshes.get(mesh_handle) {
            mesh
        } else {
            continue;
        };

        let (nodes, triangles) = mesh.build_bvh(4);
        commands.entity(entity).insert(BvhData { nodes, triangles });
        info!("BVH computed for entity {:?}", entity);
    }
}

pub trait MeshBvh {
    fn build_bvh(&self, leaf_size: usize) -> (Vec<GpuBvhNode>, Vec<GpuTriangle>);
}
