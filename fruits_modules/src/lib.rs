use fruits_ecs::{Schedule, WorldBuilder};

mod asset;
mod render;
mod transform;
mod collision;
pub mod fps_counter;

pub use asset::*;
pub use render::*;
pub use transform::*;
pub use collision::*;

pub mod utils {
    pub use crate::render::utils::*;
    pub use crate::transform::utils::*;
}

pub fn add_defult_modules_to(world: &mut WorldBuilder) {
    collision::add_module_to(world);
    transform::add_module_to(world);
    render::add_module_to(world);

    world.behavior_mut().get_mut(Schedule::Update).order_group(collision::SYSTEM_GROUP_COLLISION).before_group(transform::SYSTEM_GROUP_TRANSFORM);
    world.behavior_mut().get_mut(Schedule::Update).order_group(transform::SYSTEM_GROUP_TRANSFORM).before_group(render::SYSTEM_GROUP_RENDER);
}