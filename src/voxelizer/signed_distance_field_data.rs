use bevy::{math::bounding::Aabb3d, prelude::*};

#[derive(Debug, Clone)]
pub struct SignedDistanceFieldData {
    pub signed_distance_field: Handle<Image>,
}

impl SignedDistanceFieldData {
    pub fn sample_normalized(&self, image: &Image, uvw: Vec3) -> Option<f32> {
        let w = image.texture_descriptor.size.width;
        let h = image.texture_descriptor.size.height;
        let d = image.texture_descriptor.size.depth_or_array_layers;

        if uvw.x < 0.0 || uvw.x > 1.0 || uvw.y < 0.0 || uvw.y > 1.0 || uvw.z < 0.0 || uvw.z > 1.0 {
            return None;
        }

        let idx = uvw * Vec3::new((w - 1) as f32, (h - 1) as f32, (d - 1) as f32);

        let ix = idx.x.round() as usize;
        let iy = idx.y.round() as usize;
        let iz = idx.z.round() as usize;

        if let Some(image_data) = &image.data {
            let voxels: &[f32] = bytemuck::cast_slice(image_data);
            let index = ix + iy * w as usize + iz * w as usize * h as usize;

            if (uvw - Vec3::splat(0.5)).length() < 0.01 {
                info_once!(
                    "uvw={:?}, idx={:?}, ix={}, iy={}, iz={}, index={}, value={:?}",
                    uvw,
                    idx,
                    ix,
                    iy,
                    iz,
                    index,
                    voxels.get(index).copied()
                );
            }

            voxels.get(index).copied()
        } else {
            None
        }
    }

    pub fn sample_world_point(
        &self,
        image: &Image,
        world_pt: Vec3,
        global_transform: &Transform,
        bounds: &Aabb3d,
    ) -> Option<f32> {
        let world_to_local = global_transform.to_matrix().inverse();
        let local_pt = world_to_local.transform_point3(world_pt);

        let extent = bounds.max - bounds.min;
        let uvw = (local_pt - bounds.min.to_vec3()) / extent.to_vec3();
        if world_pt == Vec3::splat(5.0) {
            info_once!(
                "world_pt={:?}, local_pt={:?}, bounds={:?}, uvw={:?}, transform={:?}",
                world_pt,
                local_pt,
                bounds,
                uvw,
                global_transform
            );
        }
        self.sample_normalized(image, uvw)
    }
}
