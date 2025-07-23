use fruits_math::Mat4;

pub const GIZMO_LINES_PER_DRAW_MAX: usize = 1024 * 16;
pub const STANDARD_MESH_MATERIAL_INSTANCES_PER_DRAW_MAX: usize = 1024;

pub fn create_screen_world_to_clip_matrix(width: f32, height: f32, near: f32, far: f32) -> Mat4<f32> {
    let z_center = (near + far) * 0.5;
    let z_far_to_center = far - z_center;
    
    Mat4::from_array([
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, -2.0 / height, 0.0, 0.0],
        [0.0, 0.0, 1.0 / z_far_to_center, 0.0],
        [-1.0, 1.0, -z_center / z_far_to_center, 1.0],
    ])
}