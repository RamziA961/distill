use bevy::{pbr::ExtendedMaterial, prelude::*};
use bevy_app_compute::prelude as compute;
use raymarch_material::RaymarchMaterialExtension;
use signed_distance_field_data::SignedDistanceFieldData;
use snapshot::SnapshotType;

mod raymarch;
pub mod raymarch_material;
mod raymarch_systems;
pub mod signed_distance_field_data;
mod snapshot;
mod voxelizer_systems;
pub mod voxelizer_worker;

pub const SIZE: u32 = 128;

pub struct VoxelizationPlugin;

impl Plugin for VoxelizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            compute::AppComputePlugin,
            compute::AppComputeWorkerPlugin::<voxelizer_worker::VoxelizationWorker>::default(),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, RaymarchMaterialExtension>>::default()
        ));

        app.add_systems(
            Update,
            (
                voxelizer_systems::extract_voxelization_data,
                voxelizer_systems::queue_voxelization,
                raymarch_systems::spawn_raymarch_render_targets,
                raymarch_systems::update_raymarch_materials,
            )
                .chain(),
        );

        app.init_state::<SnapshotType>();
        app.add_systems(
            Update,
            (snapshot::snapshotter, snapshot::cycle_snapshot_type),
        );
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

#[derive(Debug, Clone, Component)]
pub struct VoxelizationData {
    pub state: VoxelizationState,
    pub data: Option<SignedDistanceFieldData>,
}
