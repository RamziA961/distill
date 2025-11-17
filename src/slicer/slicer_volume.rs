use bevy::{
    math::bounding::{Aabb3d, BoundingVolume},
    prelude::*,
};

#[derive(Debug, Clone, Resource)]
pub struct SlicerVolumeData {
    pub volume_bounds: Aabb3d,
    pub slice_height: f32,
}

const DEFAULT_VOLUME: Vec3 = Vec3::splat(10.0);

impl Default for SlicerVolumeData {
    fn default() -> Self {
        let half_size = DEFAULT_VOLUME / 2.0;
        Self {
            volume_bounds: Aabb3d::new(half_size, half_size),
            slice_height: 0.05,
        }
    }
}

pub(super) fn render_slice_volume_visualization(
    volume_data: Res<SlicerVolumeData>,
    mut gizmos: Gizmos,
) {
    let size = (volume_data.volume_bounds.half_size() * 2.0).to_vec3();

    for i in 0..=(size.x as u32) {
        gizmos.line(
            Vec3::new(i as f32, 0.0, 0.0),
            Vec3::new(i as f32, 0.0, size.z),
            Color::WHITE,
        );
    }

    for i in 0..=(size.z as u32) {
        gizmos.line(
            Vec3::new(0.0, 0.0, i as f32),
            Vec3::new(size.x, 0.0, i as f32),
            Color::WHITE,
        );
    }

    let center = volume_data.volume_bounds.center().to_vec3();
    gizmos.cuboid(
        Transform::from_translation(center).with_scale(size),
        Color::BLACK,
    );
}
