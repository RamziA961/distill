use bevy::prelude::*;

pub trait CameraMarker {}

#[macro_export]
macro_rules! generate_camera_markers {
    ($($marker_name:ident),*) => {
        $(
            #[allow(dead_code)]
            #[derive(Component, Clone, Default)]
            pub struct $marker_name;

            impl CameraMarker for $marker_name {}
        )*
    }
}

generate_camera_markers!(
    CameraMarkerPrimary,
    CameraMarkerSeconday,
    CameraMarkerTertiary,
    CameraMarkerSupporting
);
