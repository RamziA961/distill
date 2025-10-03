
struct Box3 {
    min: vec3<f32>,
    max: vec3<f32>,
}

struct Camera {
    position: vec3<f32>,
    forward: vec3<f32>,
    right: vec3<f32>,
    up: vec3<f32>,
}

struct RayHit {
    t_min: f32, // distance along ray to entry point
    t_max: f32, // distance along ray to exit point
    hit: bool,
}
