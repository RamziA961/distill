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

@group(2) @binding(4)
var<uniform> screen_resolution: vec2<f32>;

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

//@fragment
//fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
//    let frag_coord = position.xy;
//    let uv = (frag_coord / screen_resolution) * 2.0 - 1.0;
//
//    let origin = camera.position;
//    let dir = normalize(camera.forward + uv.x * camera.right + uv.y * camera.up);
//
//    // Intersect with the cube bounding the SDF
//    let hit = intersect_box(origin, dir, grid_bounds);
//    if (!hit.hit) {
//        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // background
//    }
//
//    let start = origin + dir * max(hit.t_min, 1.0);
//    // Raymarch inside bounds
//    let t = raymarch(start, dir);
//    if (t < 0.0) {
//        return vec4<f32>(0.5, 0.0, 0.0, 0.0); // miss inside cube
//    }
//
//    let p = origin + dir * t;
//    let n = normalize(vec3<f32>(
//        voxel_lookup(p + vec3<f32>(EPSILON, 0, 0)) - voxel_lookup(p - vec3<f32>(EPSILON, 0, 0)),
//        voxel_lookup(p + vec3<f32>(0, EPSILON, 0)) - voxel_lookup(p - vec3<f32>(0, EPSILON, 0)),
//        voxel_lookup(p + vec3<f32>(0, 0, EPSILON)) - voxel_lookup(p - vec3<f32>(0, 0, EPSILON))
//    ));
//
//    return vec4<f32>(0.5 * (n + vec3<f32>(1.0)), 1.0);
//}

@fragment
fn fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let frag_coord = position.xy;
    let uv = (frag_coord / screen_resolution) * 2.0 - 1.0;

    let origin = camera.position;
    let dir = normalize(camera.forward + uv.x * camera.right + uv.y * camera.up);

    let start = origin + dir;
    // Raymarch inside bounds
    let t = raymarch(start, dir);
    if (t < 0.0) {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    let p = origin + dir * t;
    let n = normalize(vec3<f32>(
        voxel_lookup(p + vec3<f32>(EPSILON, 0, 0)) - voxel_lookup(p - vec3<f32>(EPSILON, 0, 0)),
        voxel_lookup(p + vec3<f32>(0, EPSILON, 0)) - voxel_lookup(p - vec3<f32>(0, EPSILON, 0)),
        voxel_lookup(p + vec3<f32>(0, 0, EPSILON)) - voxel_lookup(p - vec3<f32>(0, 0, EPSILON))
    ));
    return vec4<f32>(0.5 * (n + vec3<f32>(1.0)), 1.0);
}
