use crate::gpu_types::{GpuBox3, GpuBvhNode, GpuTriangle};
use bevy::math::Vec3;

pub(super) fn build_bvh(triangles: &[GpuTriangle], leaf_size: usize) -> Vec<GpuBvhNode> {
    let mut nodes = Vec::new();
    let mut triangle_indices: Vec<u32> = (0..triangles.len() as u32).collect();
    build_node(
        triangles,
        &mut triangle_indices,
        0,
        triangles.len(),
        leaf_size,
        &mut nodes,
    );
    nodes
}

fn build_node(
    triangles: &[GpuTriangle],
    triangle_indices: &mut [u32],
    start: usize,
    end: usize,
    leaf_size: usize,
    nodes: &mut Vec<GpuBvhNode>,
) -> u32 {
    // Compute AABB
    let mut node_min = Vec3::splat(f32::INFINITY);
    let mut node_max = Vec3::splat(f32::NEG_INFINITY);
    for t in &triangles[start..end] {
        let (bmin, bmax) = t.bounds();
        node_min = node_min.min(bmin.into());
        node_max = node_max.max(bmax.into());
    }

    let count = end - start;
    let node_index = nodes.len() as u32;

    nodes.push(GpuBvhNode::new(
        GpuBox3::new(node_min.into(), node_max.into()),
        0,
        0,
        0,
    ));

    if count <= leaf_size {
        // Leaf node
        nodes[node_index as usize].with_left_index(start as u32);
        nodes[node_index as usize].with_right_index(u32::MAX);
        nodes[node_index as usize].with_triangle_count(count as u32);
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

    // Compute median and sort indices along axis
    let mid = (start + end) / 2;
    triangle_indices[start..end].sort_by(|&a_idx, &b_idx| {
        let a = triangles[a_idx as usize].centroid()[axis];
        let b = triangles[b_idx as usize].centroid()[axis];
        a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
    });

    let left = build_node(triangles, triangle_indices, start, mid, leaf_size, nodes);
    let right = build_node(triangles, triangle_indices, mid, end, leaf_size, nodes);

    nodes[node_index as usize].with_left_index(left);
    nodes[node_index as usize].with_right_index(right);

    node_index
}
