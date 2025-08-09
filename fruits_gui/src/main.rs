use fruits_modules::{render, transform::{self}};
use fruits_prelude::*;

mod systems;
mod resources;
mod events;

use systems::*;

fn main() {
    let mut app = App::new();

    fruits_modules::transform::add_module_to(app.ecs_mut());
    fruits_modules::render::add_module_to(app.ecs_mut());
    fruits_modules::fps_counter::add_module_to(app.ecs_mut());
    // todo: to fruits_modules
    fruits_debug::add_module_as_client_to(app.ecs_mut());

    app.ecs_mut().behavior_mut().get_mut(Schedule::Start).add_system(init);

    // app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(respawn_scene);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(connect_debug_system);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(parse_debug_msg_system);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(update_hierarchy);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(request_hierarchy_system);
    

    // app.ecs_mut().behavior_mut().get_mut(Schedule::Update).order_system(respawn_scene).before_group(transform::SYSTEM_GROUP);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).order_system(parse_debug_msg_system).before_system(update_hierarchy);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).order_system(update_hierarchy).before_group(transform::SYSTEM_GROUP);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).order_system(update_hierarchy).before_group(render::SYSTEM_GROUP);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).order_group(transform::SYSTEM_GROUP).before_group(render::SYSTEM_GROUP);

    app.run();
}
