mod assets;
mod components;
mod resources;
mod systems;
pub mod utils;

use crate::asset::AssetStorageResource;

pub use self::{
    assets::*,
    components::*,
    resources::*,
    systems::*,
};

use fruits_ecs::{Schedule, WorldBuilder};

pub const SYSTEM_GROUP: &'static str = "fruits_render";

pub fn add_module_to(world: &mut WorldBuilder) {
    world.data_mut().resources_mut().insert(SurfaceTextureResource { texture: None, }).ok().unwrap();
    world.data_mut().resources_mut().insert(AssetStorageResource::<StandardMaterial>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(AssetStorageResource::<StandardMesh>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(AssetStorageResource::<StandardTexture>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(AssetStorageResource::<Font>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(GizmosResource::default()).ok().unwrap();
    world.data_mut().resources_mut().insert(ScreenSpaceResource::default()).ok().unwrap();

    let start = world.behavior_mut().get_mut(Schedule::Start);

    start.group(SYSTEM_GROUP)
        .add_child_system(create_standard_render_resource)
        .add_child_system(recreate_depth_texture_resource)
        .add_child_system(create_gizmos_render_resource);

    start.order_system(recreate_depth_texture_resource).before_system(create_standard_render_resource);

    let update = world.behavior_mut().get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .add_child_system(request_surface_texture)
        .add_child_group("fruits_render_stuff")
        .add_child_system(present_surface);

    update.group("fruits_render_stuff")
        .add_child_system(update_camera_uniform)
        .add_child_system(update_text_batched_mesh)
        .add_child_system(update_image_batched_mesh)
        .add_child_system(recreate_depth_texture_resource)
        .add_child_system(clear_depth)
        .add_child_system(render_meshes_and_materials_instanced)
        .add_child_system(render_meshes_and_materials_batched)
        .add_child_system(render_gizmos);
    
    update.order_system(update_camera_uniform).before_system(render_meshes_and_materials_instanced);
    update.order_system(render_meshes_and_materials_instanced).before_system(render_meshes_and_materials_batched);
    update.order_system(update_camera_uniform).before_system(render_meshes_and_materials_batched);
    update.order_system(recreate_depth_texture_resource).before_system(clear_depth);
    update.order_system(clear_depth).before_system(render_meshes_and_materials_instanced);
    update.order_system(clear_depth).before_system(render_meshes_and_materials_batched);
    update.order_system(update_text_batched_mesh).before_system(render_meshes_and_materials_batched);
    update.order_system(update_image_batched_mesh).before_system(render_meshes_and_materials_batched);
    update.order_system(render_meshes_and_materials_instanced).before_system(render_gizmos);
    update.order_system(render_meshes_and_materials_batched).before_system(render_gizmos);

    update.order_system(request_surface_texture).before_group("fruits_render_stuff");
    update.order_group("fruits_render_stuff").before_system(present_surface);
}
