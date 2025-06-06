use fruits_math::{Mat3, Vec3};

use crate::collision::{equations::{self, QuadraticEquationResult}, CollisionAabb, CollisionBox, CollisionLine, CollisionShape, CollisionSphere};

pub fn overlaps(lhs: &CollisionShape, rhs: &CollisionShape) -> bool {
    match (lhs, rhs) {
        (CollisionShape::Point(lhs), CollisionShape::Point(rhs)) => overlaps_pt_pt(lhs, rhs),
        (CollisionShape::Point(lhs), CollisionShape::Line(rhs)) => overlaps_pt_ln(lhs, rhs),
        (CollisionShape::Point(lhs), CollisionShape::Aabb(rhs)) => overlaps_pt_aa(lhs, rhs),
        (CollisionShape::Point(lhs), CollisionShape::Box(rhs)) => overlaps_pt_bx(lhs, rhs),
        (CollisionShape::Point(lhs), CollisionShape::Sphere(rhs)) => overlaps_pt_sp(lhs, rhs),
        (CollisionShape::Point(lhs), CollisionShape::Triangle(rhs)) => overlaps_pt_tr(lhs, rhs),
        
        (CollisionShape::Line(lhs), CollisionShape::Point(rhs)) => overlaps_pt_ln(rhs, lhs),
        (CollisionShape::Line(lhs), CollisionShape::Line(rhs)) => overlaps_ln_ln(lhs, rhs),
        (CollisionShape::Line(lhs), CollisionShape::Aabb(rhs)) => overlaps_ln_aa(lhs, rhs),
        (CollisionShape::Line(lhs), CollisionShape::Box(rhs)) => overlaps_ln_bx(lhs, rhs),
        (CollisionShape::Line(lhs), CollisionShape::Sphere(rhs)) => overlaps_ln_sp(lhs, rhs),
        (CollisionShape::Line(lhs), CollisionShape::Triangle(rhs)) => overlaps_ln_tr(lhs, rhs),
        
        (CollisionShape::Aabb(lhs), CollisionShape::Point(rhs)) => overlaps_pt_aa(rhs, lhs),
        (CollisionShape::Aabb(lhs), CollisionShape::Line(rhs)) => overlaps_ln_aa(rhs, lhs),
        (CollisionShape::Aabb(lhs), CollisionShape::Aabb(rhs)) => overlaps_aa_aa(lhs, rhs),
        (CollisionShape::Aabb(lhs), CollisionShape::Box(rhs)) => overlaps_aa_bx(lhs, rhs),
        (CollisionShape::Aabb(lhs), CollisionShape::Sphere(rhs)) => overlaps_aa_sp(lhs, rhs),
        (CollisionShape::Aabb(lhs), CollisionShape::Triangle(rhs)) => overlaps_aa_tr(lhs, rhs),
        
        (CollisionShape::Box(lhs), CollisionShape::Point(rhs)) => overlaps_pt_bx(rhs, lhs),
        (CollisionShape::Box(lhs), CollisionShape::Line(rhs)) => overlaps_ln_bx(rhs, lhs),
        (CollisionShape::Box(lhs), CollisionShape::Aabb(rhs)) => overlaps_aa_bx(rhs, lhs),
        (CollisionShape::Box(lhs), CollisionShape::Box(rhs)) => overlaps_bx_bx(rhs, lhs),
        (CollisionShape::Box(lhs), CollisionShape::Sphere(rhs)) => overlaps_bx_sp(lhs, rhs),
        (CollisionShape::Box(lhs), CollisionShape::Triangle(rhs)) => overlaps_bx_tr(lhs, rhs),
        
        (CollisionShape::Sphere(lhs), CollisionShape::Point(rhs)) => overlaps_pt_sp(rhs, lhs),
        (CollisionShape::Sphere(lhs), CollisionShape::Line(rhs)) => overlaps_ln_sp(rhs, lhs),
        (CollisionShape::Sphere(lhs), CollisionShape::Aabb(rhs)) => overlaps_aa_sp(rhs, lhs),
        (CollisionShape::Sphere(lhs), CollisionShape::Box(rhs)) => overlaps_bx_sp(rhs, lhs),
        // todo: Sphere, Triangle
        (CollisionShape::Sphere(lhs), CollisionShape::Sphere(rhs)) => overlaps_sp_sp(lhs, rhs),
        
        (CollisionShape::Triangle(lhs), CollisionShape::Point(rhs)) => overlaps_pt_tr(rhs, lhs),
        (CollisionShape::Triangle(lhs), CollisionShape::Line(rhs)) => overlaps_ln_tr(rhs, lhs),
        (CollisionShape::Triangle(lhs), CollisionShape::Aabb(rhs)) => overlaps_aa_tr(rhs, lhs),
        (CollisionShape::Triangle(lhs), CollisionShape::Box(rhs)) => overlaps_bx_tr(rhs, lhs),
        // todo: Triangle, Sphere
        // todo: Triangle, Triangle
        
        
        // todo: remove
        _ => todo!(),
    }
}

