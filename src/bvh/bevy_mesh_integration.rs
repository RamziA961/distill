use super::{MeshBvh, bvh_builder};
use crate::gpu_types::{GpuBvhNode, GpuTriangle, GpuVec3};
use bevy::{prelude::*, render::mesh::VertexAttributeValues};

impl MeshBvh for Mesh {
    fn build_bvh(&self, leaf_size: usize) -> (Vec<GpuBvhNode>, Vec<GpuTriangle>) {
        use bevy::render::mesh::PrimitiveTopology;

        assert_eq!(
            self.primitive_topology(),
            PrimitiveTopology::TriangleList,
            "Only triangle meshes are supported for BVH building."
        );

        let positions = match self.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() {
            VertexAttributeValues::Float32x3(v) => v.clone(),
            _ => panic!("Unexpected vertex format for positions"),
        };

        let indices = self
            .indices()
            .map(|i| i.iter().collect::<Vec<_>>())
            .unwrap_or_else(|| (0..positions.len()).collect());

        let mut tris = Vec::with_capacity(indices.len() / 3);
        for tri in indices.chunks_exact(3) {
            let a = GpuVec3::from_array(&positions[tri[0]]);
            let b = GpuVec3::from_array(&positions[tri[1]]);
            let c = GpuVec3::from_array(&positions[tri[2]]);
            tris.push(GpuTriangle::new(a, b, c));
        }

        let nodes = bvh_builder::build_bvh(&tris, leaf_size);
        (nodes, tris)
    }
}
