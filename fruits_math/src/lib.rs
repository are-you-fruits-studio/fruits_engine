mod mat;
mod mat2;
mod mat3;
mod mat4;
mod num;
mod vec;
mod quat;

pub use mat::*;
pub use mat2::*;
pub use mat3::*;
pub use mat4::*;
pub use num::*;
pub use vec::*;
pub use quat::*;

pub fn into_matrix4x4_with_pos<T: Number>(mat: Mat3<T>, pos: Vec3<T>) -> Mat4<T> {
    Mat4::from_array(
        [
            [mat[0][0], mat[0][1], mat[0][2], T::ZERO],
            [mat[1][0], mat[1][1], mat[1][2], T::ZERO],
            [mat[2][0], mat[2][1], mat[2][2], T::ZERO],
            [pos.x, pos.y, pos.z, T::ONE],
        ]
    )
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