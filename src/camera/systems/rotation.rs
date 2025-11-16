use crate::camera::{configuration::CameraConfiguration, marker::CameraMarker};
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub fn camera_rotation_system<C>(
    mut camera_transforms: Query<&mut Transform, With<C>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    camera_config: Res<CameraConfiguration<C>>,
    cursor_options: Single<&CursorOptions, With<PrimaryWindow>>,
) where
    C: CameraMarker + Component + Clone,
{
    // Prevent rotation when cursor is active
    if cursor_options.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let delta = if mouse_motion.delta != Vec2::ZERO {
        mouse_motion.delta
    } else {
        return;
    };

    for mut transform in camera_transforms.iter_mut() {
        let d_yaw = -delta.x * camera_config.rotation_sensitivity.x;
        let d_pitch = -delta.y * camera_config.rotation_sensitivity.y;

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + d_yaw;
        let pitch = (pitch + d_pitch).clamp(-camera_config.pitch_limit, camera_config.pitch_limit);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}
