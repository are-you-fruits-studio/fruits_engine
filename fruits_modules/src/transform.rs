mod components;
mod systems;

pub use self::{
    components::*,
    systems::*,
};

use fruits_ecs::{Schedule, WorldBuilder};

pub const SYSTEM_GROUP: &'static str = "fruits_transform";

pub fn add_module_to(world: &mut WorldBuilder) {
    let update = world.behavior_mut().get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .add_child_system(adjust_component_sets)
        .add_child_system(update_parents_remove_invalid_children)
        .add_child_system(update_parents_add_missing_children)
        .add_child_system(update_parents_destroy_empty_parents)
        .add_child_system(calculate_global_transform);

    update.order_system(adjust_component_sets).before_system(update_parents_remove_invalid_children);
    update.order_system(update_parents_remove_invalid_children).before_system(update_parents_add_missing_children);
    update.order_system(update_parents_add_missing_children).before_system(update_parents_destroy_empty_parents);
    update.order_system(update_parents_destroy_empty_parents).before_system(calculate_global_transform);
}