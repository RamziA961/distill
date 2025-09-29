#import "shaders/distance_fns.wgsl"::closest_point_on_triangle
#import "shaders/util_fns.wgsl"::calculate_triangle_normal
#import "shaders/common_types.wgsl"::Box3;

struct VoxelUniforms {
    size: u32,
}

struct MeshUniforms {
    aabb: Box3,
}

@group(0) @binding(0)
var<storage, read_write> voxel_texture: array<f32>;

@group(0) @binding(1)
var<storage> vertices: array<vec3<f32>>;

@group(0) @binding(2)
var<storage> triangles: array<vec3<u32>>;

@group(0) @binding(3)
var<uniform> voxel_uniforms: VoxelUniforms;

@group(0) @binding(4)
var<uniform> mesh_uniforms: MeshUniforms;

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let mesh_min = mesh_uniforms.aabb.min;
    let mesh_extent = mesh_uniforms.aabb.max - mesh_min;
    let size = voxel_uniforms.size;

    if (any(id >= vec3<u32>(size, size, size))) {
        return;
    }

    let voxel_index: u32 = id.x + id.y * size + id.z * size * size;

    // world position of voxel center
    let p_uv = (vec3<f32>(id) + 0.5) / vec3<f32>(size);
    let p_local = p_uv * mesh_extent + mesh_min;
    
    // Closest distance and corresponding triangle normal
    var unsigned_dist = 1e30;
    var closest_normal = vec3<f32>(0.0); // Store normal of the closest triangle
    var closest_pt = vec3<f32>(0.0); // Store the actual closest point on the closest triangle

    for(var t_i = 0u; t_i < arrayLength(&triangles); t_i++) {
        let t = triangles[t_i];

        let a = vertices[t.x];
        let b = vertices[t.y];
        let c = vertices[t.z];

        let curr_closest_pt = closest_point_on_triangle(p_local, a, b, c);
        let dist = length(curr_closest_pt - p_local);
        
        if(dist < unsigned_dist) {
            unsigned_dist = dist;
            closest_pt = curr_closest_pt;
            closest_normal = calculate_triangle_normal(a, b, c);
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
