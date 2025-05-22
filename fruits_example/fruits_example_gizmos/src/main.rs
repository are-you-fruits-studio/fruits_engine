use fruits_math::{Vec2, Vec3, Vec4};
use fruits_modules::render::{GizmoLine, GizmoSpace, GizmosResource};
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
    let color = Vec4::new(1.0, 0.0, 0.0, 1.0);

    let lines = gizmos.space(GizmoSpace::Viewport);

    lines.push(GizmoLine { start: Vec3::new(-0.5, -0.5, 0.0), end: Vec3::new(-0.5, 0.5, 0.0), color, });
    lines.push(GizmoLine { start: Vec3::new(-0.5, 0.5, 0.0), end: Vec3::new(0.5, 0.5, 0.0), color, });
    lines.push(GizmoLine { start: Vec3::new(0.5, 0.5, 0.0), end: Vec3::new(0.5, -0.5, 0.0), color, });
    lines.push(GizmoLine { start: Vec3::new(0.5, -0.5, 0.0), end: Vec3::new(-0.5, -0.5, 0.0), color, });
}
