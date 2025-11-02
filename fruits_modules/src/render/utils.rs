use fruits_math::Mat4;
use wgpu::*;

use crate::{AssetStorageResource, DepthTextureResource, LitUniform, RenderSpace, RenderState, StandardMaterial, StandardRenderAssetsResource, StandardRenderResource, StandardTexture, UnlitUniform};

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

// todo
pub(crate) fn get_render_data<'a, 'b>(
    material: &StandardMaterial,
    standard_render_res: &'a StandardRenderResource,
    render_state: &RenderState,
    textures: &'b AssetStorageResource<StandardTexture>,
    standard_render_assets_res: &StandardRenderAssetsResource,
    window_to_clip_mat: Mat4<f32>,
) -> (&'a RenderPipeline, &'a BindGroup, &'b BindGroup) {
    match material {
        StandardMaterial::Lit(material) => {
            let lit_data = &standard_render_res.lit;
            
            let world_to_clip = match material.space {
                RenderSpace::Clip => Mat4::IDENTITY,
                RenderSpace::Window => window_to_clip_mat,
                RenderSpace::World => standard_render_res.camera_proj_matrix,
            };

            let uniform = LitUniform {
                albedo_color: material.albedo_color,
                metallic: material.metallic,
                emission_color: material.emission_color,
                roughness: material.roughness,
                alpha_threshold: material.alpha_threshold,
                camera_position_world: standard_render_res.camera_pos,
                world_to_clip,
                _padding: Default::default(),
            };

            render_state.queue().write_buffer(&lit_data.buffer_uniform, 0, fruits_utils::mem::as_bytes(&[uniform]));

            let bind_group_tex = match &material.albedo_tex {
                Some(albedo_tex) => &textures.get(albedo_tex).unwrap().native().bind_group,
                None => &textures.get(&standard_render_assets_res.texture_white).unwrap().native().bind_group,
            };

            (&lit_data.render_pipeline, &lit_data.bind_group_uniform, bind_group_tex)
        },
        StandardMaterial::Unlit(material) => {
            let unlit_data = &standard_render_res.unlit;

            let world_to_clip = match material.space {
                RenderSpace::Clip => Mat4::IDENTITY,
                RenderSpace::Window => window_to_clip_mat,
                RenderSpace::World => standard_render_res.camera_proj_matrix,
            };

            let uniform = UnlitUniform {
                world_to_clip,
                color: material.color,
                alpha_threshold: material.alpha_threshold,
                _padding: Default::default(),
            };

            render_state.queue().write_buffer(&unlit_data.buffer_uniform, 0, fruits_utils::mem::as_bytes(&[uniform]));

            let bind_group_tex = match &material.color_tex {
                Some(color_tex) => &textures.get(color_tex).unwrap().native().bind_group,
                None => &textures.get(&standard_render_assets_res.texture_white).unwrap().native().bind_group,
            };

            (&unlit_data.render_pipeline, &unlit_data.bind_group_uniform, bind_group_tex)
        },
    }
}