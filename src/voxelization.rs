use bevy::prelude::*;
use bevy_app_compute::prelude as compute;

use crate::voxelization::{raymarch_material::RaymarchMaterial, snapshot::SnapshotType};

mod raymarch;
pub mod raymarch_material;
mod raymarch_systems;
mod snapshot;
mod voxelization_systems;
pub mod voxelization_worker;

pub struct VoxelizationPlugin;

impl Plugin for VoxelizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(compute::AppComputePlugin);
        app.add_plugins(compute::AppComputeWorkerPlugin::<
            voxelization_worker::VoxelizationWorker,
        >::default());

        app.add_plugins(MaterialPlugin::<RaymarchMaterial>::default());

        app.insert_resource(SnapshotType::SignedDistance);
        app.add_systems(Update, snapshot::snapshotter);

        app.add_systems(
            Update,
            (
                voxelization_systems::queue_voxelization,
                voxelization_systems::extract_voxelization_data,
                //raymarch_systems::spawn_raymarch_render_targets,
                //raymarch_systems::update_raymarch_materials,
            )
                .chain(),
        );
    }
}

#[derive(Debug, Clone, Component)]
pub struct VoxelizeMarker;

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