fn overlaps_pt_pt(lhs: &Vec3<f32>, rhs: &Vec3<f32>) -> bool {
    lhs == rhs
}

fn overlaps_pt_ln(lhs: &Vec3<f32>, rhs: &CollisionLine) -> bool {
    if lhs == &rhs.start {
        return true;
    }
    
    if lhs == &rhs.end {
        return true;
    }
    
    if rhs.start == rhs.end {
        return false;
    }

    let vector = rhs.end - rhs.start;

    let abs_vector = vector.map(f32::abs);

    let progress = if abs_vector.x > abs_vector.y && abs_vector.x > abs_vector.z {
        fruits_math::inv_lerp(rhs.start.x, rhs.end.x, lhs.x)
    } else if abs_vector.y > abs_vector.z {
        fruits_math::inv_lerp(rhs.start.y, rhs.end.y, lhs.y)
    } else {
        fruits_math::inv_lerp(rhs.start.z, rhs.end.z, lhs.z)
    };

    if rhs.bounds.is_start_restricted() && progress < 0.0 {
        return false;
    }

    if rhs.bounds.is_end_restricted() && progress > 1.0 {
        return false;
    }

    let projected_point = Vec3::lerp(&rhs.start, rhs.end, progress);
    
    lhs == &projected_point
}

fn overlaps_ln_ln(s1: &CollisionLine, s2: &CollisionLine) -> bool {
    let p1 = s1.start;
    let p2 = s1.end;
    let p3 = s2.start;
    let p4 = s2.end;

    if p1 == p2 {
        return overlaps_pt_ln(&p1, s2);
    }
    if p3 == p4 {
        return overlaps_pt_ln(&p3, s1);
    }

    let progress1 =
        (
            (p1 - p3).dot(&(p4 - p3)) * (p4 - p3).dot(&(p2 - p1)) 
            - (p1 - p3).dot(&(p2 - p1)) * (p4 - p3).dot(&(p4 - p3))
        )
        / (
            (p2 - p1).dot(&(p2 - p1)) * (p4 - p3).dot(&(p4 - p3))
            - (p4 - p3).dot(&(p2 - p1)) * (p4 - p3).dot(&(p2 - p1))
        );


    if s1.bounds.is_start_restricted() && progress1 < 0.0 {
        return false;
    }

    if s1.bounds.is_end_restricted() && progress1 > 1.0 {
        return false;
    }

    let progress2 = ((p1 - p3).dot(&(p4 - p3)) + progress1 * (p4 - p3).dot(&(p2 - p1))) / (p4 - p3).dot(&(p4 - p3));
    

    if s2.bounds.is_start_restricted() && progress2 < 0.0 {
        return false;
    }

    if s2.bounds.is_end_restricted() && progress2 > 1.0 {
        return false;
    }
    
    let closest1 = Vec3::lerp(&s1.start, s1.end, progress1);
    let closest2 = Vec3::lerp(&s2.start, s2.end, progress2);

    closest1 == closest2
}

fn overlaps_pt_sp(s1: &Vec3<f32>, s2: &CollisionSphere) -> bool {
    let distance_vector = *s1 - s2.center;

    distance_vector.length_sq() <= s2.radius * s2.radius
}

fn overlaps_pt_tr(s1: &Vec3<f32>, s2: &[Vec3<f32>; 3]) -> bool {
    let [a, b, c] = *s2;

    // Triangle edges
    let v0 = c - a;
    let v1 = b - a;
    let v2 = *s1 - a;

    // Compute dot products
    let dot00 = v0.dot(&v0);
    let dot01 = v0.dot(&v1);
    let dot02 = v0.dot(&v2);
    let dot11 = v1.dot(&v1);
    let dot12 = v1.dot(&v2);

    // Compute barycentric coordinates
    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < f32::EPSILON {
        return false; // Degenerate triangle
    }

    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    // Check if point is in triangle
    return u >= 0.0 && v >= 0.0 && (u + v) <= 1.0 && is_point_on_triangle_plane(a, b, c, *s1);

    fn is_point_on_triangle_plane(a: Vec3<f32>, b: Vec3<f32>, c: Vec3<f32>, point: Vec3<f32>) -> bool {
        let normal = (b - a).cross(c - a).normalized();
        let distance = (point - a).dot(&normal);
        distance.abs() < 1e-5  // epsilon tolerance
    }
}

