use std::path::Path;

use bevy::prelude::*;
use bevy_app_compute::prelude as compute;
use image::{GrayImage, Luma, Rgb, RgbImage};

use crate::voxelization::voxelization_worker::{SIZE, VoxelVariables, VoxelizationWorker};

#[derive(Resource, Debug, Default)]
#[allow(dead_code)]
pub enum SnapshotType {
    #[default]
    Occupancy,
    SignedDistance,
    AbsoluteDistance,
}

pub fn snapshotter(
    input: Res<ButtonInput<KeyCode>>,
    snapshot_type: Res<SnapshotType>,
    worker: Res<compute::AppComputeWorker<VoxelizationWorker>>,
) {
    if !worker.ready() {
        return;
    }

    if !input.just_pressed(KeyCode::KeyC) {
        return;
    }

    let voxels = worker.read_vec::<f32>(VoxelVariables::VoxelTexture.as_ref());

    match snapshot_type.into_inner() {
        SnapshotType::Occupancy => occupancy_visualization(&voxels),
        SnapshotType::SignedDistance => signed_distance_visualization(&voxels),
        SnapshotType::AbsoluteDistance => absolute_distance_visualization(&voxels),
    }

    info!("Saved {} voxel slices to disk", SIZE);
}

fn signed_distance_visualization(voxels: &[f32]) {
    // Find min/max for normalization
    let min_val = voxels.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = voxels.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let abs_max = min_val.abs().max(max_val.abs());
    info!(min = min_val, max = max_val, abs_max = abs_max);

    for z in 0..SIZE {
        let mut img = RgbImage::new(SIZE, SIZE);

        for y in 0..SIZE {
            for x in 0..SIZE {
                let index = (x + y * SIZE + z * SIZE * SIZE) as usize;
                let value = voxels[index];

                // Normalize value to [-1.0, 1.0]
                let normalized = (value / abs_max).clamp(-1.0, 1.0);

                // Map negative -> blue, positive -> red
                let (r, g, b) = if normalized < 0.0 {
                    let intensity = (-normalized * 255.0) as u8;
                    (0, 0, intensity) // blue channel
                } else {
                    let intensity = (normalized * 255.0) as u8;
                    (intensity, 0, 0) // red channel
                };

                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        // Save slice as PNG
        let filename = format!("temp/voxel_slice_{z:03}.png");
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(filename);
        img.save(p).expect("Failed to save voxel slice image");
    }
}

fn absolute_distance_visualization(voxels: &[f32]) {
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

    for z in 0..SIZE {
        let mut img = GrayImage::new(SIZE, SIZE);

        for y in 0..SIZE {
            for x in 0..SIZE {
                let index = (x + y * SIZE + z * SIZE * SIZE) as usize;
                let value = unsigned_voxels[index];

                // Normalize to 0..255
                let normalized = ((value - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
                let pixel_value = (normalized * 255.0) as u8;

                img.put_pixel(x, y, Luma([pixel_value]));
            }
        }
        // Save slice as PNG
        let filename = format!("temp/voxel_slice_{z:03}.png");
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(filename);
        img.save(p).expect("Failed to save voxel slice image");
    }
}

fn occupancy_visualization(voxels: &[f32]) {
    // Find min/max for normalization
    let min_val = voxels.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = voxels.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    info!(min = min_val, max = max_val);

    for z in 0..SIZE {
        let mut img = GrayImage::new(SIZE, SIZE);

        for y in 0..SIZE {
            for x in 0..SIZE {
                let index = (x + y * SIZE + z * SIZE * SIZE) as usize;
                let value = voxels[index];

                // Normalize to 0..255
                let normalized = if value < 0.0 { 0.0 } else { 1.0 };
                let pixel_value = (normalized * 255.0) as u8;

                img.put_pixel(x, y, Luma([pixel_value]));
            }
        }
        // Save slice as PNG
        let filename = format!("temp/voxel_slice_{z:03}.png");
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(filename);
        img.save(p).expect("Failed to save voxel slice image");
    }
}
