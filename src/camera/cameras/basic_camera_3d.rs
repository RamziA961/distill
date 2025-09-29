use crate::camera::marker::CameraMarker;
use bevy::prelude::*;

#[allow(dead_code)]
pub fn initialization<C>(mut commands: Commands)
where
    C: CameraMarker + Component + Clone + Default,
{
    commands.spawn((C::default(), Camera3d::default()));
}
