use crate::*;

pub fn check_button_system(
    rect_q: WorldQuery<&GlobalRectComponent>,
    input: Res<InputResource>,
    raycast_res: Res<UiRaycastResource>,
    mut click_evt: EvtMut<ButtonClickEvent>,
) {
    let left_just_pressed = input.mouse.is_just_pressed(MouseButton::Left);
    let left_just_released = input.mouse.is_just_released(MouseButton::Left);

    if !left_just_pressed && !left_just_released {
        return;
    }

    let pos = Vec2::from_array(input.mouse.position.map(|v| v as f32));

    let mut hits = Vec::new();

    raycast_res.bvh.query(CollisionLine {
        bounds: LineBoundType::UNRESTRICTED,
        start: pos.xyn(0.0),
        end: pos.xyn(1.0),
    }.into(), &mut hits);

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
