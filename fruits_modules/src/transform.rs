mod components;
mod systems;

pub use self::{
    components::*,
    systems::*,
};

use fruits_ecs::{Schedule, WorldBuilder};

pub fn add_module_to(world: &mut WorldBuilder) {
    let update = world.behavior_mut().get_mut(Schedule::Update);

    update.add_system(adjust_component_sets);
    update.add_system(update_parents_remove_invalid_children);
    update.add_system(update_parents_add_missing_children);
    update.add_system(update_parents_destroy_empty_parents);
    update.add_system(calculate_global_transform);
    
    update.order(adjust_component_sets).before(update_parents_remove_invalid_children);
    update.order(update_parents_remove_invalid_children).before(update_parents_add_missing_children);
    update.order(update_parents_add_missing_children).before(update_parents_destroy_empty_parents);
    update.order(update_parents_destroy_empty_parents).before(calculate_global_transform);
}