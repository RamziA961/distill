use bevy::prelude::*;
use bevy_app_compute::prelude as compute;

use crate::voxelization::snapshot::SnapshotType;

mod snapshot;
pub mod voxelization_worker;

#[derive(Debug, Clone, Component)]
pub struct VoxelizeMarker;

pub struct VoxelizationPlugin;

impl Plugin for VoxelizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(compute::AppComputePlugin);
        app.add_plugins(compute::AppComputeWorkerPlugin::<
            voxelization_worker::VoxelizationWorker,
        >::default());

        app.insert_resource(SnapshotType::Occupancy);
        app.add_systems(Update, snapshot::snapshotter);
    }
}
