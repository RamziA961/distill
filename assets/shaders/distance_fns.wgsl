struct ClosestPointOnTriangleResult {
    point: vec3<f32>,
    barycentric: vec3<f32>, // (u, v, w)
};

fn closest_point_on_triangle(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> ClosestPointOnTriangleResult {
let ab = b - a;
    let ac = c - a;
    let ap = p - a;

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if (d1 <= 0.0 && d2 <= 0.0) {
        return ClosestPointOnTriangleResult(a, vec3<f32>(1.0, 0.0, 0.0));
    }

    let bp = p - b;
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if (d3 >= 0.0 && d4 <= d3) {
        return ClosestPointOnTriangleResult(b, vec3<f32>(0.0, 1.0, 0.0));
    }

    let vc = d1 * d4 - d3 * d2;
    if (vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0) {
        let v = d1 / (d1 - d3);
        return ClosestPointOnTriangleResult(a + v * ab, vec3<f32>(1.0 - v, v, 0.0));
    }

    let cp = p - c;
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if (d6 >= 0.0 && d5 <= d6) {
        return ClosestPointOnTriangleResult(c, vec3<f32>(0.0, 0.0, 1.0));
    }

    let vb = d5 * d2 - d1 * d6;
    if (vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0) {
        let w = d2 / (d2 - d6);
        return ClosestPointOnTriangleResult(a + w * ac, vec3<f32>(1.0 - w, 0.0, w));
    }

    let va = d3 * d6 - d5 * d4;
    if (va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0) {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return ClosestPointOnTriangleResult(b + w * (c - b), vec3<f32>(0.0, 1.0 - w, w));
    }

    // Inside face region
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let u = 1.0 - v - w;
    return ClosestPointOnTriangleResult(a + ab * v + ac * w, vec3<f32>(u, v, w));
}

fn distance_to_aabb(pt: vec3<f32>, minv: vec3<f32>, maxv: vec3<f32>) -> f32 {
    let d = max(max(minv - pt, vec3<f32>(0.0)), pt - maxv);
    return length(d);
}
