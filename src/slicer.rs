use bevy::prelude::*;

use crate::slicer::slicer_volume::render_slice_volume_visualization;

pub(crate) mod slicer_volume;

pub struct SlicerPlugin;

impl Plugin for SlicerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_slice_volume_visualization);
    }
}

