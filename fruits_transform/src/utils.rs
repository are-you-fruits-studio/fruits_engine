use std::collections::VecDeque;

use fruits_ecs::{EntitiesHolderMut, EntityId, QueryFilter, WorldQuery};

use crate::{ChildComponent, ParentComponent};

pub fn hierarchy_iter_breadth_first_parent_to_child(
    q: &WorldQuery<(EntityId, Option<&ChildComponent>, Option<&ParentComponent>)>,
    mut f: impl FnMut(EntityId, EntityId),
) {
    let mut ents_to_calc = q
        .iter()
        .filter_map(|(e, c, _p)| {
            let Some(child_component) = c else {
                return Some((e, EntityId::EMPTY));
            };

            if q.get(child_component.parent).is_none() {
                return Some((e, EntityId::EMPTY));
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
pub fn hierarchy_iter_depth_first_parent_to_child<R, F: QueryFilter>(
    q: &WorldQuery<(EntityId, Option<&ChildComponent>, Option<&ParentComponent>), F>,
    mut f: impl FnMut(EntityId, &[EntityId]) -> R,
) {
    hierarchy_iter_depth_first(q, move |src, dst, is_moving_to_root| {
        if is_moving_to_root {
            // moving child_to_parent
            return;
        }

        if q.get(src).is_none() {
            // src does not exist - dst is root. Dst as child
            f(EntityId::EMPTY, &[dst]);
        }

        let Some((_, _, p)) = q.get(dst) else {
            // dst does not exist - src is root. Imposible while moving parent_to_child
            return;
        };

        // dst as parent
        let children = p.map(|p| p.children.as_slice()).unwrap_or(&[]);

        f(dst, children);
    })
}

/// f(optional parent, children)
pub fn hierarchy_iter_depth_first_child_to_parent<R, F: QueryFilter>(
    q: &WorldQuery<(EntityId, Option<&ChildComponent>, Option<&ParentComponent>), F>,
    mut f: impl FnMut(EntityId, &[EntityId]) -> R,
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
        let children = p.map(|p| p.children.as_slice()).unwrap_or(&[]);

        f(src, children);

        if q.get(dst).is_none() {
            // dst does not exist - src is root. Src as child
            f(EntityId::EMPTY, &[src]);
        }
    })
}

/// f(src, dst, is_moving_to_root)
pub fn hierarchy_iter_depth_first<R, F: QueryFilter>(
    q: &WorldQuery<(EntityId, Option<&ChildComponent>, Option<&ParentComponent>), F>,
    mut f: impl FnMut(EntityId, EntityId, bool) -> R,
) {
    let roots = q
        .iter()
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

    let get_children = |ent| -> &[EntityId] {
        let Some(p) = q.get(ent).map(|t| t.2).flatten() else {
            return &[];
        };
        p.children.as_slice()
    };

    let mut stack = VecDeque::<(EntityId, usize)>::new();

    for root in roots {
        f(EntityId::EMPTY, root, false);

        let mut entity = root;
        let mut child_idx_to_check = 0;

        loop {
            let children = get_children(entity);

            if let Some(&child_entity) = children.get(child_idx_to_check) {
                stack.push_back((entity, child_idx_to_check + 1));
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

            f(entity, EntityId::EMPTY, true);
            break;
        }
    }
}

pub fn destroy_entity_and_children(mut ec: EntitiesHolderMut, ent: EntityId) {
    destroy_entity_children(ec.as_mut(), ent);

    ec.destroy_entity(ent);
}

pub fn destroy_entity_children(mut ec: EntitiesHolderMut, ent: EntityId) {
    let mut check_buffer = Vec::new();

    if let Some(parent) = ec.get_component::<ParentComponent>(ent) {
        check_buffer.extend(parent.children.iter());
    };

    while let Some(ent) = check_buffer.pop() {
        if let Some(parent) = ec.get_component::<ParentComponent>(ent) {
            check_buffer.extend(parent.children.iter());
        };

        ec.destroy_entity(ent);
    }
}
