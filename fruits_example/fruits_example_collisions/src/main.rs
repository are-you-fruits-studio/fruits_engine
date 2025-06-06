use core::f32;

use fruits_math::{Mat, Quat, Vec2, Vec3, Vec4};
use fruits_modules::{collision::{self, CollisionAabb, CollisionBox, CollisionLine, CollisionShape, CollisionSphere}, render::{CameraComponent, GizmoLine, GizmoSpace, GizmosResource}, transform::{GlobalTransform, LocalTransform}};
use fruits_prelude::*;

fn main() {
    let mut app = App::new();

    fruits_modules::render::add_module_to(app.ecs_mut());
    fruits_modules::transform::add_module_to(app.ecs_mut());

    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(move_camera);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(update_system);

    let ec = app.ecs_mut().data_mut().entities_components_mut();
    
    let camera = ec.create_entity();

    ec.add_component(camera, GlobalTransform {
        scale_rotation: Mat::IDENTITY,
        position: Vec3::new(0.0_f32, 0.0_f32, -2.0f32),
    }).ok().unwrap();
    ec.add_component(camera, CameraComponent {
        near: 0.1_f32,
        far: 1_000_f32,
        fov: 90_f32.to_radians(),
    }).ok().unwrap();
    ec.add_component(camera, LocalTransform::IDENTITY).ok().unwrap();
    // ec.add_component(camera, GlobalTransform::IDENTITY).ok().unwrap();
    // ec.add_component(camera, CameraComponent { near: 0.1, far: 1000.0, fov: 130.0_f32.to_radians() }).ok().unwrap();

    app.run();
}

fn move_camera(
    mut q: WorldQuery<(&mut LocalTransform, &CameraComponent)>,
    input: Res<InputResource>,
) {
    let mut direction = Vec3::<f32>::with_all(0.0);
    let mut rot = 0.0;

    if input.keyboard.is_pressed(KeyCode::KeyQ) { rot -= 1.0 };
    if input.keyboard.is_pressed(KeyCode::KeyW) { direction.z += 1.0 };
    if input.keyboard.is_pressed(KeyCode::KeyE) { rot += 1.0 };
    if input.keyboard.is_pressed(KeyCode::KeyR) { direction.y += 1.0 };
    if input.keyboard.is_pressed(KeyCode::KeyA) { direction.x -= 1.0 };
    if input.keyboard.is_pressed(KeyCode::KeyS) { direction.z -= 1.0 };
    if input.keyboard.is_pressed(KeyCode::KeyD) { direction.x += 1.0 };
    if input.keyboard.is_pressed(KeyCode::KeyF) { direction.y -= 1.0 };

    for (transform, _) in q.iter_mut() {
        transform.position += transform.rotation.to_matrix() * direction * 0.01;
        transform.rotation = Quat::rotation_y(rot as f64 * 0.01) * transform.rotation;
    }
}

fn update_system(
    mut gizmos: ResMut<GizmosResource>,
) {
    let sh1 = CollisionAabb {
        center: Vec3::new(2.2, 0.0, 0.0),
        scale: Vec3::new(1.0, 1.0, 1.0)
    }.into();
    let sh2 = CollisionBox {
        center: Vec3::new(1.0, 0.0, 0.0),
        scale: Vec3::with_all(1.0),
        rotation: Quat::rotation_y(1.0),
    }.into();

    let overlaps = collision::overlaps(
        &sh1,
        &sh2,
    );

    let lines = gizmos.space(GizmoSpace::World);
    let color = if overlaps { Vec4::new(1.0, 0.0, 0.0, 1.0) } else { Vec4::new(0.0, 1.0, 0.0, 1.0) };

    draw_gizmo_collision_shape(lines, color, sh1);
    draw_gizmo_collision_shape(lines, color, sh2);
}

fn draw_gizmo_collision_shape(lines: &mut Vec<GizmoLine>, color: Vec4<f32>, sh: CollisionShape) {
    match sh {
        CollisionShape::Point(sh) => draw_gizmo_point(lines, color, sh),
        CollisionShape::Line(sh) => draw_gizmo_line(lines, color, sh),
        CollisionShape::Sphere(sh) => draw_gizmo_sphere(lines, color, sh),
        CollisionShape::Aabb(sh) => draw_gizmo_aabb(lines, color, sh),
        CollisionShape::Box(sh) => draw_gizmo_box(lines, color, sh),
        CollisionShape::Triangle(sh) => draw_gizmo_line_list(lines, color, &sh),
    }
}

fn draw_gizmo_point(lines: &mut Vec<GizmoLine>, color: Vec4<f32>, sh: Vec3<f32>) {
    let scale = 0.05;

    lines.push(GizmoLine { start: sh - Vec3::new(scale, 0.0, 0.0), end: sh + Vec3::new(scale, 0.0, 0.0), color, });
    lines.push(GizmoLine { start: sh - Vec3::new(0.0, scale, 0.0), end: sh + Vec3::new(0.0, scale, 0.0), color, });
    lines.push(GizmoLine { start: sh - Vec3::new(0.0, 0.0, scale), end: sh + Vec3::new(0.0, 0.0, scale), color, });
}

fn draw_gizmo_line(lines: &mut Vec<GizmoLine>, color: Vec4<f32>, sh: CollisionLine) {
    let mut start = sh.start;
    let mut end = sh.end;

    if !sh.bounds.is_end_restricted() {
        end += (sh.end - sh.start).normalized() * 1000.0;
    }
    if !sh.bounds.is_start_restricted() {
        start += (sh.start - sh.end).normalized() * 1000.0;
    }

    lines.push(GizmoLine { start, end, color, });
}

