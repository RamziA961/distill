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

        let indices: Vec<usize> = match self.indices() {
            Some(bevy::render::mesh::Indices::U16(i)) => i.iter().map(|&v| v as usize).collect(),
            Some(bevy::render::mesh::Indices::U32(i)) => i.iter().map(|&v| v as usize).collect(),
            None => (0..positions.len()).collect(),
        };

        let normals = match self.attribute(Mesh::ATTRIBUTE_NORMAL) {
            Some(VertexAttributeValues::Float32x3(v)) => v.clone(),
            _ => {
                panic!("Mesh is missing normals — try recomputing them before building the BVH");
            }
        };

        let mut tris = Vec::with_capacity(indices.len() / 3);
        for tri in indices.chunks_exact(3) {
            let i0 = tri[0];
            let i1 = tri[1];
            let i2 = tri[2];

            let a = GpuVec3::from_array(&positions[i0]);
            let b = GpuVec3::from_array(&positions[i1]);
            let c = GpuVec3::from_array(&positions[i2]);

            let na = GpuVec3::from_array(&normals[i0]);
            let nb = GpuVec3::from_array(&normals[i1]);
            let nc = GpuVec3::from_array(&normals[i2]);

            tris.push(GpuTriangle::new(a, b, c, na, nb, nc));
        }

        let nodes = bvh_builder::build_bvh(&tris, leaf_size);
        (nodes, tris)
    }
}
