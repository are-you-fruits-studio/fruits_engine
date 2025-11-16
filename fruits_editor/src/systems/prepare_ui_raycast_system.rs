use crate::*;

pub fn prepare_ui_raycast_system(
    button_q: WorldQuery<(Entity, &GlobalRectComponent), WithFilter<ButtonComponent>>,
    mut raycast_res: ResMut<UiRaycastResource>,
) {
    let iter = button_q.iter()
        .filter_map(|(ent, rect)| {
            if rect.scale.map(|v| v < 0.0).any() {
                return None;
            }

            Some((CollisionAabb {
                center: rect.center.xyn(rect.z),
                extents: (rect.scale * 0.5).xyn(1.0),
            }.into_shape(), ent))
        });

    raycast_res.bvh = Bvh::new(iter);
}
