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
    
    // Closest distance and corresponding triangle normal
    var unsigned_dist = 1e30;
    var closest_normal = vec3<f32>(0.0); // Store normal of the closest triangle
    var closest_pt = vec3<f32>(0.0); // Store the actual closest point on the closest triangle

    var stack: array<u32, STACK_SIZE>;
    var stack_ptr = 1u;
    stack[0] = 0u; // root node

    loop {
        if (stack_ptr == 0u) {
            break;
        }
        
        stack_ptr -= 1u; // pop
        let node_index = stack[stack_ptr];

        let node = bvh_nodes[node_index];
        // AABB distance test
        let dmin = distance_to_aabb(p_local, node.aabb.min, node.aabb.max);
        if(dmin > unsigned_dist) {
            // Skip nodes farther than current best
            continue;
        }

        if (node.triangle_count > 0u) {
            // Leaf node
            for (var i = 0u; i < node.triangle_count; i++) {
                let tri_idx = node.left_index + i;
                let t = triangles[tri_idx];

                let curr_closest_pt = closest_point_on_triangle(p_local, t.a, t.b, t.c);
                let dist = length(curr_closest_pt - p_local);

                if (dist < unsigned_dist) {
                    unsigned_dist = dist;
                    closest_pt = curr_closest_pt;
                    closest_normal = calculate_triangle_normal(t.a, t.b, t.c);
                }
            }
        } else {
            // Internal node - push children onto stack
            if (stack_ptr + 2u > STACK_SIZE) {
                // Avoid overflow
                continue;
            }

            let left = node.left_index;
            let right = node.right_index;
            
            // Compute rough distance to children AABB to order pushes
            let dleft = distance_to_aabb(p_local, bvh_nodes[left].aabb.min, bvh_nodes[left].aabb.max);
            let dright = distance_to_aabb(p_local, bvh_nodes[right].aabb.min, bvh_nodes[right].aabb.max);

            // Push the farther one first (so nearer one gets processed next)
            if(dleft < dright) {
                // left first
                stack[stack_ptr] = right;
                stack[stack_ptr + 1u] = left;
            } else {
                // right first
                stack[stack_ptr] = left;
                stack[stack_ptr + 1u] = right;
            }
            stack_ptr += 2u;
            
        }
    }

    // Sign determination using closest normal
    // For a point P and its closest point on the surface (closest_p_on_tri) with normal (closest_normal)
    // if dot(P - closest_p_on_tri, closest_normal) > 0, P is outside, else inside.
    let vec_to_surface = p_local - closest_pt;
    let sign_dot_product = dot(vec_to_surface, closest_normal);
    // Use >= to handle points exactly on the surface as outside
    let si = select(-1.0, 1.0, sign_dot_product >= 0.0);

    let value = unsigned_dist * si;
    voxel_texture[voxel_index] = value;
}
