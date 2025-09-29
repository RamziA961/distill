use bevy::prelude::*;

use crate::camera::{configuration::CameraConfiguration, marker::CameraMarker};

pub fn camera_translation_system<C>(
    mut camera_transforms: Query<&mut Transform, With<C>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,
    camera_config: Res<CameraConfiguration<C>>,
) where
    C: CameraMarker + Component + Clone,
{
    let v = camera_config.translation_velocity;
    for mut transform in camera_transforms.iter_mut() {
        let mut dir = Vec3::ZERO;
        for key in keyboard.get_pressed() {
            match key {
                KeyCode::KeyW => {
                    dir += Vec3::new(0.0, 0.0, -1.0);
                }
                KeyCode::KeyA => {
                    dir += Vec3::new(-1.0, 0.0, 0.0);
                }
                KeyCode::KeyD => {
                    dir += Vec3::new(1.0, 0.0, 0.0);
                }
                KeyCode::KeyS => {
                    dir += Vec3::new(0.0, 0.0, 1.0);
                }
                _ => {}
            }
            let rotation = transform.rotation;
            let rotated = rotation.mul_vec3(dir);
            transform.translation += rotated * time.delta_secs() * v;
        }
    }
}
