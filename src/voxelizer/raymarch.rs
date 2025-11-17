use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Component)]
pub struct RaymarchRenderTarget {
    pub source_entity: Entity,
}
