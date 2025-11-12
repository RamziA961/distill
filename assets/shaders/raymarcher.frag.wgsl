#import "shaders/common_types.wgsl"::{Box3, Camera}
#import "shaders/util_fns.wgsl"::ray_aabb_intersect

@group(2) @binding(0)
var voxel_texture: texture_3d<f32>;

@group(2) @binding(1)
var voxel_sampler: sampler;

@group(2) @binding(2)
var<uniform> camera: Camera;

@group(2) @binding(3)
var<uniform> grid_size: u32;

@group(2) @binding(4)
var<uniform> grid_bounds: Box3;

const OUT_OF_BOUNDS_DIST: f32 = 1e30;
const EPSILON: f32 = 0.5;

// Sample 3D SDF using hardware interpolation and mipmaps
fn voxel_lookup(p: vec3<f32>, mip: f32) -> f32 {
    let extent = grid_bounds.max - grid_bounds.min;
    let rel = (p - grid_bounds.min) / extent;

    return select(
        textureSampleLevel(voxel_texture, voxel_sampler, rel, mip).r,
        OUT_OF_BOUNDS_DIST,
        any(rel < vec3<f32>(0.0)) || any(rel > vec3<f32>(1.0))
    );
}

// Compute dynamic max steps based on ray length and voxel size
fn compute_max_steps(dir: vec3<f32>, voxel_size: f32, max_dist: f32) -> u32 {
    // Estimate the number of steps needed across the box
    let ideal_steps = max_dist / voxel_size;
    return clamp(u32(ideal_steps), 32u, 256u);
}

// Raymarch loop with adaptive stepping and mipmap LOD heuristic
fn raymarch(origin: vec3<f32>, dir: vec3<f32>, voxel_size: f32, max_dist: f32) -> f32 {
    var t = 0.0;

    // Sample at start point
    var p = origin;
    var d = voxel_lookup(p, 0.0);

    let max_steps = compute_max_steps(dir, voxel_size, max_dist);

    for (var i = 0u; i < max_steps; i++) {
        if (t > max_dist) {
            break;
        }

        if (d >= OUT_OF_BOUNDS_DIST * 0.1) {
            // skip large empty space faster
            t += voxel_size * 2.0;
            p = origin + dir * t;
            d = voxel_lookup(p, 0.0);
            continue;
        }

        if (d < voxel_size * EPSILON) {
            // Hit detected – refine using binary search for precision
            var t_back = t - voxel_size;
            for (var j = 0u; j < 3u; j++) {
                let t_mid = 0.5 * (t + t_back);
                let p_mid = origin + dir * t_mid;
                let d_mid = voxel_lookup(p_mid, 0.0);
                if (d_mid < 0.0) {
                    t = t_mid;
                } else {
                    t_back = t_mid;
                }
            }
            return t;
        }

        // Adaptive mip-level: coarser far away, finer near surface
        let mip = clamp(log2(d / voxel_size), 0.0, 4.0);

        // Adaptive step size: larger far away, smaller near surface
        let step = max(d, voxel_size * 0.5);

        // Precompute next sample point and next distance (fused iteration)
        t += step;
        p = origin + dir * t;
        d = voxel_lookup(p, mip);
    }

    return -1.0;
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
    let result = ray_aabb_intersect(camera_position, dir, grid_bounds);
    if (!result.hit) {
        return vec4<f32>(0.0); // background
    }
    
    // Start and end distances inside the box
    let start_t = max(result.tmin, 0.0);
    let end_t = result.tmax;
    let max_dist = end_t - start_t;

    // Start point inside bounds
    let start = camera_position + dir * start_t;

    let voxel_size = max(
        max(
            (grid_bounds.max - grid_bounds.min).x,
            (grid_bounds.max - grid_bounds.min).y
        ),
        (grid_bounds.max - grid_bounds.min).z
    ) / f32(grid_size);

    let t = raymarch(start, dir, voxel_size, max_dist);
    if (t < 0.0) {
        return vec4<f32>(0.0); // miss inside cube
    }

    let eps = voxel_size * 0.5;

    let p = start + dir * t;
    // Compute normal by fast central difference
    let n = normalize(vec3<f32>(
        voxel_lookup(p + vec3<f32>(eps, 0.0, 0.0), 0.0) - voxel_lookup(p - vec3<f32>(eps, 0.0, 0.0), 0.0),
        voxel_lookup(p + vec3<f32>(0.0, eps, 0.0), 0.0) - voxel_lookup(p - vec3<f32>(0.0, eps, 0.0), 0.0),
        voxel_lookup(p + vec3<f32>(0.0, 0.0, eps), 0.0) - voxel_lookup(p - vec3<f32>(0.0, 0.0, eps), 0.0)
    ));

    return vec4<f32>(n, 1.0);
}

