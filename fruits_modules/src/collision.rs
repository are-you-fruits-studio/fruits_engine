mod line_bound_type;
mod shapes;
mod shapes_overlap;
mod resources;
mod systems;
mod components;
mod utils;
// mod shapes_collision;

pub use line_bound_type::*;
pub use shapes::*;
pub use shapes_overlap::*;
// todo
// pub use shapes_collision::*;
pub use resources::*;
pub use systems::*;
pub use components::*;
pub use utils::*;

use fruits_ecs::{Schedule, WorldBuilder, WorldBuilderMut};

pub const SYSTEM_GROUP_COLLISION: &'static str = "fruits_collision";

pub(crate) fn add_module_to(mut world: WorldBuilderMut) {
    world.data_mut().resources_mut().insert(CollisionWorldResource::default()).ok().unwrap();

    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP_COLLISION).insert_child_system(update_collision_world);
}
