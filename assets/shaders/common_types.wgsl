
struct Triangle {
    a: vec3<f32>,
    b: vec3<f32>,
    c: vec3<f32>,
    na: vec3<f32>,
    nb: vec3<f32>,
    nc: vec3<f32>,
}

struct Box3 {
    min: vec3<f32>,
    max: vec3<f32>,
}

struct Camera {
    view_mat: mat4x4f,
    inv_view_mat: mat4x4f,
    projection_mat: mat4x4f,
    inv_projection_mat: mat4x4f,
}

/// Struct representing the result of a ray-AABB intersection
struct RayHit {
    tmin: f32,   // Distance along the ray to the entry point of the AABB
    tmax: f32,   // Distance along the ray to the exit point of the AABB
    hit: bool,   // True if the ray intersects the AABB, false otherwise
}

struct BvhNode {
    aabb: Box3,
    left_index: u32,
    right_index: u32,
    triangle_count: u32,
}
