#import "shaders/common_types.wgsl"::{Box3, RayHit}

// A small epsilon to prevent floating-point division by zero issues.
const EPSILON: f32 = 0.00001;

/// Ray-triangle intersection using the Möller–Trumbore algorithm.
fn ray_triangle_intersect(origin: vec3<f32>, direction: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> RayHit {
    let edge1 = b - a;
    let edge2 = c - a;

    // Calculate determinant
    let pvec = cross(direction, edge2);
    let det = dot(edge1, pvec);

    // If determinant is close to zero, ray lies in plane of triangle or is parallel
    if (abs(det) < EPSILON) {
        return RayHit(0.0, 0.0, false);
    }

    let inv_det = 1.0 / det;

    // Distance from vertex a to ray origin
    let tvec = origin - a;

    // Compute barycentric coordinate u
    let u = dot(tvec, pvec) * inv_det;
    if (u < 0.0 || u > 1.0) {
        return RayHit(0.0, 0.0, false);
    }

    // Compute barycentric coordinate v
    let qvec = cross(tvec, edge1);
    let v = dot(direction, qvec) * inv_det;
    if (v < 0.0 || u + v > 1.0) {
        return RayHit(0.0, 0.0, false);
    }

    // Compute intersection distance along the ray
    let t = dot(edge2, qvec) * inv_det;
    if (t < EPSILON) {
        // Intersection is behind the ray origin
        return RayHit(0.0, 0.0, false);
    }

    return RayHit(t, t, true);
}

/// Ray-AABB intersection using the slab method (vectorized min/max approach).
///
/// This function computes whether a ray intersects an axis-aligned bounding box (AABB)
/// and, if so, the distances along the ray where it enters and exits the box.
fn ray_aabb_intersect(origin: vec3<f32>, dir: vec3<f32>, box: Box3) -> RayHit {
    // Calculate t values for intersection with slabs
    let t1 = (box.min - origin) / dir;
    let t2 = (box.max - origin) / dir;

    // Calculate per-axis near and far intersection t-values
    let t_near = min(t1, t2);
    let t_far = max(t1, t2);

    // Calculate overall entry and exit t-values
    // We use a specific t_min_ray_start (e.g., 0.0) if the ray is a segment
    let t_min_ray_start = 0.0; 
    var tmin = max(max(t_near.x, t_near.y), t_near.z);
    var tmax = min(min(t_far.x, t_far.y), t_far.z);

    // Ensure the intersection starts at or after the ray's origin (t >= 0.0)
    tmin = max(tmin, t_min_ray_start);

    // Ray intersects if tmin is less than tmax
    let hit = (tmin <= tmax);

    return RayHit(tmin, tmax, hit);
}

// Utility function to calculate triangle normal (assuming consistent winding)
fn calculate_triangle_normal(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    return normalize(cross(b - a, c - a));
}
