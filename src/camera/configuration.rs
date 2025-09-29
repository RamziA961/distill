#![allow(dead_code)]
use bevy::prelude::*;
use std::{f32::consts::FRAC_PI_2, marker::PhantomData};

use super::marker::CameraMarker;

#[derive(Resource, Clone, Debug)]
pub struct CameraConfiguration<C>
where
    C: CameraMarker + Component + Clone,
{
    pub rotation_sensitivity: Vec2,
    pub pitch_limit: f32,
    pub translation_velocity: f32,
    _marker: PhantomData<C>,
}

impl<C> CameraConfiguration<C>
where
    C: CameraMarker + Component + Clone,
{
    pub fn with_translation_velocity(mut self, translation_velocity: f32) -> Self {
        self.translation_velocity = translation_velocity;
        self
    }

    pub fn with_rotation_sensitivity(mut self, rotation_sensitivity: Vec2) -> Self {
        self.rotation_sensitivity = rotation_sensitivity;
        self
    }

    pub fn with_pitch_limit(mut self, pitch_limit: f32) -> Self {
        self.pitch_limit = pitch_limit;
        self
    }
}

impl<C> Default for CameraConfiguration<C>
where
    C: CameraMarker + Component + Clone,
{
    fn default() -> Self {
        Self {
            rotation_sensitivity: Vec2::new(0.003, 0.002),
            pitch_limit: FRAC_PI_2 - 0.01,
            translation_velocity: 2.0,
            _marker: PhantomData,
        }
    }
}
