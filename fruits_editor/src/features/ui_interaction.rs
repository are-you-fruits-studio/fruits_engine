use fruits_engine::*;

use crate::SYSTEM_GROUP;

pub fn register_feature(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(UiRaycastResource::default());

    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update
        .group(SYSTEM_GROUP)
        .insert_child_system(prepare_ui_raycast_system)
        .insert_child_system(check_button_system);

    update
        .order_system(prepare_ui_raycast_system)
        .before_system(check_button_system);
}

#[derive(Resource, Default)]
pub struct UiRaycastResource {
    pub bvh: Bvh<EntityId>,
}

#[derive(Component, Debug, Clone)]
pub struct ButtonComponent;

#[derive(Event)]
pub struct ButtonClickEvent {
    pub entity: EntityId,
}

pub fn prepare_ui_raycast_system(
    button_q: WorldQuery<(EntityId, &GlobalRectComponent, Option<&GlobalDisableableComponent>), WithFilter<ButtonComponent>>,
    mut raycast_res: ResMut<UiRaycastResource>,
) {
    let iter = button_q.iter().filter_map(|(ent, rect, disableable)| {
        if disableable.copied().unwrap_or_default().is_disabled {
            return None;
        }

        if rect.scale.map(|v| v < 0.0).any() {
            return None;
        }

        Some((
            CollisionAabb {
                center: rect.center.xyn(rect.z),
                extents: (rect.scale * 0.5).xyn(1.0),
            }
            .into_shape(),
            ent,
        ))
    });

    raycast_res.bvh = Bvh::new(iter);
}

pub fn check_button_system(
    rect_q: WorldQuery<&GlobalRectComponent>,
    input: Res<InputResource>,
    raycast_res: Res<UiRaycastResource>,
    mut click_evt: EvtMut<ButtonClickEvent>,
) {
    let left_just_pressed = input.mouse.is_just_pressed(MouseButton::Left);
    let left_just_released = input.mouse.is_just_released(MouseButton::Left);

    if !left_just_pressed {
        return;
    }

    let pos = Vec2::from_array(input.mouse.position.map(|v| v as f32));

    let mut hits = Vec::new();

    raycast_res.bvh.query(
        CollisionLine {
            bounds: LineBoundType::UNRESTRICTED,
            start: pos.xyn(0.0),
            end: pos.xyn(1.0),
        }
        .into(),
        &mut hits,
    );

    let mut min_z = f32::INFINITY;
    let mut closest_ent = None;

    for &hit in &hits {
        let Some(rect) = rect_q.get(hit) else {
            continue;
        };

        if rect.z < min_z {
            min_z = rect.z;
            closest_ent = Some(hit);
        }
    }

    let Some(target_ent) = closest_ent else {
        return;
    };

    click_evt.push(ButtonClickEvent { entity: target_ent });
}
