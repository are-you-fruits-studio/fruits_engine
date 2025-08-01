use fruits_math::Mat4;

pub const GIZMO_LINES_PER_DRAW_MAX: usize = 1024 * 16;
pub const STANDARD_MESH_MATERIAL_INSTANCES_PER_DRAW_MAX: usize = 1024;
pub const BATCHED_MESH_MATERIAL_TRIANGLES_PER_DRAW_MAX: usize = 1024 * 32;

pub fn create_window_to_clip_matrix(width: f32, height: f32, near: f32, far: f32) -> Mat4<f32> {
    let z_near_to_far_inv = 1.0 / (far - near);
    
    Mat4::from_array([
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, -2.0 / height, 0.0, 0.0],
        [0.0, 0.0, z_near_to_far_inv, 0.0],
        [-1.0, 1.0, -near * z_near_to_far_inv, 1.0],
    ])
}