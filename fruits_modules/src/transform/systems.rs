use fruits_app::RenderStateResource;
use fruits_ecs::{Entity, ExclusiveWorldAccess, OrFilter, Res, WithFilter, WithoutFilter, WorldQuery};
use fruits_math::{Mat3, Vec2};

use crate::{render::{GlobalDisableableComponent, LocalDisableableComponent}, transform::{utils, GlobalRectComponent, LocalRectComponent}, RectChildAlignComponent, UiDirection, UiSpacing, UiVal};

use super::{ChildComponent, GlobalTransform, LocalTransform, ParentComponent};

pub fn adjust_component_sets(
    mut world: ExclusiveWorldAccess,
) {
    // todo: do the same with rects?
    let mut buffer = Vec::new();

    let entities_components = world.entities_components_mut();

    // local without global

    buffer.extend(entities_components.query_filtered::<Entity, (
        WithFilter<LocalTransform>,
        WithoutFilter<GlobalTransform>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.add_component(e, GlobalTransform::IDENTITY).ok().unwrap();
    }

    buffer.extend(entities_components.query_filtered::<Entity, (
        WithFilter<LocalRectComponent>,
        WithoutFilter<GlobalRectComponent>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.add_component(e, GlobalRectComponent::default()).ok().unwrap();
    }

    buffer.extend(entities_components.query_filtered::<Entity, (
        WithFilter<LocalDisableableComponent>,
        WithoutFilter<GlobalDisableableComponent>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.add_component(e, GlobalDisableableComponent::default()).ok().unwrap();
    }

    //

    // todo: remove?

    // buffer.extend(entities_components.query_filtered::<Entity, (
    //     WithoutFilter<GlobalTransform>,
    //     WithoutFilter<LocalTransform>,
    //     WithFilter<ParentComponent>,
    // )>().iter());
    // for e in buffer.drain(..) {
    //     entities_components.remove_component::<ParentComponent>(e).unwrap();
    // }

    buffer.extend(entities_components.query_filtered::<Entity, (
        OrFilter<(WithFilter<GlobalTransform>, WithFilter<GlobalRectComponent>, WithFilter<GlobalDisableableComponent>)>,
        WithoutFilter<ParentComponent>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.add_component(e, ParentComponent { children: Vec::new(), }).ok().unwrap();
    }

    //

    // buffer.extend(entities_components.query_filtered::<Entity, (
    //     WithoutFilter<LocalTransform>,
    //     WithoutFilter<LocalRectComponent>,
    //     WithFilter<ChildComponent>,
    // )>().iter());
    // for e in buffer.drain(..) {
    //     entities_components.remove_component::<ChildComponent>(e).unwrap();
    // }

    // buffer.extend(entities_components.query_filtered::<Entity, (
    //     WithFilter<LocalTransform>,
    //     WithoutFilter<ChildComponent>,
    // )>().iter());
    // for e in buffer.drain(..) {
    //     entities_components.add_component(e, ChildComponent { parent: Entity::EMPTY }).ok().unwrap();
    // }
}

// - Update ParentComponents according to ChildComponents
//     - Remove children from parent components
pub fn update_parents_remove_invalid_children(
    mut parents: WorldQuery<(Entity, &mut ParentComponent)>,
    children: WorldQuery<&ChildComponent>,
) {
    let mut indices_to_remove = Vec::new();
    
    for (parent_entity, parent) in parents.iter_mut() {
        for (index, &child_entity) in parent.children.iter().enumerate().rev() {
            if child_entity == parent_entity {
                indices_to_remove.push(index);
                continue;
            }
            let Some(child_child_c) = children.get(child_entity) else {
                indices_to_remove.push(index);
                continue;
            };
            if child_child_c.parent != parent_entity {
                indices_to_remove.push(index);
            }
        }

        for &index in indices_to_remove.iter() {
            parent.children.remove(index);
        }

        indices_to_remove.clear();
    }
}

// - Update ParentComponents according to ChildComponents
//     - Add missing children to parent components with creation if needed
pub fn update_parents_add_missing_children(
    mut world: ExclusiveWorldAccess,
) {
    let ec = world.entities_components_mut();

    let children = ec
        .query::<(Entity, &ChildComponent)>()
        .iter()
        .map(|(e, c)| (e, c.parent))
        .filter(|(_, pe)| ec.get_component::<ParentComponent>(*pe).is_some())
        .collect::<Vec<_>>();

    for (child_entity, parent_entity) in children.into_iter() {
        let parent = ec.get_component_mut::<ParentComponent>(parent_entity).unwrap();

        if !parent.children.contains(&child_entity) {
            parent.children.push(child_entity);
        }
    }
}

// - Calculate GlobalTransform from LocalTransform and child-parent relation with tree-ordering from a root parent to all the child leaves.
pub fn calculate_global_transform(
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    local_transform_q: WorldQuery<&LocalTransform>,
    mut global_transform_q: WorldQuery<&mut GlobalTransform>,
) {
    utils::hierarchy_iter_depth_first_parent_to_child(&hierarchy_q, |parent, children| {
        for &ent in children {
            let parent_global_transform = match global_transform_q.get(parent) {
                None => GlobalTransform::IDENTITY,
                Some(&parent_global_transform) => parent_global_transform,
            };

            let Some(&local_transform) = local_transform_q.get(ent) else {
                continue;
            };
            let Some(global_transform) = global_transform_q.get_mut(ent) else {
                continue;
            };

            global_transform.position = parent_global_transform.scale_rotation * local_transform.position + parent_global_transform.position;
            global_transform.scale_rotation = parent_global_transform.scale_rotation * (local_transform.rotation.to_matrix() * Mat3::scale(local_transform.scale));
        }
    });
}

// - Calculate GlobalRectComponent from LocalRectComponent and child-parent relation with tree-ordering from all the child leaves to a root parent.
pub fn calculate_global_rect_scale_hierarchy_independent(
    render_state: Res<RenderStateResource>,
    mut rect_q: WorldQuery<(&LocalRectComponent, &mut GlobalRectComponent)>,
) {
    let window_size: [u32; 2] = render_state.size().into();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    for (local_rect, global_rect) in rect_q.iter_mut() {
        let scale = Vec2::from_fn(|i| {
            local_rect.scale[i].map(|v| v.into_px_without_parent(window_size).map(|v| v[i]))
                .flatten()
                .unwrap_or(0.0)
        });

        global_rect.scale = scale;
    }
}

// - Calculate GlobalRectComponent from LocalRectComponent and child-parent relation with tree-ordering from all the child leaves to a root parent.
pub fn calculate_global_rect_scale_children_based(
    render_state: Res<RenderStateResource>,
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    local_rect_q: WorldQuery<(&LocalRectComponent, Option<&RectChildAlignComponent>)>,
    mut global_rect_q: WorldQuery<&mut GlobalRectComponent>,
) {
    let window_size: [i32; 2] = render_state.size().into();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    utils::hierarchy_iter_depth_first_child_to_parent(&hierarchy_q, |parent, children| {
        let Some((&local_rect, align_c)) = local_rect_q.get(parent) else {
            return;
        };
        let Some(&parent_global_rect) = global_rect_q.get(parent) else {
            return;
        };

        let mut child_based_scale = if local_rect.scale.as_array().iter().all(Option::is_some) {
            Vec2::splat(0.0)
        } else {
            let mut max = Vec2::splat(0.0);
            let mut sum = Vec2::splat(0.0);
            let mut count = 0;

            for &child in children {
                let Some(child_transform) = global_rect_q.get(child) else {
                    continue;
                };

                count += 1;
                max = max.zip_copied(child_transform.scale, f32::max);
                sum += child_transform.scale;
            }

            let uival_into_px = |v: UiVal| -> Vec2<f32> {
                v.into_px(parent_global_rect.scale, window_size)
            };

            if let Some(align_c) = align_c {
                let gaps_count = count.max(1) - 1;
                let total_gap = match align_c.spacing {
                    UiSpacing::Chunk => uival_into_px(align_c.min_gap) * gaps_count as f32,
                    UiSpacing::SpaceBetween => uival_into_px(align_c.min_gap) * gaps_count as f32,
                    UiSpacing::SpaceAround => uival_into_px(align_c.min_gap) * (gaps_count + 1) as f32,
                    UiSpacing::SpaceEvenly => uival_into_px(align_c.min_gap) * (gaps_count + 2) as f32,
                };
                sum += total_gap;
            }

            match align_c.map(|a| a.direction) {
                None => max,
                Some(UiDirection::Horizontal) => Vec2::new(sum.x, max.y),
                Some(UiDirection::Vertical) => Vec2::new(max.x, sum.y),
            }
        };

        let Some(global_rect) = global_rect_q.get_mut(parent) else {
            return;
        };

        for (i, s) in local_rect.scale.as_array().iter().enumerate() {
            if s.is_some() {
                child_based_scale[i] = global_rect.scale[i];
            }
        }

        global_rect.scale = child_based_scale;
    });
}

// - Calculate GlobalRectComponent from LocalRectComponent and child-parent relation with tree-ordering from a root parent to all the child leaves.
pub fn calculate_global_rect_scale_parent_based(
    render_state: Res<RenderStateResource>,
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    local_rect_q: WorldQuery<&LocalRectComponent>,
    mut global_rect_q: WorldQuery<&mut GlobalRectComponent>,
) {
    let window_size: [u32; 2] = render_state.size().into();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    utils::hierarchy_iter_depth_first_parent_to_child(&hierarchy_q, |parent, children| {
        if children.is_empty() {
            return;
        }

        for &ent in children {
            let parent_global_rect = global_rect_q.get(parent).copied().unwrap_or(GlobalRectComponent { center: window_size * 0.5, scale: window_size, z: 0.0 });

            let Some(local_rect) = local_rect_q.get(ent) else {
                continue;
            };
            let Some(global_rect) = global_rect_q.get_mut(ent) else {
                continue;
            };

            let parent_min = parent_global_rect.center - parent_global_rect.scale * 0.5;
            let parent_max = parent_global_rect.center + parent_global_rect.scale * 0.5;

            let ui_val_to_px = |v: UiVal| -> Vec2<f32> {
                v.into_px(parent_global_rect.scale, window_size)
            };
            
            let parent_min = parent_min + Vec2::from_fn(|i| ui_val_to_px(local_rect.parent_padding_min[i])[i]);
            let parent_max = parent_max - Vec2::from_fn(|i| ui_val_to_px(local_rect.parent_padding_max[i])[i]);

            let padded_parent_scale = parent_max - parent_min;

            let ui_val_to_px = |v: UiVal| -> Vec2<f32> {
                v.into_px(padded_parent_scale, window_size)
            };

            let final_scale = Vec2::from_fn(|i| {
                local_rect.scale[i]
                    .map(|v| ui_val_to_px(v)[i])
                    .unwrap_or(global_rect.scale[i])
            });

            let anchored_pos = parent_min.lerp_separately(parent_max, local_rect.anchor);
            let offset_pos = anchored_pos + Vec2::from_fn(|i| ui_val_to_px(local_rect.offset[i])[i]);
            let pivoted_center = offset_pos + final_scale * (Vec2::splat(0.5) - local_rect.pivot);

            global_rect.scale = final_scale;
            global_rect.z = parent_global_rect.z + local_rect.z;
            global_rect.center = pivoted_center;
        }
    });
}

pub fn calculate_global_rect_pos(
    render_state: Res<RenderStateResource>,
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    local_rect_q: WorldQuery<&LocalRectComponent>,
    align_q: WorldQuery<&RectChildAlignComponent>,
    mut global_rect_q: WorldQuery<&mut GlobalRectComponent>,
) {
    let window_size: [i32; 2] = render_state.size().into();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    utils::hierarchy_iter_depth_first_parent_to_child(&hierarchy_q, |parent, children| {
        if children.is_empty() {
            return;
        }

        let align_data = 'switch: {
            let Some(align_c) = align_q.get(parent) else {
                break 'switch None;
            };

            let Some(&parent_global_c) = global_rect_q.get(parent) else {
                break 'switch None;
            };

            
            Some((align_c, parent_global_c))
        };

        match align_data {
            None => {
                for &ent in children {
                    let parent_global_rect = global_rect_q.get(parent).copied().unwrap_or(GlobalRectComponent { center: window_size * 0.5, scale: window_size, z: 0.0 });

                    let Some(local_rect) = local_rect_q.get(ent) else {
                        continue;
                    };
                    let Some(global_rect) = global_rect_q.get_mut(ent) else {
                        continue;
                    };

                    let parent_min = parent_global_rect.center - parent_global_rect.scale * 0.5;
                    let parent_max = parent_global_rect.center + parent_global_rect.scale * 0.5;

                    let ui_val_to_px = |v: UiVal| -> Vec2<f32> {
                        v.into_px(parent_global_rect.scale, window_size)
                    };

                    let parent_min = parent_min + Vec2::from_fn(|i| ui_val_to_px(local_rect.parent_padding_min[i])[i]);
                    let parent_max = parent_max - Vec2::from_fn(|i| ui_val_to_px(local_rect.parent_padding_max[i])[i]);

                    let padded_parent_scale = parent_max - parent_min;

                    let ui_val_to_px = |v: UiVal| -> Vec2<f32> {
                        v.into_px(padded_parent_scale, window_size)
                    };

                    let anchored_pos = parent_min.lerp_separately(parent_max, local_rect.anchor);
                    let offset_pos = anchored_pos + Vec2::from_fn(|i| ui_val_to_px(local_rect.offset[i])[i]);
                    let pivoted_center = offset_pos + global_rect.scale * (Vec2::splat(0.5) - local_rect.pivot);

                    global_rect.z = parent_global_rect.z + local_rect.z;
                    global_rect.center = pivoted_center;
                }
            },
            Some((align_c, parent_global_c)) => {
                let dir_axis = align_c.direction.to_axis_idx();
                let dir_perp_axis = 1 - dir_axis;

                let mut children_count = 0;
                let mut children_scale = Vec2::splat(0.0);

                for &child in children {
                    let Some(child_global_c) = global_rect_q.get_mut(child) else {
                        continue;
                    };

                    children_count += 1;
                    children_scale[dir_axis] += child_global_c.scale[dir_axis];
                    children_scale[dir_perp_axis] = children_scale[dir_perp_axis].max(child_global_c.scale[dir_perp_axis]);
                }

                let gaps_count = children_count - 1;

                let uival_into_px = |v: UiVal| -> Vec2<f32> {
                    v.into_px(parent_global_c.scale, window_size)
                };

                let children_scale_with_gaps;
                let gap;
                let pre_gap;

                match align_c.spacing {
                    UiSpacing::Chunk => {
                        let mut final_scale = children_scale;
                        final_scale[dir_axis] += uival_into_px(align_c.min_gap)[dir_axis] * gaps_count as f32;

                        children_scale_with_gaps = final_scale;
                        gap = uival_into_px(align_c.min_gap)[dir_axis];
                        pre_gap = 0.0;
                    },
                    UiSpacing::SpaceBetween => {
                        let mut final_scale = children_scale;
                        final_scale[dir_axis] += uival_into_px(align_c.min_gap)[dir_axis] * gaps_count as f32;
                        final_scale[dir_axis] = final_scale[dir_axis].max(parent_global_c.scale[dir_axis]);

                        children_scale_with_gaps = final_scale;
                        gap = (final_scale[dir_axis] - children_scale[dir_axis]) / gaps_count as f32;
                        pre_gap = 0.0;
                    },
                    UiSpacing::SpaceAround => {
                        let mut final_scale = children_scale;
                        final_scale[dir_axis] += uival_into_px(align_c.min_gap)[dir_axis] * (gaps_count + 1) as f32;
                        final_scale[dir_axis] = final_scale[dir_axis].max(parent_global_c.scale[dir_axis]);

                        children_scale_with_gaps = final_scale;
                        gap = (final_scale[dir_axis] - children_scale[dir_axis]) / (gaps_count + 1) as f32;
                        pre_gap = gap * 0.5;
                    },
                    UiSpacing::SpaceEvenly => {
                        let mut final_scale = children_scale;
                        final_scale[dir_axis] += uival_into_px(align_c.min_gap)[dir_axis] * (gaps_count + 2) as f32;
                        final_scale[dir_axis] = final_scale[dir_axis].max(parent_global_c.scale[dir_axis]);

                        children_scale_with_gaps = final_scale;
                        gap = (final_scale[dir_axis] - children_scale[dir_axis]) / (gaps_count + 2) as f32;
                        pre_gap = gap;
                    },
                }

                let children_center_dir = parent_global_c.center[dir_axis] + (children_scale_with_gaps[dir_axis] - parent_global_c.scale[dir_axis]) * (0.5 - align_c.anchor[dir_axis]);
                let children_start_dir = children_center_dir - children_scale_with_gaps[dir_axis] * 0.5;

                let mut child_start_dir = children_start_dir + pre_gap;

                for &child in children {
                    let Some(child_global_c) = global_rect_q.get_mut(child) else {
                        continue;
                    };

                    let child_center_perp_dir = parent_global_c.center[dir_perp_axis] + (child_global_c.scale[dir_perp_axis] - parent_global_c.scale[dir_perp_axis]) * (0.5 - align_c.anchor[dir_perp_axis]);

                    if child_global_c.scale[dir_axis] < 0.0 {
                        dbg!(child_global_c.scale[dir_axis]); 
                    }
                    child_global_c.center[dir_axis] = child_start_dir + child_global_c.scale[dir_axis] * 0.5;
                    child_global_c.center[dir_perp_axis] = child_center_perp_dir;

                    let local_z = local_rect_q.get(child).map(|l| l.z).unwrap_or(0.0);
                    child_global_c.z = parent_global_c.z + local_z;

                    child_start_dir += child_global_c.scale[dir_axis] + gap;
                }
            },
        }
    });
}

// - Calculate GlobalDisableableComponent from LocalDisableableComponent and child-parent relation with tree-ordering from a root parent to all the child leaves.
pub fn calculate_global_disableable(
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    local_disableable_q: WorldQuery<&LocalDisableableComponent>,
    mut global_disableable_q: WorldQuery<&mut GlobalDisableableComponent>,
) {
    utils::hierarchy_iter_depth_first_parent_to_child(&hierarchy_q, |parent, children| {
        for &ent in children {
            let parent_global_disableable = global_disableable_q.get(parent).copied().unwrap_or_default();

            let Some(&local_disableable) = local_disableable_q.get(ent) else {
                continue;
            };
            let Some(global_disableable) = global_disableable_q.get_mut(ent) else {
                continue;
            };

            global_disableable.is_disabled = parent_global_disableable.is_disabled || local_disableable.is_disabled;
        }
    });
}