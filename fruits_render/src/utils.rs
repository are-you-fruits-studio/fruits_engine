use fruits_math::Mat4;
use wgpu::*;
use fruits_render_core::StandardTexture;
use fruits_render_core::RenderState;

use crate::{
    AssetStorageResource, RenderSpace, StandardMaterial, StandardRenderAssetsResource, StandardRenderResource,
    StandardUniform,
};

pub const GIZMO_LINES_PER_DRAW_MAX: usize = 1024 * 16;
pub const INSTANCES_PER_DRAW_MAX: usize = 1024 * 8;
pub const TRIANGLES_PER_BATCHED_DRAW_MAX: usize = 1024 * 32;

pub fn create_window_to_clip_matrix(width: f32, height: f32, near: f32, far: f32) -> Mat4<f32> {
    let z_near_to_far_inv = 1.0 / (far - near);

    Mat4::from_array([
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, -2.0 / height, 0.0, 0.0],
        [0.0, 0.0, z_near_to_far_inv, 0.0],
        [-1.0, 1.0, -near * z_near_to_far_inv, 1.0],
    ])
}

// todo
pub(crate) fn get_render_data<'a, 'b>(
    material: &StandardMaterial,
    standard_render_res: &'a StandardRenderResource,
    render_state: &RenderState,
    textures: &'b AssetStorageResource<StandardTexture>,
    standard_render_assets_res: &StandardRenderAssetsResource,
    window_to_clip_mat: Mat4<f32>,
) -> (&'a RenderPipeline, &'a BindGroup, &'b BindGroup) {
    let world_to_clip = match material.space {
        RenderSpace::Clip => Mat4::IDENTITY,
        RenderSpace::Window => window_to_clip_mat,
        RenderSpace::World => standard_render_res.camera_proj_matrix,
    };

    let uniform = StandardUniform {
        color: material.color.map(|x| x.powf(2.2)),
        emission_color: material.emission_color.map(|x| x.powf(2.2)),
        metallic: material.metallic,
        roughness: material.roughness,
        alpha_threshold: material.alpha_threshold.unwrap_or(0.0),
        camera_position_world: standard_render_res.camera_pos,
        world_to_clip,
        _padding: Default::default(),
    };

    render_state.queue().write_buffer(
        &standard_render_res.buffer_uniform,
        0,
        fruits_utils::mem::as_bytes(&[uniform]),
    );

    let bind_group_tex = match &material.color_tex {
        Some(color_tex) => unsafe { &textures.get(color_tex).unwrap().native().bind_group },
        None => {
            unsafe { &textures
                .get(&standard_render_assets_res.texture_white)
                .unwrap()
                .native()
                .bind_group }
        }
    };

    let render_pipeline = match (material.is_lit, material.alpha_threshold.is_some()) {
        (true, true) => &standard_render_res.render_pipeline_opaque_lit,
        (false, true) => &standard_render_res.render_pipeline_opaque_unlit,
        (true, false) => &standard_render_res.render_pipeline_transparent_lit,
        (false, false) => &standard_render_res.render_pipeline_transparent_unlit,
    };

    (&render_pipeline, &standard_render_res.bind_group_uniform, bind_group_tex)
}