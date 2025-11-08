use crate::gpu_types::{GpuBox3, GpuBvhNode};
use bevy::{
    math::Vec3,
    render::mesh::{Mesh, VertexAttributeValues},
};

pub trait MeshBvh {
    fn build_bvh(&self, leaf_size: usize) -> (Vec<GpuBvhNode>, Vec<Triangle>);
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    pub fn centroid(&self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }

    pub fn bounds(&self) -> (Vec3, Vec3) {
        let min = self.a.min(self.b).min(self.c);
        let max = self.a.max(self.b).max(self.c);
        (min, max)
    }
}

impl MeshBvh for Mesh {
    fn build_bvh(&self, leaf_size: usize) -> (Vec<GpuBvhNode>, Vec<Triangle>) {
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
            let a = Vec3::from_array(positions[tri[0]]);
            let b = Vec3::from_array(positions[tri[1]]);
            let c = Vec3::from_array(positions[tri[2]]);
            tris.push(Triangle { a, b, c });
        }

        let mut tris_mut = tris.clone();
        let nodes = build_bvh(&mut tris_mut, leaf_size);
        (nodes, tris)
    }
}

fn build_bvh(triangles: &mut [Triangle], leaf_size: usize) -> Vec<GpuBvhNode> {
    let mut nodes = Vec::new();
    build_node(triangles, 0, triangles.len(), leaf_size, &mut nodes);
    nodes
}

fn build_node(
    tris: &mut [Triangle],
    start: usize,
    end: usize,
    leaf_size: usize,
    nodes: &mut Vec<GpuBvhNode>,
) -> u32 {
    // Compute AABB
    let mut node_min = Vec3::splat(f32::INFINITY);
    let mut node_max = Vec3::splat(f32::NEG_INFINITY);
    for t in &tris[start..end] {
        let (bmin, bmax) = t.bounds();
        node_min = node_min.min(bmin);
        node_max = node_max.max(bmax);
    }

    let count = end - start;
    let node_index = nodes.len() as u32;

    if count <= leaf_size {
        // Leaf node
        nodes.push(GpuBvhNode::new(
            GpuBox3::new(node_min.into(), node_max.into()),
            start as u32,
            count as u32,
        ));
        return node_index;
    }

    // Choose split axis
    let extent = node_max - node_min;
    let axis = if extent.x > extent.y && extent.x > extent.z {
        0
    } else if extent.y > extent.z {
        1
    } else {
        2
    };

    // Compute split position: median of centroids along chosen axis
    let mid = (start + end) / 2;
    tris[start..end].sort_by(|a, b| {
        a.centroid()[axis]
            .partial_cmp(&b.centroid()[axis])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Placeholder internal node (filled after children are built)
    nodes.push(GpuBvhNode::new(
        GpuBox3::new(node_min.into(), node_max.into()),
        0,
        0,
    ));

    let left = build_node(tris, start, mid, leaf_size, nodes);
    let _right = build_node(tris, mid, end, leaf_size, nodes);

    // Fill in left_first after children are built
    nodes[node_index as usize].with_entry_index(left);

    node_index
}