fn overlaps_ln_sp(s1: &CollisionLine, s2: &CollisionSphere) -> bool {
    let d = s1.end - s1.start;
    let v = s2.center - s1.start;
    let mut t = v.dot(&d) / d.dot(&d);

    if s1.bounds.is_start_restricted() {
        t = t.max(0.0);
    }

    if s1.bounds.is_end_restricted() {
        t = t.min(1.0);
    }

    let closest_point = Vec3::lerp(&s1.start, s1.end, t);

    (closest_point - s2.center).length_sq() <= s2.radius * s2.radius
}

fn overlaps_bx_bx(s1: &CollisionBox, s2: &CollisionBox) -> bool {
    let rot_inv = s1.rotation.to_matrix().inverse().unwrap();

    overlaps_centered_aa_bx(
        &(s1.scale / 2.0),
        &(rot_inv * (s2.center - s1.center)),
        &s2.scale,
        &(rot_inv * s2.rotation.to_matrix()),
    )
}

fn overlaps_bx_sp(s1: &CollisionBox, s2: &CollisionSphere) -> bool {
    let rot_inv = s1.rotation.to_matrix().inverse().unwrap();

    let s2 = CollisionSphere {
        radius: s2.radius,
        center: (rot_inv * (s2.center - s1.center))
    };

    overlaps_centered_aa_sp(
        &(s1.scale / 2.0),
        &s2,
    )
}

fn overlaps_bx_tr(s1: &CollisionBox, s2: &[Vec3<f32>; 3]) -> bool {
    let rot_inv = s1.rotation.to_matrix().inverse().unwrap();

    let s2 = s2.map(|v| rot_inv * (v - s1.center));

    overlaps_centered_aa_tr(
        &(s1.scale / 2.0),
        &s2,
    )
}

fn overlaps_sp_sp(s1: &CollisionSphere, s2: &CollisionSphere) -> bool {
    let r_sum = s1.radius + s2.radius;
    
    (s1.center - s2.center).length_sq() <= r_sum * r_sum
}

fn overlaps_pt_aa(s1: &Vec3<f32>, s2: &CollisionAabb) -> bool {
    overlaps_centered_aa_pt(&(s2.scale / 2.0), &(*s1 - s2.center))
}

fn overlaps_pt_bx(s1: &Vec3<f32>, s2: &CollisionBox) -> bool {
    let s1 = s2.rotation.to_matrix().inverse().unwrap() * (*s1 - s2.center);

    let extents = s2.scale / 2.0;

    let mut result = true;

    result &= s1.x.abs() <= extents.x;
    result &= s1.y.abs() <= extents.y;
    result &= s1.z.abs() <= extents.z;

    result
}

fn overlaps_ln_aa(s1: &CollisionLine, s2: &CollisionAabb) -> bool {
    overlaps_centered_aa_ln(
        &(s2.scale / 2.0),
        &CollisionLine {
            start: s1.start - s2.center,
            end: s1.end - s2.center,
            bounds: s1.bounds,
        },
    )
}

fn overlaps_ln_bx(s1: &CollisionLine, s2: &CollisionBox) -> bool {
    let s1 = CollisionLine {
        start: s2.rotation.to_matrix().inverse().unwrap() * (s1.start - s2.center),
        end: s2.rotation.to_matrix().inverse().unwrap() * (s1.end - s2.center),
        bounds: s1.bounds,
    };

    let extents = s2.scale / 2.0;

    let mut t_min = 0.0;
    let mut t_max = 1.0;

    if !s1.bounds.is_start_restricted() {
        t_min = f32::NEG_INFINITY;
    }

    if !s1.bounds.is_end_restricted() {
        t_max = f32::INFINITY;
    }

    for i in 0..3 {
        let x1 = s1.start[i];
        let x2 = s1.end[i];
        let x_min = -extents[i];
        let x_max = extents[i];
        
        if x2 != x1 {
            let mut t1 = (x_min - x1) / (x2 - x1);
            let mut t2 = (x_max - x1) / (x2 - x1);
            
            if t1 > t2 {
                (t1, t2) = (t2, t1);
            }

            t_min = t_min.max(t1);
            t_max = t_max.min(t2);
        }
        else if x1 < x_min || x1 > x_max {
            return false;
        }
    }

    t_min <= t_max
}

