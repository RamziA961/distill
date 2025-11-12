use crate::{
    camera::marker::CameraMarkerPrimary,
    gpu_types::{GpuBox3, GpuCamera},
    voxelization::{
        VoxelizationData, VoxelizationState, VoxelizeTargetMarker, raymarch::RaymarchRenderTarget,
        raymarch_material::RaymarchMaterialExtension, voxelization_worker::SIZE,
    },
};
use bevy::{pbr::ExtendedMaterial, prelude::*, render::mesh::MeshAabb};

pub(super) fn spawn_raymarch_render_targets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, RaymarchMaterialExtension>>>,
    voxel_query: Query<(Entity, &Mesh3d, &VoxelizationData), With<VoxelizeTargetMarker>>,
    camera_params: Single<(&Transform, &Projection), With<CameraMarkerPrimary>>,
    existing_targets: Query<&RaymarchRenderTarget>,
) {
    if voxel_query.is_empty() {
        return;
    }

    let (camera_transform, projection) = camera_params.into_inner();
    let camera = GpuCamera::from_transform_and_projection(camera_transform, projection);

    for (entity, mesh_handle, voxel_data) in voxel_query.iter() {
        // Only spawn for meshes that have finished voxelization
        if voxel_data.state != VoxelizationState::Computed {
            continue;
        }

        // Skip if we already spawned a render target for this mesh
        if existing_targets.iter().any(|t| t.source_entity == entity) {
            continue;
        }

        let mesh = meshes.get(mesh_handle).unwrap();
        let grid_bounds = mesh.compute_aabb().map(GpuBox3::from).unwrap();
        let grid_size = SIZE;

        let Some(voxel_info) = &voxel_data.data else {
            error!(
                "VoxelizationData for entity {:?} is in Computed state but has no SignedDistanceFieldData!",
                entity
            );
            continue;
        };

        let sdf_handle = voxel_info.signed_distance_field.clone();

        // Spawn a separate render target entity
        commands.spawn((
            RaymarchRenderTarget {
                source_entity: entity,
            },
            Mesh3d(meshes.add(Cuboid::from_size(grid_bounds.size().into()))),
            //MeshMaterial3d(materials.add(RaymarchMaterial {
            //    voxel_texture: sdf_handle,
            //    camera,
            //    grid_bounds,
            //    grid_size,
            //})),
            MeshMaterial3d(materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Color::linear_rgba(1.0, 0.0, 0.0, 1.0),
                    alpha_mode: AlphaMode::Blend,
                    unlit: false,
                    ..Default::default()
                },
                extension: RaymarchMaterialExtension {
                    voxel_texture: sdf_handle,
                    camera,
                    grid_size,
                    grid_bounds,
                },
            })),
            Transform::from_translation(grid_bounds.center().into()),
        ));
    }
}

pub(super) fn update_raymarch_materials(
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, RaymarchMaterialExtension>>>,
    targets: Query<(
        &RaymarchRenderTarget,
        &MeshMaterial3d<ExtendedMaterial<StandardMaterial, RaymarchMaterialExtension>>,
    )>,
    _voxel_sources: Query<&VoxelizationData, With<VoxelizeTargetMarker>>,
    camera_params: Single<(&Transform, &Projection), With<CameraMarkerPrimary>>,
) {
    let (camera_transform, projection) = camera_params.into_inner();
    let camera = GpuCamera::from_transform_and_projection(camera_transform, projection);

    for (_, material_handle) in targets.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.extension.camera = camera;

            //if let Ok(voxel_data) = voxel_sources.get(target.source_entity) {
            //    if let Some(sdf_data) = &voxel_data.data {
            //        material.voxel_texture = sdf_data.signed_distance_field.clone();
            //    }
            //}
        }
    }
}
