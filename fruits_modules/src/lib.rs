use fruits_asset_storage::AssetStorageResource;
use fruits_ecs::{Schedule, WorldBuilderMut};
use fruits_prefab::{Prefab, PrefabComponentsDeserializerResource};

pub mod fps_counter;

pub fn add_defult_modules_to(mut world: WorldBuilderMut) {
    fruits_collision::add_collision_module_to(world.as_mut());
    fruits_transform::add_transform_module_to(world.as_mut());
    fruits_render::add_render_module_to(world.as_mut());

    world.data_mut().resources_mut().insert(AssetStorageResource::<Prefab>::new()).ok().unwrap();
    world.data_mut().resources_mut().insert(PrefabComponentsDeserializerResource::default()).ok().unwrap();

    world
        .behavior_mut()
        .get_mut(Schedule::Update)
        .order_group(fruits_collision::SYSTEM_GROUP_COLLISION)
        .before_group(fruits_transform::SYSTEM_GROUP_TRANSFORM)
        .before_group(fruits_render::SYSTEM_GROUP_RENDER);
}
