mod assets;
mod components;
mod resources;
mod systems;
pub mod utils;

use crate::asset::AssetStorageResource;

pub use self::{assets::*, components::*, resources::*, systems::*};

use fruits_ecs::{Schedule, WorldBuilderMut};

pub const SYSTEM_GROUP_RENDER: &'static str = "fruits_render";

pub(crate) fn add_module_to(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(SurfaceTextureResource { texture: None })
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<StandardMaterial>::new())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<StandardMesh>::new())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<StandardTexture>::new())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(AssetStorageResource::<Font>::new())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(GizmosResource::default())
        .ok()
        .unwrap();
    world
        .data_mut()
        .resources_mut()
        .insert(ScreenSpaceResource::default())
        .ok()
        .unwrap();

    let mut world_behavior = world.behavior_mut();

    let mut start = world_behavior.get_mut(Schedule::Start);

    start
        .group(SYSTEM_GROUP_RENDER)
        .insert_child_system(create_standard_render_resource)
        .insert_child_system(recreate_depth_texture_resource)
        .insert_child_system(recreate_transparent_target_resource)
        .insert_child_system(create_gizmos_render_resource);

    start
        .order_system(recreate_depth_texture_resource)
        .before_system(create_standard_render_resource);
    start
        .order_system(recreate_transparent_target_resource)
        .before_system(create_standard_render_resource);

    let mut update = world_behavior.get_mut(Schedule::Update);

    update
        .group(SYSTEM_GROUP_RENDER)
        .insert_child_system(request_surface_texture)
        .insert_child_group("fruits_render_stuff")
        .insert_child_system(present_surface);

    update
        .group("fruits_render_stuff")
        .insert_child_system(update_camera_uniform)
        .insert_child_system(update_text_batched_mesh)
        .insert_child_system(update_image_batched_mesh)
        .insert_child_system(update_masked_batched_mesh)
        .insert_child_system(recreate_depth_texture_resource)
        .insert_child_system(recreate_transparent_target_resource)
        .insert_child_system(clear_depth)
        .insert_child_system(clear_transparent_target)
        .insert_child_system(render_opaque_instanced)
        .insert_child_system(render_opaque_batched)
        .insert_child_system(render_transparent_instanced)
        .insert_child_system(render_transparent_batched)
        .insert_child_system(render_transparent_final)
        .insert_child_system(render_gizmos);

    update
        .order_system(recreate_depth_texture_resource)
        .before_system(recreate_transparent_target_resource)
        .before_system(clear_depth)
        .before_system(clear_transparent_target)
        .before_system(update_camera_uniform)
        .before_system(render_opaque_instanced)
        .before_system(render_opaque_batched)
        .before_system(render_transparent_instanced)
        .before_system(render_transparent_batched)
        .before_system(render_transparent_final)
        .before_system(render_gizmos);

    update
        .order_system(update_text_batched_mesh)
        .before_system(update_masked_batched_mesh);
    update
        .order_system(update_image_batched_mesh)
        .before_system(update_masked_batched_mesh);
    update
        .order_system(update_masked_batched_mesh)
        .before_system(render_opaque_batched);
    update
        .order_system(update_masked_batched_mesh)
        .before_system(render_transparent_batched);

    update
        .order_system(request_surface_texture)
        .before_group("fruits_render_stuff");
    update.order_group("fruits_render_stuff").before_system(present_surface);
}
