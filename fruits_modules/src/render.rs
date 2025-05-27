mod assets;
mod components;
mod resources;
mod systems;
mod utils;

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
    world.data_mut().resources_mut().insert(AssetStorageResource::<Material>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(AssetStorageResource::<Mesh>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(GizmosResource::new()).ok().unwrap();

    let start = world.behavior_mut().get_mut(Schedule::Start);

    start.group(SYSTEM_GROUP)
        .add_child_system(create_camera_uniform_buffer)
        .add_child_system(create_camera_uniform_bind_group_layout)
        .add_child_system(create_instance_buffer)
        .add_child_system(recreate_depth_texture_resource)
        .add_child_system(create_gizmos_render_resource);

    start.order_system(create_camera_uniform_bind_group_layout).before_system(create_camera_uniform_buffer);

    let update = world.behavior_mut().get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .add_child_system(update_camera_uniform_buffer)
        .add_child_system(recreate_depth_texture_resource)
        .add_child_system(request_surface_texture)
        .add_child_system(clear_depth)
        .add_child_system(render_meshes_and_materials)
        .add_child_system(render_gizmos)
        .add_child_system(present_surface);
    
    update.order_system(update_camera_uniform_buffer).before_system(render_meshes_and_materials);
    update.order_system(request_surface_texture).before_system(present_surface);
    update.order_system(request_surface_texture).before_system(render_meshes_and_materials);
    update.order_system(recreate_depth_texture_resource).before_system(clear_depth);
    update.order_system(clear_depth).before_system(render_meshes_and_materials);
    update.order_system(render_meshes_and_materials).before_system(render_gizmos);
    update.order_system(render_gizmos).before_system(present_surface);
}
