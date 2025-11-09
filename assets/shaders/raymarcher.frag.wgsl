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

const OUT_OF_BOUNDS_DIST: f32 = 1e30;
const MAX_STEPS = 128u;

// Trilinear lookup, returns OUT_OF_BOUNDS_DIST if p outside grid
fn voxel_lookup_trilinear(p: vec3<f32>) -> f32 {
    let extent = grid_bounds.max - grid_bounds.min;
    let rel = (p - grid_bounds.min) / extent;

    if (any(rel < vec3<f32>(0.0)) || any(rel > vec3<f32>(1.0))) {
        return OUT_OF_BOUNDS_DIST;
    }

    let uvw = rel;

    // Continuous Index in [0, grid_size-1]
    let fsize = f32(grid_size - 1u);
    let fx = uvw.x * fsize;
    let fy = uvw.y * fsize;
    let fz = uvw.z * fsize;

    let ix = u32(fx);
    let iy = u32(fy);
    let iz = u32(fz);

    let wx = fx - floor(fx);
    let wy = fy - floor(fy);
    let wz = fz - floor(fz);

    // Clamp indices to valid range for sampling corners
    let ix1 = min(ix + 1u, grid_size - 1u);
    let iy1 = min(iy + 1u, grid_size - 1u);
    let iz1 = min(iz + 1u, grid_size - 1u);
    

    // Flattened indices
    let s000 = voxel_texture[ix + iy * grid_size + iz * grid_size * grid_size];
    let s100 = voxel_texture[ix1 + iy * grid_size + iz * grid_size * grid_size];
    let s010 = voxel_texture[ix + iy1 * grid_size + iz * grid_size * grid_size];
    let s110 = voxel_texture[ix1 + iy1 * grid_size + iz * grid_size * grid_size];
    let s001 = voxel_texture[ix + iy * grid_size + iz1 * grid_size * grid_size];
    let s101 = voxel_texture[ix1 + iy * grid_size + iz1 * grid_size * grid_size];
    let s011 = voxel_texture[ix + iy1 * grid_size + iz1 * grid_size * grid_size];
    let s111 = voxel_texture[ix1 + iy1 * grid_size + iz1 * grid_size * grid_size];

    // Trilinear interpolation
    // Lerp in x
    let c00 = mix(s000, s100, wx);
    let c10 = mix(s010, s110, wx);
    let c01 = mix(s001, s101, wx);
    let c11 = mix(s011, s111, wx);

    // Lerp in y
    let c0 = mix(c00, c10, wy);
    let c1 = mix(c01, c11, wy);

    // Lerp in z
    return mix(c0, c1, wz);
}


// Safe lookup with out-of-bounds handling
fn voxel_lookup_safe(p: vec3<f32>) -> f32 {
    let d = voxel_lookup_trilinear(p);
    // NaN defense at lookup level (prevents invalid math early)
    return select(d, OUT_OF_BOUNDS_DIST, d != d);
}

// Raymarch loop
fn raymarch(origin: vec3<f32>, dir: vec3<f32>, eps: f32) -> f32 {
    var t = 0.0;
    let max_dist = length(grid_bounds.max - grid_bounds.min);

    for (var i = 0u; i < MAX_STEPS; i++) {
        let p = origin + dir * t;
        let d = voxel_lookup_safe(p);
    
        // skip invalid or huge distances
        if (d >= OUT_OF_BOUNDS_DIST * 0.1) {
            t += eps * 2.0;
            if (t > max_dist) { 
                break; 
            }
            continue;
        }

        if (d < eps) {
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
        return vec4<f32>(0.0); // background
    }
    
    let voxel_extent = (grid_bounds.max - grid_bounds.min) / f32(grid_size);
    let eps = max(max(voxel_extent.x, voxel_extent.y), voxel_extent.z) * 0.6;

    // Raymarch inside bounds
    let start = camera_position + dir * max(hit.t_min, 0.0);

    let t = raymarch(start, dir, eps);
    if (t < 0.0) {
        return vec4<f32>(0.0); // miss inside cube
    }
    
    // Compute normal by central difference
    let p = start + dir * t;
    let n = normalize(vec3<f32>(
        voxel_lookup_safe(p + vec3<f32>(eps, 0.0, 0.0)) - voxel_lookup_safe(p - vec3<f32>(eps, 0.0, 0.0)),
        voxel_lookup_safe(p + vec3<f32>(0.0, eps, 0.0)) - voxel_lookup_safe(p - vec3<f32>(0.0, eps, 0.0)),
        voxel_lookup_safe(p + vec3<f32>(0.0, 0.0, eps)) - voxel_lookup_safe(p - vec3<f32>(0.0, 0.0, eps))
    ));

    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

