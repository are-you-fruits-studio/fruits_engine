use core::f32;

use fruits_math::{Quat, Vec2, Vec3, Vec4};
use fruits_modules::{collision::{self, ColliderComponent, CollisionAabb, CollisionBox, CollisionLine, CollisionShape, CollisionSphere, CollisionWorldResource, LineBoundType}, render::{CameraComponent, GizmoLine, GizmoSpace, GizmosResource}, transform::{GlobalTransform, LocalTransform}};
use fruits_prelude::*;

fn main() {
    let mut app = App::new();

    fruits_modules::render::add_module_to(app.ecs_mut());
    fruits_modules::transform::add_module_to(app.ecs_mut());
    fruits_modules::collision::add_module_to(app.ecs_mut());
    fruits_modules::fps_counter::add_module_to(app.ecs_mut());

    app.ecs_mut().behavior_mut().get_mut(Schedule::Start).add_system(init);

    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(move_camera);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(draw_collisions);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(update_colliders);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(draw_gizmo_components);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).add_system(raycast_mouse);

    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).order_system(raycast_mouse).before_system(draw_gizmo_components);
    app.ecs_mut().behavior_mut().get_mut(Schedule::Update).order_system(draw_gizmo_components).before_group(fruits_modules::render::SYSTEM_GROUP);

    app.run();
}

#[derive(Component)]
struct GizmoComponent {
    pub space: GizmoSpace,
    pub color: Vec4<f32>,
}

fn init(
    mut world: ExclusiveWorldAccess,
) {
    let ec = world.entities_components_mut();

    let camera = ec.create_entity();

    ec.add_component(camera, LocalTransform::IDENTITY).ok().unwrap();
    ec.add_component(camera, CameraComponent {
        near: 0.1_f32,
        far: 1_000_f32,
        fov: 90_f32.to_radians(),
    }).ok().unwrap();

    //

    for x in 0..60 {
        for y in 0..60 {
            create_button_entity(
                ec,
                Vec3::new(15.0 * x as f32, 15.0 * y as f32, 0.0),
                Vec3::new(9.0, 9.0, 0.1),
            );
        }
    }
}

fn create_button_entity(ec: &mut EntitiesComponentsHolder, pos: Vec3<f32>, scale: Vec3<f32>) {
    let e = ec.create_entity();

    ec.add_component(e, LocalTransform {
        position: pos,
        rotation: Quat::IDENTITY,
        scale: scale,
    }).ok().unwrap();
    ec.add_component(e, GizmoComponent {
        color: Vec4::new(0.5, 0.5, 0.5, 0.0),
        // todo: Window doesn't work?
        space: GizmoSpace::Window,
    }).ok().unwrap();
    ec.add_component(e, ColliderComponent {
        shape: CollisionShape::Aabb(CollisionAabb {
            center: Vec3::with_all(0.0),
            extents: Vec3::with_all(0.0),
        })
    }).ok().unwrap();
}

fn update_colliders(
    mut q: WorldQuery<(&GlobalTransform, &LocalTransform, &mut ColliderComponent)>,
) {
    for (global_transform, local_transform, collider) in q.iter_mut() {
        let CollisionShape::Aabb(collision_aabb) = &mut collider.shape else {
            todo!();
        };

        collision_aabb.center = global_transform.position;
        collision_aabb.extents = local_transform.scale * 0.5;
    }
}

fn raycast_mouse(
    collision_world: Res<CollisionWorldResource>,
    input: Res<InputResource>,
    mut q: WorldQuery<&mut GizmoComponent>
) {
    for gizmo_component in q.iter_mut() {
        gizmo_component.color = Vec4::new(0.5, 0.5, 0.5, 1.0);
    }

    let start = Vec3::new(input.mouse.position[0] as f32, input.mouse.position[1] as f32, 0.0);
    let end = start + Vec3::<f32>::Z;

    let line = CollisionLine {
        start,
        end,
        bounds: LineBoundType::UNRESTRICTED,
    };
    
    let mut results = Vec::new();
    collision_world.overlaps(line.into(), &mut results);
    for e in results {
        let Some(gizmo_component) = q.get_mut(e) else {
            continue;
        };
        
        gizmo_component.color = match input.mouse.is_pressed(MouseButton::Left) {
            false => Vec4::new(1.0, 0.0, 0.0, 1.0),
            true => Vec4::new(0.0, 1.0, 0.0, 1.0)
        };
    }
}

fn draw_gizmo_components(
    q: WorldQuery<(&GizmoComponent, &ColliderComponent)>,
    mut gizmos: ResMut<GizmosResource>,
) {
    for (gizmo, collider) in q.iter() {
        let CollisionShape::Aabb(collision_aabb) = collider.shape else {
            todo!();
        };

        draw_gizmo_collision_shape(gizmos.space(gizmo.space), gizmo.color, collision_aabb.into());   
    }
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

fn draw_collisions(
    mut gizmos: ResMut<GizmosResource>,
) {
    let sh1 = CollisionAabb {
        center: Vec3::new(2.2, 0.0, 0.0),
        extents: Vec3::new(0.5, 0.5, 0.5)
    }.into();
    let sh2 = CollisionBox {
        center: Vec3::new(1.0, 0.0, 0.0),
        extents: Vec3::with_all(0.5),
        rotation: Quat::rotation_y(1.0),
    }.into();

    let overlaps = collision::overlaps(
        sh1,
        sh2,
    );

    let lines = gizmos.space(GizmoSpace::World);
    let color = if overlaps { Vec4::new(1.0, 0.0, 0.0, 1.0) } else { Vec4::new(0.0, 1.0, 0.0, 1.0) };

    draw_gizmo_collision_shape(lines, color, sh1);
    draw_gizmo_collision_shape(lines, color, sh2);
}

//

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

    let ext = sh.extents;

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
    let ext = sh.extents;

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
