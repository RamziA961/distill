#import "shaders/common_types.wgsl"::{Box3, RayHit}

// A small epsilon to prevent floating-point division by zero issues.
const EPSILON: f32 = 0.00001;

// Computes the intersection of a ray with a triangle using the Möller-Trumbore algorithm.
// Returns `true` if an intersection occurs, `false` otherwise.
// Note that this does not return the distance, as it's not needed for the sign determination pass.
fn ray_triangle_intersect(origin: vec3<f32>, direction: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> bool {
    let edge1 = b - a;
    let edge2 = c - a;

    // Calculate the determinant.
    // This is a triple product: dot(cross(dir, edge2), edge1).
    let pvec = cross(direction, edge2);
    let det = dot(edge1, pvec);

    // If the determinant is close to zero, the ray lies in the same plane as the triangle.
    if (abs(det) < EPSILON) {
        return false;
    }

    let inv_det = 1.0 / det;

    // Calculate the distance from v0 to the ray origin.
    let tvec = origin - a;

    // Calculate u parameter and test bounds.
    let u = dot(tvec, pvec) * inv_det;
    if (u < 0.0 || u > 1.0) {
        return false;
    }

    // Calculate v parameter and test bounds.
    let qvec = cross(tvec, edge1);
    let v = dot(direction, qvec) * inv_det;
    if (v < 0.0 || u + v > 1.0) {
        return false;
    }

    // At this point, we know the ray intersects the triangle.
    // We can also compute the distance to the intersection point, 't'.
    // let t = dot(edge2, qvec) * inv_det;
    // if (t < EPSILON) {
    //     // Intersection behind the ray origin.
    //     return false;
    // }

    return true;
}

// Ray-AABB intersection using slab method
fn ray_aabb_intersect(origin: vec3<f32>, dir: vec3<f32>, min: vec3<f32>, max: vec3<f32>) -> bool {
    let inv_dir = 1.0 / dir;

    var tmin = (min.x - origin.x) * inv_dir.x;
    var tmax = (max.x - origin.x) * inv_dir.x;

    if (tmin > tmax) { 
        let tmp = tmin; 
        tmin = tmax; 
        tmax = tmp; 
    }

    var tymin = (min.y - origin.y) * inv_dir.y;
    var tymax = (max.y - origin.y) * inv_dir.y;
    if (tymin > tymax) { 
        let tmp = tymin; 
        tymin = tymax; 
        tymax = tmp; 
    }

    if ((tmin > tymax) || (tymin > tmax)) { 
        return false; 
    }

    tmin = max(tmin, tymin);
    tmax = min(tmax, tymax);

    var tzmin = (min.z - origin.z) * inv_dir.z;
    var tzmax = (max.z - origin.z) * inv_dir.z;
    if (tzmin > tzmax) { 
        let tmp = tzmin; 
        tzmin = tzmax; 
        tzmax = tmp; 
    }

    if ((tmin > tzmax) || (tzmin > tmax)) { 
        return false; 
    }

    return true;
}

// Utility function to calculate triangle normal (assuming consistent winding)
fn calculate_triangle_normal(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    return normalize(cross(b - a, c - a));
}

fn intersect_box(origin: vec3<f32>, dir: vec3<f32>, box: Box3) -> RayHit {
    var t1 = (box.min - origin) / dir;
    var t2 = (box.max - origin) / dir;

    // entry = max of mins, exit = min of maxs
    let tmin = max(max(min(t1.x, t2.x), min(t1.y, t2.y)), min(t1.z, t2.z));
    let tmax = min(min(max(t1.x, t2.x), max(t1.y, t2.y)), max(t1.z, t2.z));

    let hit = (tmax >= max(tmin, 0.0));
    return RayHit(tmin, tmax, hit);
}
