use crate::*;

pub fn prepare_ui_raycast_system(
    button_q: WorldQuery<(Entity, &GlobalRectComponent), WithFilter<ButtonComponent>>,
    mut raycast_res: ResMut<UiRaycastResource>,
) {
    let mut entries = Vec::new();

    for (ent, rect) in button_q.iter() {
        if rect.scale.map(|v| v < 0.0).any() {
            continue;
        }

        let aabb = CollisionAabb {
            center: rect.center.xyn(rect.z),
            extents: (rect.scale * 0.5).xyn(1.0),
        };

        entries.push((aabb, ent));
    }

    raycast_res.bvh = Bvh::new(entries);
}
