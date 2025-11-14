#import "shaders/common_types.wgsl"::{Box3, Camera}
#import "shaders/util_fns.wgsl"::ray_aabb_intersect

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}


#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(2) @binding(100)
var voxel_texture: texture_3d<f32>;

@group(2) @binding(101)
var voxel_sampler: sampler;

@group(2) @binding(102)
var<uniform> camera: Camera;

@group(2) @binding(103)
var<uniform> grid_size: u32;

@group(2) @binding(104)
var<uniform> grid_bounds: Box3;

@group(2) @binding(105)
var<uniform> local_from_world: mat4x4<f32>;

@group(2) @binding(106)
var<uniform> world_from_local: mat4x4<f32>;

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
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color); 

    // Raymarching logic
    let camera_world = camera.inv_view_mat[3].xyz;
    let dir_world = normalize(in.world_position.xyz - camera_world);

    // Transform ray into local sdf space
    let origin_local = (local_from_world * vec4<f32>(camera_world, 1.0)).xyz;
    let dir_local = normalize((local_from_world * vec4<f32>(dir_world, 0.0)).xyz);

    // Intersect with the cube bounding the SDF
    let result = ray_aabb_intersect(origin_local, dir_local, grid_bounds);
    if (!result.hit) {
        // Return background if the ray misses the volume
#ifdef PREPASS_PIPELINE
        return deferred_output(in, pbr_input);
#else
        var out: FragmentOutput;
        out.color = vec4<f32>(0.0);
        return out;
#endif
    }
    
    // Start and end distances inside the box
    let start_t = max(result.tmin, 0.0);
    let end_t = result.tmax;
    let max_dist = end_t - start_t;


    let start_local = origin_local + dir_local * start_t;

    let extent = grid_bounds.max - grid_bounds.min;
    let max_extent = max(extent.x, max(extent.y, extent.z));
    let voxel_size = max_extent / f32(grid_size);

    let t = raymarch(start_local, dir_local, voxel_size, max_dist);
    if (t < 0.0) {
        // Miss inside bounds
#ifdef PREPASS_PIPELINE
        return deferred_output(in, pbr_input);
#else
        var out: FragmentOutput;
        out.color = vec4<f32>(0.0);
        return out;
#endif
    }

    // Compute surface position and normal
    let eps = voxel_size * 0.5;
    let p_local = start_local + dir_local * t;
    // Compute normal by fast central difference
    let n_local = normalize(vec3<f32>(
        voxel_lookup(p_local + vec3<f32>(eps, 0.0, 0.0), 0.0) - voxel_lookup(p_local - vec3<f32>(eps, 0.0, 0.0), 0.0),
        voxel_lookup(p_local + vec3<f32>(0.0, eps, 0.0), 0.0) - voxel_lookup(p_local - vec3<f32>(0.0, eps, 0.0), 0.0),
        voxel_lookup(p_local + vec3<f32>(0.0, 0.0, eps), 0.0) - voxel_lookup(p_local - vec3<f32>(0.0, 0.0, eps), 0.0)
    ));

    // Back to world space for lighting
    let p_world = (world_from_local * vec4<f32>(p_local, 1.0)).xyz;
    let n_world = normalize((world_from_local * vec4<f32>(n_local, 0.0)).xyz);

    // Update PBR input with raymarched data
    pbr_input.world_position = vec4<f32>(p_world, 1.0);
    pbr_input.world_normal = n_world;
    pbr_input.N = n_world;

    // View direction in world space
    let v_world = normalize(camera_world - p_world);
    pbr_input.V = v_world;

#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // Apply Bevy's lighting
    out.color = apply_pbr_lighting(pbr_input);
    // Post-lighting (fog, tone mapping, etc.)
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}

