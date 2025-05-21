use fruits_math::Vec2;
use fruits_modules::render::GizmosResource;
use fruits_prelude::{App, ResMut, Schedule};

fn main() {
    let mut app = App::new();

    fruits_modules::render::add_module_to(app.ecs_mut());

    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(update_system);

    app.run();
}

fn update_system(
    mut gizmos: ResMut<GizmosResource>,
) {
    gizmos.lines.push([Vec2::new(-0.25, -0.25), Vec2::new(0.25, 0.25)]);
}
