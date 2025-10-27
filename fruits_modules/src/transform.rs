mod components;
mod systems;
pub mod utils;

pub use self::{
    components::*,
    systems::*,
};

use fruits_ecs::{Schedule, SystemsHolderBuilderMut, WorldBuilder, WorldBuilderMut};

pub const SYSTEM_GROUP_TRANSFORM: &'static str = "fruits_transform";

pub(crate) fn add_module_to(mut world: WorldBuilderMut) {
    add_module_to_schedule(world.behavior_mut().get_mut(Schedule::Start));
    add_module_to_schedule(world.behavior_mut().get_mut(Schedule::Update));
}

fn add_module_to_schedule(mut schedule: SystemsHolderBuilderMut) {
    schedule.group(SYSTEM_GROUP_TRANSFORM)
        .insert_child_system(adjust_component_sets)
        .insert_child_system(update_parents_remove_invalid_children)
        .insert_child_system(update_parents_add_missing_children)
        .insert_child_system(calculate_global_disableable)
        .insert_child_system(calculate_global_transform)
        .insert_child_system(calculate_global_rect_scale_hierarchy_independent)
        .insert_child_system(calculate_global_rect_scale_children_based)
        .insert_child_system(calculate_global_rect_scale_parent_based)
        .insert_child_system(calculate_global_rect_pos);

    schedule.order_system(adjust_component_sets).before_system(update_parents_remove_invalid_children);
    schedule.order_system(update_parents_remove_invalid_children).before_system(update_parents_add_missing_children);
    schedule.order_system(update_parents_add_missing_children).before_system(calculate_global_disableable);
    schedule.order_system(update_parents_add_missing_children).before_system(calculate_global_transform);
    schedule.order_system(update_parents_add_missing_children).before_system(calculate_global_rect_scale_hierarchy_independent);
    schedule.order_system(update_parents_add_missing_children).before_system(calculate_global_rect_scale_children_based);
    schedule.order_system(update_parents_add_missing_children).before_system(calculate_global_rect_scale_parent_based);
    schedule.order_system(update_parents_add_missing_children).before_system(calculate_global_rect_pos);
    schedule.order_system(calculate_global_rect_scale_hierarchy_independent).before_system(calculate_global_rect_scale_children_based);
    schedule.order_system(calculate_global_rect_scale_children_based).before_system(calculate_global_rect_scale_parent_based);
    schedule.order_system(calculate_global_rect_scale_parent_based).before_system(calculate_global_rect_pos);
}