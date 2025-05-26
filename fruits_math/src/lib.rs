mod matrix;
mod matrix2x2;
mod matrix3x3;
mod matrix4x4;
mod num;
mod vec;
mod quaternion;

pub use matrix::*;
pub use matrix2x2::*;
pub use matrix3x3::*;
pub use matrix4x4::*;
pub use num::*;
pub use vec::*;
pub use quaternion::*;

pub fn into_matrix4x4_with_pos<T: Number>(mat: Matrix3x3<T>, pos: Vec3<T>) -> Matrix4x4<T> {
    Matrix4x4::from_array(
        [
            [*mat.get_0_0(), *mat.get_0_1(), *mat.get_0_2(), T::ZERO],
            [*mat.get_1_0(), *mat.get_1_1(), *mat.get_1_2(), T::ZERO],
            [*mat.get_2_0(), *mat.get_2_1(), *mat.get_2_2(), T::ZERO],
            [pos.x, pos.y, pos.z, T::ONE],
        ]
    )
}

pub fn perspective_proj_matrix(fov: f32, near: f32, far: f32, aspect: f32) -> Matrix4x4<f32> {
    // todo
    // let s = -1_f32 / ((fov / 2_f32).tan());

    // Matrix4x4::<f32>::from_array([
    //     [s, 0_f32, 0_f32, 0_f32],
    //     [0_f32, s, 0_f32, 0_f32],
    //     [0_f32, 0_f32, (-far / (far - near)), 1_f32],
    //     [0_f32, 0_f32, ((-far * near) / (far - near)), 0_f32],
    // ])

    
    let s = 1_f32 / ((fov / 2_f32).tan());

    Matrix4x4::<f32>::from_array([
        [(s / aspect), 0_f32, 0_f32, 0_f32],
        [0_f32, s, 0_f32, 0_f32],
        [0_f32, 0_f32, (far / (far - near)), 1_f32],
        [0_f32, 0_f32, ((-far * near) / (far - near)), 0_f32],
    ])
}