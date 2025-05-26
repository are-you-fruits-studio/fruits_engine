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

pub fn add_module_to(world: &mut WorldBuilder) {
    world.data_mut().resources_mut().insert(SurfaceTextureResource { texture: None, }).ok().unwrap();
    world.data_mut().resources_mut().insert(AssetStorageResource::<Material>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(AssetStorageResource::<Mesh>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(GizmosResource::new()).ok().unwrap();
    
    world.behavior_mut().get_mut(Schedule::Start).add_system(create_camera_uniform_buffer);
    world.behavior_mut().get_mut(Schedule::Start).add_system(create_camera_uniform_bind_group_layout);
    world.behavior_mut().get_mut(Schedule::Start).add_system(create_instance_buffer);
    world.behavior_mut().get_mut(Schedule::Start).add_system(recreate_depth_texture_resource);
    world.behavior_mut().get_mut(Schedule::Start).add_system(create_gizmos_render_resource);
    world.behavior_mut().get_mut(Schedule::Update).add_system(update_camera_uniform_buffer);
    world.behavior_mut().get_mut(Schedule::Update).add_system(recreate_depth_texture_resource);
    world.behavior_mut().get_mut(Schedule::Update).add_system(request_surface_texture);
    world.behavior_mut().get_mut(Schedule::Update).add_system(clear_depth);
    world.behavior_mut().get_mut(Schedule::Update).add_system(render_meshes_and_materials);
    world.behavior_mut().get_mut(Schedule::Update).add_system(render_gizmos);
    world.behavior_mut().get_mut(Schedule::Update).add_system(present_surface);
    
    world.behavior_mut().get_mut(Schedule::Start).order(create_camera_uniform_bind_group_layout).before(create_camera_uniform_buffer);
    world.behavior_mut().get_mut(Schedule::Update).order(update_camera_uniform_buffer).before(render_meshes_and_materials);
    world.behavior_mut().get_mut(Schedule::Update).order(request_surface_texture).before(present_surface);
    world.behavior_mut().get_mut(Schedule::Update).order(request_surface_texture).before(render_meshes_and_materials);
    world.behavior_mut().get_mut(Schedule::Update).order(recreate_depth_texture_resource).before(clear_depth);
    world.behavior_mut().get_mut(Schedule::Update).order(clear_depth).before(render_meshes_and_materials);
    world.behavior_mut().get_mut(Schedule::Update).order(render_meshes_and_materials).before(render_gizmos);
    world.behavior_mut().get_mut(Schedule::Update).order(render_gizmos).before(present_surface);
}
