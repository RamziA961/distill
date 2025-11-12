use crate::voxelization::{raymarch_material::RaymarchMaterialExtension, snapshot::SnapshotType};
use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_app_compute::prelude as compute;

mod raymarch;
pub mod raymarch_material;
mod raymarch_systems;
mod snapshot;
mod voxelization_systems;
pub mod voxelization_worker;

pub struct VoxelizationPlugin;

impl Plugin for VoxelizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            compute::AppComputePlugin,
            compute::AppComputeWorkerPlugin::<voxelization_worker::VoxelizationWorker>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, RaymarchMaterialExtension>>::default()
        ));

        app.add_systems(
            Update,
            (
                voxelization_systems::extract_voxelization_data,
                voxelization_systems::queue_voxelization,
                raymarch_systems::spawn_raymarch_render_targets,
                raymarch_systems::update_raymarch_materials,
            )
                .chain(),
        );

        app.insert_resource(SnapshotType::SignedDistance);
        app.add_systems(Update, snapshot::snapshotter);
    }
}

#[derive(Debug, Clone, Component)]
pub struct VoxelizeTargetMarker;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum VoxelizationState {
    #[default]
    InProgress,
    Computed,
}

#[derive(Debug, Clone)]
pub struct SignedDistanceFieldData {
    pub signed_distance_field: Handle<Image>,
}

#[derive(Debug, Clone, Component)]
pub struct VoxelizationData {
    state: VoxelizationState,
    data: Option<SignedDistanceFieldData>,
}
