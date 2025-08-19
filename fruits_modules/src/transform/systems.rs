use std::collections::VecDeque;

use fruits_app::RenderStateResource;
use fruits_ecs::{Entity, ExclusiveWorldAccess, OrFilter, WithFilter, WithoutFilter, WorldQuery};
use fruits_math::{Mat3, Vec2};

use crate::transform::{GlobalRectComponent, LocalRectComponent};

use super::{ChildComponent, GlobalTransform, LocalTransform, ParentComponent, UiVal};

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
    mut world: ExclusiveWorldAccess,
) {
    let ec = world.entities_components_mut();

    let mut transforms_to_calc = ec
        .query_filtered::<Entity, WithFilter<GlobalTransform>>()
        .iter()
        .filter(|e| {
            let Some(child_component) = ec.get_component::<ChildComponent>(*e) else {
                return true;
            };

            !ec.contains_entity(child_component.parent)
        })
        .collect::<VecDeque<_>>();

    while let Some(transform) = transforms_to_calc.pop_front() {
        let parent_global_transform = match ec.get_component::<ChildComponent>(transform) {
            None => GlobalTransform::IDENTITY,
            Some(child_component) => match ec.get_component::<GlobalTransform>(child_component.parent) {
                None => GlobalTransform::IDENTITY,
                Some(&parent_global_transform) => parent_global_transform,
            }
        };

        // todo: Check geometry operations
        // {
        let Some(&local_transform) = ec.get_component::<LocalTransform>(transform) else {
            continue;
        };
        let Some(global_transform) = ec.get_component_mut::<GlobalTransform>(transform) else {
            continue;
        };
        global_transform.position = parent_global_transform.scale_rotation * local_transform.position + parent_global_transform.position;
        global_transform.scale_rotation = parent_global_transform.scale_rotation * (local_transform.rotation.to_matrix() * Mat3::scale(local_transform.scale));
        // }

        let Some(children) = ec.get_component::<ParentComponent>(transform) else {
            continue;
        };

        for &child in children.children.iter() {
            if child == transform {
                continue;
            }

            let Some(child_child_c) = ec.get_component::<ChildComponent>(child) else {
                continue;
            };
            
            if child_child_c.parent != transform {
                continue;
            }

            transforms_to_calc.push_back(child);
        }
    }
}

// - Calculate GlobalRectComponent from LocalRectComponent and child-parent relation with tree-ordering from a root parent to all the child leaves.
pub fn calculate_global_rect(
    mut world: ExclusiveWorldAccess,
) {
    let (res, ec, _) = world.as_tuple_mut();

    let window_size: [u32; 2] = res.get::<RenderStateResource>().unwrap().size().into();
    let window_size = Vec2::from_array(window_size.map(|v| v as f32));

    let mut rects_to_calc = ec
        .query_filtered::<Entity, WithFilter<GlobalRectComponent>>()
        .iter()
        .filter(|e| {
            let Some(child_component) = ec.get_component::<ChildComponent>(*e) else {
                return true;
            };

            !ec.contains_entity(child_component.parent)
        })
        .collect::<VecDeque<_>>();

    while let Some(rect) = rects_to_calc.pop_front() {
        let parent_global_rect = match ec.get_component::<ChildComponent>(rect) {
            None => GlobalRectComponent { center: window_size * 0.5, scale: window_size, z: 0.0 },
            Some(child_component) => match ec.get_component::<GlobalRectComponent>(child_component.parent) {
                None => GlobalRectComponent { center: window_size * 0.5, scale: window_size, z: 0.0 },
                Some(&parent_global_rect) => parent_global_rect,
            }
        };

        // todo: Check geometry operations
        // {
        let Some(&local_rect) = ec.get_component::<LocalRectComponent>(rect) else {
            continue;
        };
        let Some(global_rect) = ec.get_component_mut::<GlobalRectComponent>(rect) else {
            continue;
        };
        *global_rect = LocalRectComponent::calculate_global_rect(&local_rect, &parent_global_rect, window_size);
        // }

        let Some(children) = ec.get_component::<ParentComponent>(rect) else {
            continue;
        };

        for &child in children.children.iter() {
            if child == rect {
                continue;
            }

            let Some(child_child_c) = ec.get_component::<ChildComponent>(child) else {
                continue;
            };
            
            if child_child_c.parent != rect {
                continue;
            }

            rects_to_calc.push_back(child);
        }
    }
}