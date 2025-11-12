use bevy::{
    asset::RenderAssetUsages,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_app_compute::prelude::AppComputeWorker;
use tracing::instrument;

use crate::{
    bvh::BvhData,
    voxelization::{
        SignedDistanceFieldData, VoxelizationData, VoxelizationState, VoxelizeTargetMarker,
        voxelization_worker::{SIZE, VoxelVariables, VoxelizationWorker},
    },
};

#[allow(clippy::type_complexity)]
#[instrument(skip_all)]
pub(super) fn queue_voxelization(
    mut commands: Commands,
    mesh_data: Query<(Entity, &BvhData), (With<VoxelizeTargetMarker>, Without<VoxelizationData>)>,
    mut worker: ResMut<AppComputeWorker<VoxelizationWorker>>,
) {
    if mesh_data.is_empty() {
        trace!("No meshes to voxelize.");
        return;
    }

    info!(
        mesh_count = mesh_data.iter().count(),
        "Starting voxelization queue."
    );

    if let Some((entity, bvh_data)) = mesh_data.iter().next() {
        info!("Uploading mesh {entity:?} to GPU.");
        info!(
            n_triangles = bvh_data.triangles.len(),
            n_bvh_nodes = bvh_data.nodes.len(),
        );
        info!(bvh_root = ?bvh_data.nodes[0]);

        worker.write_slice(VoxelVariables::Triangles.as_ref(), &bvh_data.triangles);
        worker.write_slice(VoxelVariables::BvhNodes.as_ref(), &bvh_data.nodes);
        info!("Starting voxelization for entity {entity:?}.");

        commands.entity(entity).insert(VoxelizationData {
            state: VoxelizationState::InProgress,
            data: None,
        });

        worker.execute();
    }
}

#[instrument(skip_all)]
pub(super) fn extract_voxelization_data(
    mut images: ResMut<Assets<Image>>,
    worker: ResMut<AppComputeWorker<VoxelizationWorker>>,
    mut query: Query<(Entity, &mut VoxelizationData), With<VoxelizeTargetMarker>>,
) {
    if !worker.ready() {
        trace!("Worker is not ready!");
        return;
    }

    if !worker.is_changed() {
        info!("Worker has not changed. Skipping read.");
        return;
    }
    for (entity, mut voxel_data) in query.iter_mut() {
        if voxel_data.state != VoxelizationState::InProgress {
            continue;
        }

        info!("Reading voxelization results for entity {entity:?}.");

        let sdf_buffer = worker
            .read_raw(VoxelVariables::VoxelTexture.as_ref())
            .to_vec();

        let grid_size = SIZE;
        let extent = Extent3d {
            width: grid_size,
            height: grid_size,
            depth_or_array_layers: grid_size,
        };

        // Convert to GPU 3D texture
        let mut image = Image::new(
            extent,
            TextureDimension::D3,
            bytemuck::cast_slice(&sdf_buffer).to_vec(),
            TextureFormat::R32Float, // one channel, 32-bit float
            RenderAssetUsages::RENDER_WORLD,
        );

        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            ..default()
        });

        let handle = images.add(image);

        voxel_data.state = VoxelizationState::Computed;
        voxel_data.data = Some(SignedDistanceFieldData {
            signed_distance_field: handle,
        });
    }
}
