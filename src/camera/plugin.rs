use bevy::prelude::*;

use super::configuration::CameraConfiguration;
use super::marker::CameraMarker;
use super::systems::{rotation::camera_rotation_system, translation::camera_translation_system};

#[derive(Clone, Default)]
pub struct CameraPlugin<C>
where
    C: CameraMarker + Component + Clone,
{
    pub configuration: CameraConfiguration<C>,
}

impl<C> Plugin for CameraPlugin<C>
where
    C: CameraMarker + Component + Clone,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.configuration.clone()).add_systems(
            Update,
            (camera_translation_system::<C>, camera_rotation_system::<C>),
        );
    }
}
