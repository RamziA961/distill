#import "shaders/distance_fns.wgsl"::{distance_to_aabb, closest_point_on_triangle};
#import "shaders/util_fns.wgsl"::calculate_triangle_normal;
#import "shaders/common_types.wgsl"::{Box3, BvhNode, Triangle};

struct VoxelUniforms {
    size: u32,
}

@group(0) @binding(0)
var<storage, read_write> voxel_texture: array<f32>;

@group(0) @binding(1)
var<storage> triangles: array<Triangle>;

@group(0) @binding(2)
var<storage> bvh_nodes: array<BvhNode>;

@group(0) @binding(3)
var<uniform> voxel_uniforms: VoxelUniforms;

const STACK_SIZE: u32 = 128;

struct ClosestResult {
    dist: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
};

/// Traverse BVH and return closest point + distance + normal
fn closest_point_bvh(p_local: vec3<f32>) -> ClosestResult {
    var best = ClosestResult(1e30, vec3<f32>(0.0), vec3<f32>(0.0));

    var stack: array<u32, STACK_SIZE>;
    var stack_ptr = 1u;
    stack[0] = 0u; // root node

    loop {
        if (stack_ptr == 0u) {
            break;
        }

        stack_ptr -= 1u;
        let node_index = stack[stack_ptr];
        let node = bvh_nodes[node_index];

        // Early skip if AABB is farther than current best
        let dmin = distance_to_aabb(p_local, node.aabb.min, node.aabb.max);
        if (dmin > best.dist) {
            continue;
        }

        if (node.triangle_count > 0u) {
            // Leaf node
            for (var i = 0u; i < node.triangle_count; i++) {
                let tri_idx = node.left_index + i;
                let tri = triangles[tri_idx];

                let curr_pt = closest_point_on_triangle(p_local, tri.a, tri.b, tri.c);
                let dist = length(curr_pt - p_local);

                if (dist < best.dist) {
                    best.dist = dist;
                    best.point = curr_pt;
                    best.normal = calculate_triangle_normal(tri.a, tri.b, tri.c);
                }
            }
        } else {
            // Internal node
            if (stack_ptr + 2u > STACK_SIZE) {
                continue;
            }

            let left = node.left_index;
            let right = node.right_index;

            let dleft = distance_to_aabb(p_local, bvh_nodes[left].aabb.min, bvh_nodes[left].aabb.max);
            let dright = distance_to_aabb(p_local, bvh_nodes[right].aabb.min, bvh_nodes[right].aabb.max);

            // Push farther first so nearer pops next
            if (dleft < dright) {
                stack[stack_ptr] = right;
                stack[stack_ptr + 1u] = left;
            } else {
                stack[stack_ptr] = left;
                stack[stack_ptr + 1u] = right;
            }
            stack_ptr += 2u;
        }
    }

    return best;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let aabb = bvh_nodes[0].aabb;
    let mesh_extent = aabb.max - aabb.min;
    let size = voxel_uniforms.size;

    if (any(id >= vec3<u32>(size, size, size))) {
        return;
    }

    let voxel_index: u32 = id.x + id.y * size + id.z * size * size;

    // world position of voxel center
    let p_uv = (vec3<f32>(id) + 0.5) / vec3<f32>(size);
    let p_local = p_uv * mesh_extent + aabb.min;
    
    // Get closest point & normal via BVH
    let result = closest_point_bvh(p_local);

    // Sign determination using closest normal
    // For a point P and its closest point on the surface (closest_p_on_tri) with normal (closest_normal)
    // if dot(P - closest_p_on_tri, closest_normal) > 0, P is outside, else inside.
    let vec_to_surface = p_local - result.point;
    let sign_dot_product = dot(vec_to_surface, result.normal);
    // Use >= to handle points exactly on the surface as outside
    let si = select(-1.0, 1.0, sign_dot_product >= 0.0);

    let value = result.dist * si;
    voxel_texture[voxel_index] = value;
}
