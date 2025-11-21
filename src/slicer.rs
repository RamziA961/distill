use bevy::prelude::*;

pub(crate) mod slicer_volume;
mod slicing_systems;

pub struct SlicerPlugin;

impl Plugin for SlicerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                slicer_volume::render_slice_volume_visualization,
                slicing_systems::slice_volume,
            ),
        );
    }
}

