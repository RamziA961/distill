use bevy::prelude::*;
use bevy_app_compute::prelude as compute;

use crate::voxelization::{raymarch_material::RaymarchMaterial, snapshot::SnapshotType};

pub mod raymarch_material;
mod snapshot;
pub mod voxelization_worker;

#[derive(Debug, Clone, Component)]
pub struct VoxelizeMarker;

#[derive(Debug, Clone, Component)]
pub struct VoxelizedMarker;

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
    }
}
