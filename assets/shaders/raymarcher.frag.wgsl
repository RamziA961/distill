#import "shaders/common_types.wgsl"::{Box3, Camera}
#import "shaders/util_fns.wgsl"::intersect_box

@group(2) @binding(0)
var<storage, read> voxel_texture: array<f32>;

@group(2) @binding(1)
var<uniform> camera: Camera;

@group(2) @binding(2)
var<uniform> grid_size: u32;

@group(2) @binding(3)
var<uniform> grid_bounds: Box3;

fn voxel_lookup(p: vec3<f32>) -> f32 {
    let extent = grid_bounds.max - grid_bounds.min;
    let rel = (p - grid_bounds.min) / extent;

    // Clamp to [0, 1]
    let uvw = clamp(rel, vec3<f32>(0.0), vec3<f32>(1.0));

    // Convert to integer voxel index
    let xi = u32(uvw.x * f32(grid_size - 1u));
    let yi = u32(uvw.y * f32(grid_size - 1u));
    let zi = u32(uvw.z * f32(grid_size - 1u));

    let idx = xi + yi * grid_size + zi * grid_size * grid_size;
    return voxel_texture[idx];
}


const EPSILON: f32 = 0.0001;
const MAX_STEPS = 128u;

fn raymarch(origin: vec3<f32>, dir: vec3<f32>) -> f32 {
    var t = 0.0;
    let max_dist = length(grid_bounds.max - grid_bounds.min);

    for (var i = 0u; i < MAX_STEPS; i++) {
        let p = origin + dir * t;
        let d = voxel_lookup(p);

        if (d < EPSILON) {
            return t; // hit
        }

        t += d;
        if (t > max_dist) {
            break; // escaped
        }
    }
    return -1.0; // miss
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
) -> @location(0) vec4<f32> {
    let frag_coord = position.xy;
    let camera_position = camera.inv_view_mat[3].xyz;

    let dir = normalize(world_position.xyz - camera_position);

    // Intersect with the cube bounding the SDF
    let hit = intersect_box(camera_position, dir, grid_bounds);
    if (!hit.hit) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0); // background
    }

    // Raymarch inside bounds
    let start = camera_position + dir * max(hit.t_min, 0.0);
    let t = raymarch(start, dir);
    if (t < 0.0) {
        return vec4<f32>(0.5, 0.0, 0.0, 0.0); // miss inside cube
    }

    let p = camera_position + dir * t;
    let n = normalize(vec3<f32>(
        voxel_lookup(p + vec3<f32>(EPSILON, 0, 0)) - voxel_lookup(p - vec3<f32>(EPSILON, 0, 0)),
        voxel_lookup(p + vec3<f32>(0, EPSILON, 0)) - voxel_lookup(p - vec3<f32>(0, EPSILON, 0)),
        voxel_lookup(p + vec3<f32>(0, 0, EPSILON)) - voxel_lookup(p - vec3<f32>(0, 0, EPSILON))
    ));

    return vec4<f32>(0.5 * (n + vec3<f32>(1.0)), 1.0);
}