fn overlaps_ln_tr(s1: &CollisionLine, s2: &[Vec3<f32>; 3]) -> bool {
    let o = s1.start;
    let d = s1.end - s1.start;

    let a = s2[0];
    let b = s2[1];
    let c = s2[2];

    let e1 = b - a;
    let e2 = c - a;
    
    let n = e1.cross(e2);
    let det = -d.dot(&n);
    let inv_det = 1.0 / det;
    let ao = o - a;
    let dao = ao.cross(d);

    let u = e2.dot(&dao) * inv_det;
    let v = -e1.dot(&dao) * inv_det;
    let t = ao.dot(&n) * inv_det;

    (det.abs() >= 1e-6) && (t >= 0.0) && (t <= 1.0) && (u >= 0.0) && (v >= 0.0) && ((u + v) <= 1.0)
}

fn overlaps_aa_aa(s1: &CollisionAabb, s2: &CollisionAabb) -> bool {
    let s1_ext = s1.scale / 2.0;
    let s2_ext = s2.scale / 2.0;

    let s1_min = s1.center - s1_ext;
    let s1_max = s1.center + s1_ext;

    let s2_min = s2.center - s2_ext;
    let s2_max = s2.center + s2_ext;

    let min = s1_min.zip(&s2_min, f32::max);
    let max = s1_max.zip(&s2_max, f32::min);

    min.zip(&max, |a, b| a <= b).all()
}

fn overlaps_aa_bx(s1: &CollisionAabb, s2: &CollisionBox) -> bool {
    overlaps_centered_aa_bx(
        &(s1.scale * 0.5),
        &(s2.center - s1.center),
        &(s2.scale / 2.0),
        &s2.rotation.to_matrix(),
    )
}

fn overlaps_aa_sp(s1: &CollisionAabb, s2: &CollisionSphere) -> bool {
    let mut s2 = *s2;
    s2.center -= s1.center;
    overlaps_centered_aa_sp(&(s1.scale * 0.5), &s2)
}

fn overlaps_aa_tr(s1: &CollisionAabb, s2: &[Vec3<f32>; 3]) -> bool {
    overlaps_centered_aa_tr(&(s1.scale * 0.5), &s2.map(|v| v - s1.center))
}

/// <summary>
/// Will be used later. Has 50% worse performance than the non-alt version. But can provide contact points.
/// </summary>
/// <returns></returns>
fn _overlaps_ls_alt(s1: &CollisionLine, s2: &CollisionSphere) -> bool {
    if overlaps_pt_sp(&s1.start, s2) || overlaps_pt_sp(&s1.end, s2) {
        return true;
    }

    let [ax, bx, cx] = get_axis_equation_params(s1.start.x, s1.end.x, s2.center.x);
    let [ay, by, cy] = get_axis_equation_params(s1.start.y, s1.end.y, s2.center.y);
    let [az, bz, cz] = get_axis_equation_params(s1.start.z, s1.end.z, s2.center.z);
    
    let a = ax + ay + az;
    let b = bx + by + bz;
    let c = cx + cy + cz - s2.radius * s2.radius;

    let equation_result = equations::quadratic(a, b, c);

    return match equation_result {
        QuadraticEquationResult::Single(r) => r >= 0.0 && r <= 1.0,
        QuadraticEquationResult::Double([r1, r2]) => (r1 >= 0.0 && r1 <= 1.0) || (r2 >= 0.0 && r2 <= 1.0),
        _ => false,
    };

    fn get_axis_equation_params(x0: f32, x1: f32, xc: f32) -> [f32; 3] {
        [
            (x1 - x0) * (x1 - x0),
            2.0 * (x0 - xc) * (x1 - x0),
            (x0 - xc) * (x0 - xc),
        ]
    }
}

fn overlaps_centered_aa_pt(ext: &Vec3<f32>, pt: &Vec3<f32>) -> bool {
    pt.zip(ext, |p, e| p.abs() <= e).all()
}

