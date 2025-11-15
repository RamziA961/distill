use crate::voxelization::{VoxelizationData, VoxelizationState, voxelization_worker::SIZE};
use bevy::prelude::*;
use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage};
use std::path::Path;

const TEMP_DIR: &str = "temp";

#[derive(Resource, Clone, Debug, Default, strum::EnumString, strum::AsRefStr, strum::Display)]
#[allow(dead_code)]
pub enum SnapshotType {
    #[default]
    #[strum(serialize = "occ")]
    Occupancy,
    #[strum(serialize = "sd")]
    SignedDistance,
    #[strum(serialize = "ad")]
    AbsoluteDistance,
    #[strum(serialize = "mip")]
    MaximumIntensity,
}

struct SliceStack(Vec<DynamicImage>);

pub fn snapshotter(
    input: Res<ButtonInput<KeyCode>>,
    snapshot_type: Res<SnapshotType>,
    voxel_query: Query<(Entity, &VoxelizationData), ()>,
    images: Res<Assets<Image>>,
) {
    if !input.just_pressed(KeyCode::KeyC) {
        return;
    }

    for (entity, voxel_data) in voxel_query.iter() {
        if voxel_data.state != VoxelizationState::Computed {
            warn!(
                "Voxelization for entity {:?} is not yet computed. Skipping snapshot.",
                entity
            );
            continue;
        }

        let Some(voxel_info) = &voxel_data.data else {
            error!(
                "Entity {:?} has Computed state but missing SignedDistanceFieldData!",
                entity
            );
            continue;
        };

        let Some(image) = images.get(&voxel_info.signed_distance_field) else {
            error!(
                "Image asset for entity {:?} not found in Assets<Image>!",
                entity
            );
            continue;
        };

        let Some(raw_data) = &image.data else {
            error!(
                "Image for entity {:?} has no CPU-accessible data (likely GPU-only).",
                entity
            );
            continue;
        };

        let voxels: &[f32] = bytemuck::cast_slice(raw_data);

        let SliceStack(slices) = match *snapshot_type {
            SnapshotType::Occupancy => occupancy_visualization(voxels),
            SnapshotType::SignedDistance => signed_distance_visualization(voxels),
            SnapshotType::AbsoluteDistance => absolute_distance_visualization(voxels),
            SnapshotType::MaximumIntensity => mip_visualization(voxels),
        };

        let temp_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(TEMP_DIR);
        for (i, slice) in slices.iter().enumerate() {
            let slice_path = temp_path.join(format!("{entity}_{}_{i:03}.png", *snapshot_type));
            _ = slice.save(slice_path).map_err(|e| {
                error!(
                    "Failed to save snapshot for entity {:?} slice {}: {}",
                    entity, i, e
                )
            });
        }

        info!("Snapshot(s) for entity {:?} completed.", entity);
    }
}

fn signed_distance_visualization(voxels: &[f32]) -> SliceStack {
    // Find min/max for normalization
    let min_val = voxels.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = voxels.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let abs_max = min_val.abs().max(max_val.abs());
    info!(min = min_val, max = max_val, abs_max = abs_max);

    let mut slices = Vec::with_capacity(SIZE as usize);

    for z in 0..SIZE {
        let mut img = RgbImage::new(SIZE, SIZE);

        for y in 0..SIZE {
            for x in 0..SIZE {
                let index = (x + y * SIZE + z * SIZE * SIZE) as usize;
                let value = voxels[index];

                // Normalize value to [-1.0, 1.0]
                let normalized = (value / abs_max).clamp(-1.0, 1.0);

                // Map negative -> blue, positive -> red, near surface -> green
                let (r, g, b) = if normalized < -0.01 {
                    (0, 0, 255) // outside
                } else if normalized > 0.01 {
                    (255, 0, 0) // inside
                } else {
                    (0, 255, 0) // near surface
                };

                img.put_pixel(x, SIZE - y - 1, Rgb([r, g, b]));
            }
        }
        slices.insert(z as usize, img.into());
    }

    SliceStack(slices)
}

fn absolute_distance_visualization(voxels: &[f32]) -> SliceStack {
    let unsigned_voxels = voxels.iter().map(|f| f.abs()).collect::<Vec<_>>();

    // Find min/max for normalization
    let min_val = unsigned_voxels
        .iter()
        .cloned()
        .fold(f32::INFINITY, f32::min);
    let max_val = unsigned_voxels
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max);
    info!(min = min_val, max = max_val);

    let mut slices = Vec::with_capacity(SIZE as usize);

    for z in 0..SIZE {
        let mut img = GrayImage::new(SIZE, SIZE);

        for y in 0..SIZE {
            for x in 0..SIZE {
                let index = (x + y * SIZE + z * SIZE * SIZE) as usize;
                let value = unsigned_voxels[index];

                // Normalize to 0..255
                let normalized = ((value - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
                let pixel_value = (normalized * 255.0) as u8;

                img.put_pixel(x, SIZE - y - 1, Luma([pixel_value]));
            }
        }
        slices.insert(z as usize, img.into());
    }

    SliceStack(slices)
}

fn occupancy_visualization(voxels: &[f32]) -> SliceStack {
    // Find min/max for normalization
    let min_val = voxels.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = voxels.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    info!(min = min_val, max = max_val);

    let mut slices = Vec::with_capacity(SIZE as usize);

    for z in 0..SIZE {
        let mut img = GrayImage::new(SIZE, SIZE);

        for y in 0..SIZE {
            for x in 0..SIZE {
                let index = (x + y * SIZE + z * SIZE * SIZE) as usize;
                let value = voxels[index];

                // Normalize to 0..255
                let normalized = if value < 0.0 { 0.0 } else { 1.0 };
                let pixel_value = (normalized * 255.0) as u8;

                img.put_pixel(x, SIZE - y - 1, Luma([pixel_value]));
            }
        }
        slices.insert(z as usize, img.into());
    }

    SliceStack(slices)
}

fn mip_visualization(voxels: &[f32]) -> SliceStack {
    let mut img = GrayImage::new(SIZE, SIZE);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut max_val: f64 = 0.0;
            for z in 0..SIZE {
                let index = (x + y * SIZE + z * SIZE * SIZE) as usize;
                max_val = max_val.max(voxels[index].abs() as f64);
            }
            let pixel_value = ((max_val / 1.0).clamp(0.0, 1.0) * 255.0) as u8;
            img.put_pixel(x, SIZE - y - 1, Luma([pixel_value]));
        }
    }

    SliceStack(vec![img.into()])
}
