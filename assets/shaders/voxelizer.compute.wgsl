#import "shaders/distance_fns.wgsl"::{distance_to_aabb, closest_point_on_triangle};
#import "shaders/util_fns.wgsl"::{ray_aabb_intersect, ray_triangle_intersect};
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
const PADDING_RATIO: f32 = 0.05;

struct ClosestResult {
    dist: f32,       // shortest distance found so far
    point: vec3<f32>, // closest point on the surface
    normal: vec3<f32>, // interpolated normal at closest point
};

/// Traverse a BVH to find the closest point on the mesh to `p_local`.
/// Returns a `ClosestResult` with the shortest distance, the closest point,
/// and the normal interpolated from the triangle vertices.
///
/// Performs early AABB culling to skip branches that cannot yield a closer point.
fn closest_point_bvh(p_local: vec3<f32>) -> ClosestResult {
    // Initialize best result with a very large distance
    var best = ClosestResult(1e30, vec3<f32>(0.0), vec3<f32>(0.0));

    // Stack for iterative traversal
    var stack: array<u32, STACK_SIZE>;
    var stack_ptr = 1u;
    stack[0] = 0u; // start with root node

    loop {
        if (stack_ptr == 0u) {
            break; // finished traversal
        }

        // Pop node from stack
        stack_ptr -= 1u;
        let node_index = stack[stack_ptr];
        let node = bvh_nodes[node_index];

        // Early AABB culling: skip this node if its closest point is farther than the current best
        let dmin = distance_to_aabb(p_local, node.aabb.min, node.aabb.max);
        if (dmin > best.dist) {
            continue;
        }

        if (node.triangle_count > 0u) {
            // Leaf node: test all triangles
            for (var i = 0u; i < node.triangle_count; i++) {
                let tri_idx = node.left_index + i;
                let tri = triangles[tri_idx];

                // Compute closest point on triangle
                let result = closest_point_on_triangle(p_local, tri.a, tri.b, tri.c);
                let dist = length(result.point - p_local);

                // Update best result if closer
                if (dist < best.dist) {
                    best.dist = dist;
                    best.point = result.point;
                    // Interpolate normal from vertex normals using barycentric coordinates
                    best.normal = normalize(
                        tri.na * result.barycentric.x +
                        tri.nb * result.barycentric.y +
                        tri.nc * result.barycentric.z
                    );
                }
            }
        } else {
            // Internal node: push children onto the stack
            if (stack_ptr + 2u > STACK_SIZE) {
                continue; // avoid stack overflow
            }

            let left = node.left_index;
            let right = node.right_index;

            // Compute minimum distances to children
            let dleft = distance_to_aabb(p_local, bvh_nodes[left].aabb.min, bvh_nodes[left].aabb.max);
            let dright = distance_to_aabb(p_local, bvh_nodes[right].aabb.min, bvh_nodes[right].aabb.max);

            // Push the farther child first so the nearer child is processed next
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

/// Raycast-based inside/outside test using the "odd-even rule":
/// Cast a ray in the +X direction from point `p`.
/// Count the number of intersections with triangles.
/// - Odd count -> point is inside
/// - Even count -> point is outside
///
/// Uses the same BVH structure for efficient ray intersection testing.
fn is_inside(p: vec3<f32>) -> bool {
    var count: u32 = 0u; // number of ray-triangle intersections
    let ray_dir = vec3<f32>(1.0, 0.5, 0.3); // ray direction along +X

    var stack: array<u32, STACK_SIZE>;
    var stack_ptr = 1u;
    stack[0] = 0u; // start with root node

    loop {
        if (stack_ptr == 0u) { 
            break; // finished traversal
        }
        stack_ptr -= 1u;
        let node = bvh_nodes[stack[stack_ptr]];

        // Skip this node if ray does not intersect the node's AABB
        if (!ray_aabb_intersect(p, ray_dir, node.aabb).hit) {
            continue;
        }

        if (node.triangle_count > 0u) {
            // Leaf node: test all triangles for ray intersection
            for (var i = 0u; i < node.triangle_count; i++) {
                let tri = triangles[node.left_index + i];
                if (ray_triangle_intersect(p, ray_dir, tri.a, tri.b, tri.c).hit) {
                    count += 1u;
                }
            }
        } else {
            // Internal node: push children onto stack
            if (stack_ptr + 2u > STACK_SIZE) { 
                continue; // avoid stack overflow
            }
            stack[stack_ptr] = node.left_index;
            stack[stack_ptr + 1u] = node.right_index;
            stack_ptr += 2u;
        }
    }

    // Inside if intersection count is odd
    return (count % 2u) == 1u;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = voxel_uniforms.size;
    if (any(id >= vec3<u32>(size, size, size))) {
        return;
    }
    let root_aabb = bvh_nodes[0].aabb;
    let mesh_extent = root_aabb.max - root_aabb.min;

    let voxel_index = id.x + id.y * size + id.z * size * size;

    // Add padding around the mesh
    let padding_vec = mesh_extent * PADDING_RATIO;
    let padded_min = root_aabb.min - padding_vec;
    let padded_max = root_aabb.max + padding_vec;
    let padded_extent = padded_max - padded_min;

    // World position of voxel center
    let p_uv = (vec3<f32>(id) + 0.5) / vec3<f32>(size);
    // World position of voxel center in padded grid
    let p_local = p_uv * padded_extent + padded_min;
    
    // Get closest point & normal via BVH
    let result = closest_point_bvh(p_local);
    let inside = is_inside(p_local);
    
    let si = select(1.0, -1.0, inside);
    let value = result.dist * si;
    voxel_texture[voxel_index] = value;
}
