mod components;
mod systems;

pub use self::{
    components::*,
    systems::*,
};

use fruits_ecs::{Schedule, WorldBuilder};

pub const SYSTEM_GROUP_TRANSFORM: &'static str = "fruits_transform";

pub(crate) fn add_module_to(world: &mut WorldBuilder) {
    let update = world.behavior_mut().get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP_TRANSFORM)
        .add_child_system(adjust_component_sets)
        .add_child_system(update_parents_remove_invalid_children)
        .add_child_system(update_parents_add_missing_children)
        .add_child_system(calculate_global_transform)
        .add_child_system(calculate_global_rect)
        .add_child_system(calculate_global_disableable);

    update.order_system(adjust_component_sets).before_system(update_parents_remove_invalid_children);
    update.order_system(update_parents_remove_invalid_children).before_system(update_parents_add_missing_children);
    update.order_system(update_parents_add_missing_children).before_system(calculate_global_transform);
    update.order_system(update_parents_add_missing_children).before_system(calculate_global_rect);
    update.order_system(update_parents_add_missing_children).before_system(calculate_global_disableable);
}