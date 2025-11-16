use bevy::prelude::*;
use bevy_app_compute::prelude::ShaderType;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Pod, Zeroable, Debug, ShaderType)]
#[repr(C)]
pub struct GpuCamera {
    view_mat: Mat4,
    inv_view_mat: Mat4,
    projection_mat: Mat4,
    inv_projection_mat: Mat4,
}

impl GpuCamera {
    pub fn new(
        view_mat: Mat4,
        inv_view_mat: Mat4,
        projection_mat: Mat4,
        inv_projection_mat: Mat4,
    ) -> Self {
        Self {
            view_mat,
            inv_view_mat,
            projection_mat,
            inv_projection_mat,
        }
    }
    pub fn view_mat(&self) -> &Mat4 {
        &self.view_mat
    }

    pub fn inv_view_mat(&self) -> &Mat4 {
        &self.inv_view_mat
    }

    pub fn projection_mat(&self) -> &Mat4 {
        &self.projection_mat
    }

    pub fn inv_projection_mat(&self) -> &Mat4 {
        &self.inv_projection_mat
    }

    pub fn from_transform_and_projection(transform: &Transform, projection: &Projection) -> Self {
        let t_mat = transform.to_matrix();
        let proj_mat = projection.get_clip_from_view();
        Self {
            view_mat: t_mat.inverse(),
            inv_view_mat: t_mat,
            inv_projection_mat: proj_mat.inverse(),
            projection_mat: proj_mat,
        }
    }
}
