use fruits_engine::*;

// Press O to toggle the camera between perspective and orthographic projection.
#[derive(Resource)]
struct ToggleProjectionState {
    was_pressed: bool,
}

fn main() {
    let mut app = App::new();

    add_defult_modules_to(app.ecs_mut().as_mut());

    app.ecs_mut()
        .behavior_mut()
        .get_mut(Schedule::Update)
        .insert_system(move_camera);
    app.ecs_mut()
        .behavior_mut()
        .get_mut(Schedule::Update)
        .insert_system(toggle_projection);
    app.ecs_mut()
        .behavior_mut()
        .get_mut(Schedule::Update)
        .insert_system(update_system);

    let mut world_data = app.ecs_mut().data_mut();

    world_data.resources_mut().insert(ToggleProjectionState { was_pressed: false });

    let mut ec = world_data.entities_mut();

    let camera = ec.create_entity();

    ec.add_component(
        camera,
        GlobalTransform {
            scale_rotation: Mat::IDENTITY,
            position: Vec3::new(0.0_f32, 0.0_f32, -2.0f32),
        },
    )
    .ok()
    .unwrap();
    ec.add_component(
        camera,
        CameraComponent {
            near: 0.1_f32,
            far: 1_000_f32,
            projection: CameraProjection::Perspective { fov: 90.0_f32.to_radians() },
        },
    )
    .ok()
    .unwrap();
    ec.add_component(camera, LocalTransform::IDENTITY).ok().unwrap();
    // ec.add_component(camera, GlobalTransform::IDENTITY).ok().unwrap();
    // ec.add_component(camera, CameraComponent { near: 0.1, far: 1000.0, projection: CameraProjection::Perspective { fov: 130.0_f32.to_radians() } }).ok().unwrap();

    app.run();
}

fn move_camera(mut q: WorldQuery<(&mut LocalTransform, &CameraComponent)>, input: Res<InputResource>) {
    let mut direction = Vec3::<f32>::splat(0.0);
    let mut rot = 0.0;

    if input.keyboard.is_pressed(KeyCode::KeyQ) {
        rot -= 1.0
    };
    if input.keyboard.is_pressed(KeyCode::KeyW) {
        direction.z += 1.0
    };
    if input.keyboard.is_pressed(KeyCode::KeyE) {
        rot += 1.0
    };
    if input.keyboard.is_pressed(KeyCode::KeyR) {
        direction.y += 1.0
    };
    if input.keyboard.is_pressed(KeyCode::KeyA) {
        direction.x -= 1.0
    };
    if input.keyboard.is_pressed(KeyCode::KeyS) {
        direction.z -= 1.0
    };
    if input.keyboard.is_pressed(KeyCode::KeyD) {
        direction.x += 1.0
    };
    if input.keyboard.is_pressed(KeyCode::KeyF) {
        direction.y -= 1.0
    };

    for (transform, _) in q.iter_mut() {
        transform.position += transform.rotation.to_matrix() * direction * 0.01;
        transform.rotation = Quat::rotation_y(rot as f64 * 0.01) * transform.rotation;
    }
}

fn toggle_projection(
    mut q: WorldQuery<&mut CameraComponent>,
    input: Res<InputResource>,
    mut state: ResMut<ToggleProjectionState>,
) {
    let is_pressed = input.keyboard.is_pressed(KeyCode::KeyO);

    if is_pressed && !state.was_pressed {
        for camera in q.iter_mut() {
            camera.projection = match camera.projection {
                CameraProjection::Perspective { .. } => CameraProjection::Orthographic { size: 1.0 },
                CameraProjection::Orthographic { .. } => CameraProjection::Perspective { fov: 90.0_f32.to_radians() },
            };
        }
    }

    state.was_pressed = is_pressed;
}

fn update_system(mut gizmos: ResMut<GizmosResource>) {
    let lines = gizmos.space(RenderSpace::Clip);
    let color = Vec4::new(1.0, 0.0, 0.0, 1.0);

    lines.push(GizmoLine {
        start: Vec3::new(-0.5, -0.5, 0.0),
        end: Vec3::new(-0.5, 0.5, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(-0.5, 0.5, 0.0),
        end: Vec3::new(0.5, 0.5, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(0.5, 0.5, 0.0),
        end: Vec3::new(0.5, -0.5, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(0.5, -0.5, 0.0),
        end: Vec3::new(-0.5, -0.5, 0.0),
        color,
    });

    let lines = gizmos.space(RenderSpace::Window);
    let color = Vec4::new(0.0, 1.0, 0.0, 1.0);

    lines.push(GizmoLine {
        start: Vec3::new(1.0, 1.0, 0.0),
        end: Vec3::new(1.0, 100.0, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(1.0, 100.0, 0.0),
        end: Vec3::new(100.0, 100.0, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(100.0, 100.0, 0.0),
        end: Vec3::new(100.0, 1.0, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(100.0, 1.0, 0.0),
        end: Vec3::new(1.0, 1.0, 0.0),
        color,
    });

    let lines = gizmos.space(RenderSpace::World);
    let color = Vec4::new(0.0, 0.0, 1.0, 1.0);

    lines.push(GizmoLine {
        start: Vec3::new(0.0, 0.0, 0.0),
        end: Vec3::new(0.0, 1.0, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(0.0, 1.0, 0.0),
        end: Vec3::new(1.0, 1.0, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(1.0, 1.0, 0.0),
        end: Vec3::new(1.0, 0.0, 0.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(1.0, 0.0, 0.0),
        end: Vec3::new(0.0, 0.0, 0.0),
        color,
    });

    lines.push(GizmoLine {
        start: Vec3::new(0.0, 0.0, 1.0),
        end: Vec3::new(0.0, 1.0, 1.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(0.0, 1.0, 1.0),
        end: Vec3::new(1.0, 1.0, 1.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(1.0, 1.0, 1.0),
        end: Vec3::new(1.0, 0.0, 1.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(1.0, 0.0, 1.0),
        end: Vec3::new(0.0, 0.0, 1.0),
        color,
    });

    lines.push(GizmoLine {
        start: Vec3::new(0.0, 0.0, 0.0),
        end: Vec3::new(0.0, 0.0, 1.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(0.0, 1.0, 0.0),
        end: Vec3::new(0.0, 1.0, 1.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(1.0, 1.0, 0.0),
        end: Vec3::new(1.0, 1.0, 1.0),
        color,
    });
    lines.push(GizmoLine {
        start: Vec3::new(1.0, 0.0, 0.0),
        end: Vec3::new(1.0, 0.0, 1.0),
        color,
    });
}