fn overlaps_centered_aa_ln(ext: &Vec3<f32>, ln: &CollisionLine) -> bool {
    let mut t_min = 0.0;
    let mut t_max = 1.0;

    if !ln.bounds.is_start_restricted() {
        t_min = f32::NEG_INFINITY;
    }

    if !ln.bounds.is_end_restricted() {
        t_max = f32::INFINITY;
    }

    for i in 0..3 {
        let x1 = ln.start[i];
        let x2 = ln.end[i];
        let x_min = -ext[i];
        let x_max = ext[i];
        
        if x2 != x1 {
            let mut t1 = (x_min - x1) / (x2 - x1);
            let mut t2 = (x_max - x1) / (x2 - x1);
            
            if t1 > t2 {
                (t1, t2) = (t2, t1);
            }

            t_min = t_min.max(t1);
            t_max = t_max.min(t2);
        }
        else if x1 < x_min || x1 > x_max {
            return false;
        }
    }

    t_min <= t_max
}

fn overlaps_centered_aa_bx(ext: &Vec3<f32>, bx_center: &Vec3<f32>, bx_ext: &Vec3<f32>, bx_rot: &Mat3<f32>) -> bool {
    let s2_axes = [
        *bx_rot * Vec3::X,
        *bx_rot * Vec3::Y,
        *bx_rot * Vec3::Z,
    ]; // OBB axes

    let axes = [
        Vec3::X, Vec3::Y, Vec3::Z, // AABB axes
        s2_axes[0], s2_axes[1], s2_axes[2], // OBB axes
        Vec3::X.cross(s2_axes[0]), Vec3::X.cross(s2_axes[1]), Vec3::X.cross(s2_axes[2]), // Cross products of AABB and OBB
        Vec3::Y.cross(s2_axes[0]), Vec3::Y.cross(s2_axes[1]), Vec3::Y.cross(s2_axes[2]),
        Vec3::Z.cross(s2_axes[0]), Vec3::Z.cross(s2_axes[1]), Vec3::Z.cross(s2_axes[2]),
    ];

    for axis in axes.iter().filter(|a| a.length_sq() > 1e-6) {
        let (min_a, max_a) = project_centered_aabb_on_axis(&axis, &ext);
        let (min_b, max_b) = project_box_on_axis(&axis, bx_center, bx_ext, &s2_axes);

        if max_a < min_b || max_b < min_a {
            return false;
        }
    }

    true
}

fn overlaps_centered_aa_sp(ext: &Vec3<f32>, sp: &CollisionSphere) -> bool {
    let closest = sp.center.zip(&-*ext, f32::max).zip(ext, f32::min);

    (closest - sp.center).length_sq() <= sp.radius * sp.radius
}

fn overlaps_centered_aa_tr(ext: &Vec3<f32>, sp: &[Vec3<f32>; 3]) -> bool {
    let [v0, v1, v2] = *sp;

    let f0 = v1 - v0;
    let f1 = v2 - v1;
    let f2 = v0 - v2;

    let axes = [
        Vec3::X, Vec3::Y, Vec3::Z,
        f0.cross(Vec3::X), f0.cross(Vec3::Y), f0.cross(Vec3::Z),
        f1.cross(Vec3::X), f1.cross(Vec3::Y), f1.cross(Vec3::Z),
        f2.cross(Vec3::X), f2.cross(Vec3::Y), f2.cross(Vec3::Z),
        f0.cross(f1),
    ];

    for axis in axes.iter().filter(|a| a.length_sq() > 1e-6) {
        let (min_t, max_t) = project_points_on_axis(axis, sp);
        let (min_b, max_b) = project_centered_aabb_on_axis(axis, ext);

        if max_t < min_b || max_b < min_t {
            return false;
        }
    }

    true
}

//

fn project_points_on_axis(axis: &Vec3<f32>, points: &[Vec3<f32>]) -> (f32, f32) {
    let mut min = axis.dot(&points[0]);
    let mut max = min;

    for p in &points[1..] {
        let proj = axis.dot(p);

        min = min.min(proj);
        max = max.max(proj);
    }

    (min, max)
}

fn project_centered_aabb_on_axis(axis: &Vec3<f32>, half_extents: &Vec3<f32>) -> (f32, f32) {
    let r = half_extents.zip(axis, |e, a| e * a.abs()).sum();

    (-r, r)
}

fn project_box_on_axis(axis: &Vec3<f32>, center: &Vec3<f32>, half_extents: &Vec3<f32>, axes: &[Vec3<f32>; 3]) -> (f32, f32) {
    let c = center.dot(axis);
    let r = half_extents.x * axis.dot(&axes[0]).abs()
          + half_extents.y * axis.dot(&axes[1]).abs()
          + half_extents.z * axis.dot(&axes[2]).abs();
    (c - r, c + r)
}