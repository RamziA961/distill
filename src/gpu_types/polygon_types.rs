use super::GpuVec3;
use bevy::math::Vec3;
use bevy_app_compute::prelude::ShaderType;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug, Zeroable, Pod, ShaderType)]
#[repr(C)]
pub struct GpuTriangle {
    a: GpuVec3,
    b: GpuVec3,
    c: GpuVec3,
    na: GpuVec3,
    nb: GpuVec3,
    nc: GpuVec3,
}

impl GpuTriangle {
    pub fn new(a: GpuVec3, b: GpuVec3, c: GpuVec3, na: GpuVec3, nb: GpuVec3, nc: GpuVec3) -> Self {
        Self {
            a,
            b,
            c,
            na,
            nb,
            nc,
        }
    }

    pub fn a(&self) -> &GpuVec3 {
        &self.a
    }

    pub fn b(&self) -> &GpuVec3 {
        &self.b
    }

    pub fn c(&self) -> &GpuVec3 {
        &self.c
    }

    pub fn na(&self) -> &GpuVec3 {
        &self.na
    }

    pub fn nb(&self) -> &GpuVec3 {
        &self.nb
    }

    pub fn nc(&self) -> &GpuVec3 {
        &self.nc
    }

    pub fn centroid(&self) -> GpuVec3 {
        let a = Vec3::from(self.a);
        let b = Vec3::from(self.b);
        let c = Vec3::from(self.c);
        ((a + b + c) / 3.0).into()
    }

    pub fn bounds(&self) -> (GpuVec3, GpuVec3) {
        let a = Vec3::from(self.a);
        let b = Vec3::from(self.b);
        let c = Vec3::from(self.c);
        let min = a.min(b).min(c);
        let max = a.max(b).max(c);
        (min.into(), max.into())
    }
}
