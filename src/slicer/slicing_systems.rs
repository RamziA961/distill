use crate::{
    bvh::BvhData,
    voxelizer::{VoxelizationData, VoxelizationState},
};
use bevy::prelude::*;
use image::GrayImage;
use std::path::Path;

use super::slicer_volume::SlicerVolumeData;

const TEMP_DIR: &str = "temp";

pub fn slice_volume(
    slice_bounds: Res<SlicerVolumeData>,
    voxel_data: Query<(&Transform, &VoxelizationData, &BvhData), ()>,
    images: Res<Assets<Image>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if !input.just_pressed(KeyCode::KeyO) {
        return;
    }

    info!(
        "Slicing volume with {} voxelized entities.",
        voxel_data.iter().count()
    );

    let filtered_voxel_data: Vec<_> = voxel_data
        .into_iter()
        .filter(|(_, voxel_data, _)| voxel_data.state == VoxelizationState::Computed)
        .collect();

    info!(
        "Slicing volume with {} voxelized entities.",
        filtered_voxel_data.len()
    );

    let volume = slice_bounds.volume_bounds;
    let resolution = slice_bounds.volume_resolution;

    let size = volume.max;

    let dx = resolution.x;
    let dy = resolution.y;
    let dz = resolution.z;

    let x_steps = (size.x / dx) as u32;
    let y_steps = (size.y / dy) as u32;
    let z_steps = (size.z / dz) as u32;

    let mut slices = Vec::with_capacity(size.y as usize);

    for y_step in 0..y_steps {
        let mut slice = GrayImage::new(x_steps, z_steps);
        slice.fill(255);

        let y = y_step as f32 * dy;

        for z_step in 0..z_steps {
            let z = z_step as f32 * dz;

            for x_step in 0..x_steps {
                let x = x_step as f32 * dx;
                let sample_point = Vec3::new(x, y, z);

                for (transform, voxel_data, bvh_data) in filtered_voxel_data.iter() {
                    if voxel_data.state != VoxelizationState::Computed {
                        continue;
                    }

                    let Some(voxel_info) = &voxel_data.data else {
                        continue;
                    };

                    let Some(image) = images.get(&voxel_info.signed_distance_field) else {
                        continue;
                    };

                    let bounds = bvh_data.nodes[0].aabb();

                    let Some(sdf_value) = voxel_info.sample_world_point(
                        image,
                        sample_point,
                        transform,
                        &bounds.into(),
                    ) else {
                        continue;
                    };

                    let img_x = z_step;
                    let img_y = x_steps - x_step - 1;
                    info_once!(img_x = img_x, img_y = img_y);

                    let normalized = if sdf_value < 0.0 { 0.0 } else { 1.0 };
                    let pixel_value = (normalized * 255.0) as u8;
                    slice.put_pixel(img_x, img_y, image::Luma([pixel_value]));
                }
            }
        }

        slices.insert(y_step as usize, slice);
    }

    let temp_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(TEMP_DIR);
    for (i, slice) in slices.iter().enumerate() {
        let slice_path = temp_path.join(format!("slice_{i:03}.png"));
        _ = slice.save(slice_path);
    }
}
