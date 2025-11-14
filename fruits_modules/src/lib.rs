use fruits_ecs::{Schedule, WorldBuilder, WorldBuilderMut};

mod asset;
mod collision;
pub mod fps_counter;
mod render;
mod transform;

pub use asset::*;
pub use collision::*;
pub use render::*;
pub use transform::*;

pub mod utils {
    pub use crate::render::utils::*;
    pub use crate::transform::utils::*;
}

pub fn add_defult_modules_to(mut world: WorldBuilderMut) {
    collision::add_module_to(world.as_mut());
    transform::add_module_to(world.as_mut());
    render::add_module_to(world.as_mut());

    world
        .behavior_mut()
        .get_mut(Schedule::Update)
        .order_group(collision::SYSTEM_GROUP_COLLISION)
        .before_group(transform::SYSTEM_GROUP_TRANSFORM);
    world
        .behavior_mut()
        .get_mut(Schedule::Update)
        .order_group(transform::SYSTEM_GROUP_TRANSFORM)
        .before_group(render::SYSTEM_GROUP_RENDER);
}
