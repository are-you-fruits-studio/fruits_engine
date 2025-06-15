mod line_bound_type;
mod shapes;
mod shapes_overlap;
mod resources;
mod systems;
mod components;
mod utils;

pub use line_bound_type::*;
pub use shapes::*;
pub use shapes_overlap::*;
pub use resources::*;
pub use systems::*;
pub use components::*;
pub use utils::*;

use fruits_ecs::{Schedule, WorldBuilder};

pub const SYSTEM_GROUP: &'static str = "fruits_collision";

pub fn add_module_to(world: &mut WorldBuilder) {
    world.data_mut().resources_mut().insert(CollisionWorldResource::default()).ok().unwrap();

    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP).add_child_system(update_collision_world);
}