fn draw_gizmo_box(lines: &mut Vec<GizmoLine>, color: Vec4<f32>, sh: CollisionBox) {
    let mat = sh.rotation.to_matrix();

    let ext = sh.scale / 2.0;

    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(-ext.x, -ext.y, -ext.z), end: sh.center + mat * Vec3::new(-ext.x, ext.y, -ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(-ext.x, ext.y, -ext.z), end: sh.center + mat * Vec3::new(ext.x, ext.y, -ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(ext.x, ext.y, -ext.z), end: sh.center + mat * Vec3::new(ext.x, -ext.y, -ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(ext.x, -ext.y, -ext.z), end: sh.center + mat * Vec3::new(-ext.x, -ext.y, -ext.z), color, });

    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(-ext.x, -ext.y, ext.z), end: sh.center + mat * Vec3::new(-ext.x, ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(-ext.x, ext.y, ext.z), end: sh.center + mat * Vec3::new(ext.x, ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(ext.x, ext.y, ext.z), end: sh.center + mat * Vec3::new(ext.x, -ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(ext.x, -ext.y, ext.z), end: sh.center + mat * Vec3::new(-ext.x, -ext.y, ext.z), color, });

    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(-ext.x, -ext.y, -ext.z), end: sh.center + mat * Vec3::new(-ext.x, -ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(-ext.x, ext.y, -ext.z), end: sh.center + mat * Vec3::new(-ext.x, ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(ext.x, ext.y, -ext.z), end: sh.center + mat * Vec3::new(ext.x, ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + mat * Vec3::new(ext.x, -ext.y, -ext.z), end: sh.center + mat * Vec3::new(ext.x, -ext.y, ext.z), color, });
}

fn draw_gizmo_aabb(lines: &mut Vec<GizmoLine>, color: Vec4<f32>, sh: CollisionAabb) {
    let ext = sh.scale / 2.0;

    lines.push(GizmoLine { start: sh.center + Vec3::new(-ext.x, -ext.y, -ext.z), end: sh.center + Vec3::new(-ext.x, ext.y, -ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(-ext.x, ext.y, -ext.z), end: sh.center + Vec3::new(ext.x, ext.y, -ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(ext.x, ext.y, -ext.z), end: sh.center + Vec3::new(ext.x, -ext.y, -ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(ext.x, -ext.y, -ext.z), end: sh.center + Vec3::new(-ext.x, -ext.y, -ext.z), color, });

    lines.push(GizmoLine { start: sh.center + Vec3::new(-ext.x, -ext.y, ext.z), end: sh.center + Vec3::new(-ext.x, ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(-ext.x, ext.y, ext.z), end: sh.center + Vec3::new(ext.x, ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(ext.x, ext.y, ext.z), end: sh.center + Vec3::new(ext.x, -ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(ext.x, -ext.y, ext.z), end: sh.center + Vec3::new(-ext.x, -ext.y, ext.z), color, });

    lines.push(GizmoLine { start: sh.center + Vec3::new(-ext.x, -ext.y, -ext.z), end: sh.center + Vec3::new(-ext.x, -ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(-ext.x, ext.y, -ext.z), end: sh.center + Vec3::new(-ext.x, ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(ext.x, ext.y, -ext.z), end: sh.center + Vec3::new(ext.x, ext.y, ext.z), color, });
    lines.push(GizmoLine { start: sh.center + Vec3::new(ext.x, -ext.y, -ext.z), end: sh.center + Vec3::new(ext.x, -ext.y, ext.z), color, });
}

fn draw_gizmo_sphere(lines: &mut Vec<GizmoLine>, color: Vec4<f32>, sh: CollisionSphere) {
    let precision = 20;

    let mut last_point = Option::<Vec2<f32>>::None;

    for i in 0..precision {
        let t = i as f32 / (precision - 1) as f32;

        let (x, y) = (t * 2.0 * f32::consts::PI).sin_cos();

        if let Some(last_point) = last_point {
            lines.push(GizmoLine { start: sh.center + last_point.xyn(0.0) * sh.radius, end: sh.center + Vec3 { x, y, z: 0.0 } * sh.radius, color, });
        }

        last_point = Some(Vec2::new(x, y));
    }

    last_point = Option::<Vec2<f32>>::None;

    for i in 0..precision {
        let t = i as f32 / (precision - 1) as f32;

        let (x, z) = (t * 2.0 * f32::consts::PI).sin_cos();

        if let Some(last_point) = last_point {
            lines.push(GizmoLine { start: sh.center + last_point.xny(0.0) * sh.radius, end: sh.center + Vec3 { x, y: 0.0, z } * sh.radius, color, });
        }

        last_point = Some(Vec2::new(x, z));
    }
    
    last_point = Option::<Vec2<f32>>::None;

    for i in 0..precision {
        let t = i as f32 / (precision - 1) as f32;

        let (y, z) = (t * 2.0 * f32::consts::PI).sin_cos();

        if let Some(last_point) = last_point {
            lines.push(GizmoLine { start: sh.center + last_point.nxy(0.0) * sh.radius, end: sh.center + Vec3 { x: 0.0, y, z } * sh.radius, color, });
        }

        last_point = Some(Vec2::new(y, z));
    }
}

fn draw_gizmo_line_list(lines: &mut Vec<GizmoLine>, color: Vec4<f32>, sh: &[Vec3<f32>]) {
    for i in 0..sh.len() {
        lines.push(GizmoLine { start: sh[(i + 0) % sh.len()], end: sh[(i + 1) % sh.len()], color, });
    }
}
