mod colors;
mod equations;
mod mat;
mod mat2;
mod mat3;
mod mat4;
mod num;
mod quat;
mod vec;

pub use colors::*;
pub use equations::*;
pub use mat::*;
pub use mat2::*;
pub use mat3::*;
pub use mat4::*;
pub use num::*;
pub use quat::*;
pub use vec::*;

pub fn into_matrix4x4_with_pos<T: Primitive>(mat: Mat3<T>, pos: Vec3<T>) -> Mat4<T> {
    Mat4::from_array([
        [mat[0][0], mat[0][1], mat[0][2], T::ZERO],
        [mat[1][0], mat[1][1], mat[1][2], T::ZERO],
        [mat[2][0], mat[2][1], mat[2][2], T::ZERO],
        [pos.x, pos.y, pos.z, T::ONE],
    ])
}

pub fn perspective_proj_matrix(fov: f32, near: f32, far: f32, aspect: f32) -> Mat4<f32> {
    // todo
    // let s = -1_f32 / ((fov / 2_f32).tan());

    // Mat4::<f32>::from_array([
    //     [s, 0_f32, 0_f32, 0_f32],
    //     [0_f32, s, 0_f32, 0_f32],
    //     [0_f32, 0_f32, (-far / (far - near)), 1_f32],
    //     [0_f32, 0_f32, ((-far * near) / (far - near)), 0_f32],
    // ])

    let s = 1_f32 / ((fov / 2_f32).tan());

    Mat4::<f32>::from_array([
        [(s / aspect), 0_f32, 0_f32, 0_f32],
        [0_f32, s, 0_f32, 0_f32],
        [0_f32, 0_f32, (far / (far - near)), 1_f32],
        [0_f32, 0_f32, ((-far * near) / (far - near)), 0_f32],
    ])
}

pub fn lerp<T: Number>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

pub fn inv_lerp<T: Number>(a: T, b: T, x: T) -> T {
    (x - a) / (b - a)
}

pub fn damp(
    current: f32,
    target: f32,
    velocity: &mut f32,
    smooth_time: f32,
    damping: f32,
    delta_time: f32,
) -> f32 {
    let smooth_time = smooth_time.max(1e-4);
    let omega = 2.0 / smooth_time;

    // Let the damping be fully free (negative included!)
    let zeta = damping;

    // Solve as usual for displacement from target
    let x0 = current - target;
    let v0 = *velocity;
    let t = delta_time;

    let mut new_x;
    let mut new_v;

    if zeta < 1.0 {
        // Underdamped **OR negative damping**
        //
        // For negative ζ, the exponent becomes POSITIVE
        // and the oscillation amplitude grows each frame.
        let omega_d_sq = 1.0 - zeta * zeta;
        let omega_d = if omega_d_sq > 0.0 {
            omega * omega_d_sq.sqrt()
        } else {
            // If ζ < -1, we switch into real exponent regime
            // (it will be handled below)
            0.0
        };

        if omega_d_sq > 0.0 {
            // Underdamped branch (ζ < 1)
            let exp = (-zeta * omega * t).exp();

            let c1 = x0;
            let c2 = (v0 + zeta * omega * x0) / omega_d;

            let cos = (omega_d * t).cos();
            let sin = (omega_d * t).sin();

            new_x = exp * (c1 * cos + c2 * sin);
            new_v = exp * (-zeta * omega * (c1 * cos + c2 * sin)
                + (-c1 * omega_d * sin + c2 * omega_d * cos));
        } else {
            // ζ <= -1: real exponential instability (no oscillation)
            let tmp = (zeta * zeta - 1.0).sqrt();
            let r1 = -omega * (zeta - tmp);
            let r2 = -omega * (zeta + tmp);

            let denom = r1 - r2;
            let c1 = (v0 - r2 * x0) / denom;
            let c2 = x0 - c1;

            let e1 = (r1 * t).exp();
            let e2 = (r2 * t).exp();

            new_x = c1 * e1 + c2 * e2;
            new_v = c1 * r1 * e1 + c2 * r2 * e2;
        }
    } else if (zeta - 1.0).abs() < 1e-6 {
        // Critical damping
        let exp = (-omega * t).exp();
        let c1 = x0;
        let c2 = v0 + omega * x0;

        new_x = (c1 + c2 * t) * exp;
        new_v = exp * (c2 - omega * (c1 + c2 * t));
    } else {
        // Overdamped
        let tmp = (zeta * zeta - 1.0).sqrt();
        let r1 = -omega * (zeta - tmp);
        let r2 = -omega * (zeta + tmp);

        let denom = r1 - r2;
        let c1 = (v0 - r2 * x0) / denom;
        let c2 = x0 - c1;

        let e1 = (r1 * t).exp();
        let e2 = (r2 * t).exp();

        new_x = c1 * e1 + c2 * e2;
        new_v = c1 * r1 * e1 + c2 * r2 * e2;
    }

    *velocity = new_v;
    target + new_x
}