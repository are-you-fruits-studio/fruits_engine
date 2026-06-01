use fruits_engine::*;

use crate::{SYSTEM_GROUP, features::ui_interaction::UiRaycastResource};

pub fn register_feature(mut world: WorldBuilderMut) {
    world
        .data_mut()
        .resources_mut()
        .insert(ScrollResource {
            active_scroll: EntityId::EMPTY,
        })
        .ok()
        .unwrap();

    let mut behavior = world.behavior_mut();
    let mut update = behavior.get_mut(Schedule::Update);

    update.group(SYSTEM_GROUP)
        .insert_child_system(start_end_scrolling_system)
        .insert_child_system(move_scroll_handle_system);

    update
        .order_system(start_end_scrolling_system)
        .before_system(move_scroll_handle_system);
}

#[derive(Component)]
pub struct ScrollHandleAreaComponent {
    pub content: EntityId,
    pub handle: EntityId,
}

#[derive(Resource)]
pub struct ScrollResource {
    pub active_scroll: EntityId,
}

fn start_end_scrolling_system(
    input: Res<InputResource>,
    mut scroll_res: ResMut<ScrollResource>,
    raycast_res: Res<UiRaycastResource>,
) {
    if input.mouse.is_just_released(MouseButton::Left) {
        scroll_res.active_scroll = EntityId::EMPTY;
    } else if input.mouse.is_just_pressed(MouseButton::Left) {
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

        return_if_not!(Some(clicked_ent) = hits.into_iter().next());

        scroll_res.active_scroll = clicked_ent;
    }
}

fn move_scroll_handle_system(
    input: Res<InputResource>,
    scroll_res: Res<ScrollResource>,
    mut handle_q: WorldQuery<(&ScrollHandleAreaComponent, EntityId)>,
    global_rect_q: WorldQuery<&GlobalRectComponent>,
    mut local_rect_q: WorldQuery<&mut LocalRectComponent>,
) {
    return_if_not!(Some((scroll_c, ent_area)) = handle_q.get_mut(scroll_res.active_scroll));

    let ent_handle = scroll_c.handle;

    {
        let mouse_pos = Vec2::from_array(input.mouse.position.map(|v| v as f32));

        return_if_not!(Some(area_rect) = global_rect_q.get(ent_area));
        return_if_not!(Some(handle_rect) = local_rect_q.get_mut(ent_handle));

        let min = area_rect.center.y - area_rect.scale.y * 0.5;
        let max = area_rect.center.y + area_rect.scale.y * 0.5;

        let progress = inv_lerp(min, max, mouse_pos.y);

        let progress = progress.clamp(0.0, 1.0);

        handle_rect.anchor.y = progress;
        handle_rect.pivot.y = progress;

        return_if_not!(Some(content_rect_c) = local_rect_q.get_mut(scroll_c.content));

        content_rect_c.anchor.y = progress;
        content_rect_c.pivot.y = progress;
    }
}
