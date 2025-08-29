use std::collections::VecDeque;

use fruits_app::RenderStateResource;
use fruits_ecs::{Entity, ExclusiveWorldAccess, OrFilter, Res, WithFilter, WithoutFilter, WorldQuery};
use fruits_math::{Mat3, Vec2};

use crate::{render::{GlobalDisableableComponent, LocalDisableableComponent}, transform::{GlobalRectComponent, LocalRectComponent}, UiDirection};

use super::{ChildComponent, GlobalTransform, LocalTransform, ParentComponent};

pub fn adjust_component_sets(
    mut world: ExclusiveWorldAccess,
) {
    // todo: do the same with rects?
    let mut buffer = Vec::new();

    let entities_components = world.entities_components_mut();

    buffer.extend(entities_components.query_filtered::<Entity, (
        WithFilter<LocalTransform>,
        WithoutFilter<GlobalTransform>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.add_component(e, GlobalTransform::IDENTITY).ok().unwrap();
    }

    buffer.extend(entities_components.query_filtered::<Entity, (
        WithoutFilter<GlobalTransform>,
        WithoutFilter<LocalTransform>,
        WithFilter<ParentComponent>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.remove_component::<ParentComponent>(e).unwrap();
    }

    buffer.extend(entities_components.query_filtered::<Entity, (
        OrFilter<(WithFilter<GlobalTransform>, WithFilter<LocalTransform>, WithFilter<GlobalRectComponent>, WithFilter<LocalRectComponent>)>,
        WithoutFilter<ParentComponent>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.add_component(e, ParentComponent { children: Vec::new(), }).ok().unwrap();
    }

    buffer.extend(entities_components.query_filtered::<Entity, (
        WithoutFilter<LocalTransform>,
        WithoutFilter<LocalRectComponent>,
        WithFilter<ChildComponent>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.remove_component::<ChildComponent>(e).unwrap();
    }

    buffer.extend(entities_components.query_filtered::<Entity, (
        WithFilter<LocalTransform>,
        WithoutFilter<ChildComponent>,
    )>().iter());
    for e in buffer.drain(..) {
        entities_components.add_component(e, ChildComponent { parent: Entity::EMPTY }).ok().unwrap();
    }
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
    hierarchy_iter_depth_first_parent_to_child(&hierarchy_q, |parent, children| {
        for &ent in children {
            let parent_global_transform = match global_transform_q.get(parent) {
                None => GlobalTransform::IDENTITY,
                Some(&parent_global_transform) => parent_global_transform,
            };

            // todo: Check geometry operations
            // {

            let Some(&local_transform) = local_transform_q.get(ent) else {
                return;
            };
            let Some(global_transform) = global_transform_q.get_mut(ent) else {
                return;
            };
            global_transform.position = parent_global_transform.scale_rotation * local_transform.position + parent_global_transform.position;
            global_transform.scale_rotation = parent_global_transform.scale_rotation * (local_transform.rotation.to_matrix() * Mat3::scale(local_transform.scale));
            // }
        }
    });
}

// - Calculate GlobalRectComponent from LocalRectComponent and child-parent relation with tree-ordering from all the child leaves to a root parent.
pub fn precalculate_global_rect_hierarchy_independent(
    render_state: Res<RenderStateResource>,
    mut rect_q: WorldQuery<(&LocalRectComponent, &mut GlobalRectComponent)>,
) {
    let window_size: [u32; 2] = render_state.size().into();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    for (local_rect, global_rect) in rect_q.iter_mut() {
        global_rect.scale = LocalRectComponent::calculate_scale_hierarchy_independent(local_rect, window_size);
    }
}

// - Calculate GlobalRectComponent from LocalRectComponent and child-parent relation with tree-ordering from all the child leaves to a root parent.
pub fn precalculate_global_rect_children_based(
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    local_rect_q: WorldQuery<&LocalRectComponent>,
    mut global_rect_q: WorldQuery<&mut GlobalRectComponent>,
) {
    hierarchy_iter_depth_first_child_to_parent(&hierarchy_q, |parent, children| {
        let Some(&local_rect) = local_rect_q.get(parent) else {
            return;
        };
        let Some(_global_rect) = global_rect_q.get_mut(parent) else {
            return;
        };

        let mut child_based_scale = if local_rect.scale.as_array().iter().all(Option::is_some) {
            Vec2::with_all(0.0)
        } else {
            let mut max = Vec2::with_all(0.0);
            let mut sum = Vec2::with_all(0.0);

            for &child in children {
                let Some(child_transform) = global_rect_q.get(child) else {
                    continue;
                };

                max = max.zip_copied(child_transform.scale, f32::max);
                sum += child_transform.scale;
            }

            match local_rect.children_align {
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
pub fn calculate_global_rect_parent_based(
    render_state: Res<RenderStateResource>,
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    local_rect_q: WorldQuery<&LocalRectComponent>,
    mut global_rect_q: WorldQuery<&mut GlobalRectComponent>,
) {
    let window_size: [u32; 2] = render_state.size().into();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    hierarchy_iter_depth_first_parent_to_child(&hierarchy_q, |parent, children| {
        for &ent in children {
            let parent_global_rect = match global_rect_q.get(parent) {
                None => GlobalRectComponent { center: window_size * 0.5, scale: window_size, z: 0.0 },
                Some(&parent_global_rect) => parent_global_rect,
            };

            // todo: Check geometry operations
            // {
            let Some(&local_rect) = local_rect_q.get(ent) else {
                return;
            };
            let Some(global_rect) = global_rect_q.get_mut(ent) else {
                return;
            };
            *global_rect = LocalRectComponent::calculate_global_rect(&local_rect, &parent_global_rect, window_size, global_rect.scale);
            // }
        }
    });
}

// - Calculate GlobalDisableableComponent from LocalDisableableComponent and child-parent relation with tree-ordering from a root parent to all the child leaves.
pub fn calculate_global_disableable(
    hierarchy_q: WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    local_disableable_q: WorldQuery<&LocalDisableableComponent>,
    mut global_disableable_q: WorldQuery<&mut GlobalDisableableComponent>,
) {
    hierarchy_iter_depth_first_parent_to_child(&hierarchy_q, |parent, children| {
        for &ent in children {
            let parent_global_disableable = match global_disableable_q.get(parent) {
                None => GlobalDisableableComponent::default(),
                Some(&parent_global_transform) => parent_global_transform,
            };

            let Some(&local_disableable) = local_disableable_q.get(ent) else {
                return;
            };
            let Some(global_disableable) = global_disableable_q.get_mut(ent) else {
                return;
            };
            global_disableable.is_disabled = parent_global_disableable.is_disabled || local_disableable.is_disabled;
        }
    });
}

fn hierarchy_iter_breadth_first_parent_to_child(
    q: &WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    mut f: impl FnMut(Entity, Entity),
) {
    let mut ents_to_calc = q.iter()
        .filter_map(|(e, c, _p)| {
            let Some(child_component) = c else {
                return Some((e, Entity::EMPTY));
            };

            if q.get(child_component.parent).is_none() {
                return Some((e, Entity::EMPTY));
            }
            
            None
        })
        .collect::<VecDeque<_>>();

    while let Some((entity, parent)) = ents_to_calc.pop_front() {
        let entity_parent_c = q.get(entity).unwrap().2;

        if let Some(entity_parent_c) = entity_parent_c {
            for &child in entity_parent_c.children.iter() {
                if child == entity {
                    continue;
                }

                let Some(child_child_c) = q.get(child).unwrap().1 else {
                    continue;
                };

                if child_child_c.parent != entity {
                    continue;
                }

                ents_to_calc.push_back((child, entity));
            }
        };

        f(entity, parent);
    }
}

/// f(optional parent, children)
fn hierarchy_iter_depth_first_parent_to_child(
    q: &WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    mut f: impl FnMut(Entity, &[Entity]),
) {
    hierarchy_iter_depth_first(q, move |src, dst, is_moving_to_root| {
        if is_moving_to_root {
            // moving child_to_parent
            return;
        }

        if q.get(src).is_none() {
            // src does not exist - dst is root. Dst as child
            f(Entity::EMPTY, &[dst]);
        }
 
        let Some((_, _, p)) = q.get(dst) else {
            // dst does not exist - src is root. Imposible while moving parent_to_child
            return;
        };

        // dst as parent
        let children = p
            .map(|p| p.children.as_slice()).unwrap_or(&[]);

        f(dst, children);
    })
}

/// f(optional parent, children)
fn hierarchy_iter_depth_first_child_to_parent(
    q: &WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    mut f: impl FnMut(Entity, &[Entity]),
) {
    hierarchy_iter_depth_first(q, move |src, dst, is_moving_to_root| {
        if !is_moving_to_root {
            // moving parent_to_child
            return;
        }
 
        let Some((_, _, p)) = q.get(src) else {
            // src does not exist - dst is root. Imposible while moving child_to_parent
            return;
        };

        // src as parent
        let children = p
            .map(|p| p.children.as_slice()).unwrap_or(&[]);

        f(src, children);

        if q.get(dst).is_none() {
            // dst does not exist - src is root. Src as child
            f(Entity::EMPTY, &[src]);
        }
    })
}

/// f(src, dst, is_moving_to_root)
fn hierarchy_iter_depth_first(
    q: &WorldQuery<(Entity, Option<&ChildComponent>, Option<&ParentComponent>)>,
    mut f: impl FnMut(Entity, Entity, bool),
) {
    let roots = q.iter()
        .filter_map(|(e, c, _p)| {
            let Some(child_component) = c else {
                return Some(e);
            };

            if q.get(child_component.parent).is_none() {
                return Some(e);
            }
            
            None
        })
        .collect::<Vec<_>>();

    let get_children = |ent| -> &[Entity] {
        let Some(p) = q.get(ent).map(|t| t.2).flatten() else {
            return &[];
        };
        p.children.as_slice()
    };

    let mut stack = VecDeque::<(Entity, usize)>::new();

    for root in roots {
        f(Entity::EMPTY, root, false);

        let mut entity = root;
        let mut child_idx_to_check = 0;

        loop {
            let children = get_children(entity);

            if child_idx_to_check < children.len() {
                stack.push_back((entity, child_idx_to_check + 1));
                let child_entity = children[child_idx_to_check];
                f(entity, child_entity, false);
                entity = child_entity;
                child_idx_to_check = 0;
                continue;
            }

            // todo: handle root
            if let Some((parent_ent, parent_idx_to_check)) = stack.pop_back() {
                f(entity, parent_ent, true);
                entity = parent_ent;
                child_idx_to_check = parent_idx_to_check;
                continue;
            }

            f(entity, Entity::EMPTY, true);
            break;
        }
    }
}