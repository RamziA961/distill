
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

struct RayHit {
    t_min: f32, // distance along ray to entry point
    t_max: f32, // distance along ray to exit point
    hit: bool,
}
