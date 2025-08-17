use std::{any::TypeId, collections::{HashMap, HashSet}};

use fruits_utils::graph::Graph;

use crate::*;

pub struct SystemInfo {
    pub type_id: TypeId,
    pub system: Box<dyn System>,
}

impl std::fmt::Debug for SystemInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemInfo")
            .field("type_id", &self.type_id)
            .field("system", &self.system.system_name()).finish()
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum OrderEntry {
    System(TypeId),
    Group(&'static str),
}

pub fn create_ordering_graph(ordered_systems: &[SystemInfo], explicit_ordering: &HashSet<(TypeId, TypeId)>) -> OrderGraph {
    let system_index_by_type = ordered_systems.iter().enumerate().map(|(i, s)| (s.type_id, i)).collect::<HashMap<_, _>>();

    let mut system_by_data_readonly = HashMap::<TypeId, HashSet<usize>>::new();
    let mut system_by_data_mutable = HashMap::<TypeId, HashSet<usize>>::new();
    let mut systems_global_mutable = HashSet::<usize>::new();

    let mut analyzed_systems = HashSet::<usize>::new();

    let mut directions = vec![HashSet::<usize>::new(); ordered_systems.len()];

    for (previous_id, next_id) in explicit_ordering.iter() {
        let Some(&previous_index) = system_index_by_type.get(previous_id) else {
            continue;
        };
        
        let Some(&next_index) = system_index_by_type.get(next_id) else {
            continue;
        };

        directions[previous_index].insert(next_index);
    }

    for (system_index, system) in ordered_systems.iter().enumerate() {
        let mut data_usage = DataUsage::new();

        system.system.fill_data_usage(&mut data_usage);

        match data_usage {
            DataUsage::PerType(per_type_usage) => {
                for (type_id, DataUsageDetails { is_mutable, .. }) in per_type_usage.values().iter() {
                    if *is_mutable {
                        for &other_readonly_system_index in system_by_data_readonly.get(type_id).iter().flat_map(|m| m.iter()) {
                            directions[other_readonly_system_index].insert(system_index);
                        }
                        for &other_mutable_system_index in system_by_data_mutable.get(type_id).iter().flat_map(|m| m.iter()) {
                            directions[other_mutable_system_index].insert(system_index);
                        }
        
                        system_by_data_mutable.entry(*type_id).or_default().insert(system_index);
                    } else {
                        for &other_mutable_system_index in system_by_data_mutable.get(type_id).iter().flat_map(|m| m.iter()) {
                            directions[other_mutable_system_index].insert(system_index);
                        }
        
                        system_by_data_readonly.entry(*type_id).or_default().insert(system_index);
                    }
                }

                for &other_global_mutable_system_index in systems_global_mutable.iter() {
                    directions[other_global_mutable_system_index].insert(system_index);
                }
            },
            DataUsage::GlobalMutable => {
                for &other_system_index in analyzed_systems.iter() {
                    directions[other_system_index].insert(system_index);
                }

                systems_global_mutable.insert(system_index);
            }
        };

        analyzed_systems.insert(system_index);
    }

    let directions = directions.into_iter().map(|v| v.into_iter().collect::<Vec<_>>()).collect::<Vec<_>>();

    OrderGraph::new(directions).unwrap()
}

pub fn sort_systems_by_order(mut systems: HashMap<TypeId, Box<dyn System>>, systems_ordering: &HashSet<(TypeId, TypeId)>) -> Vec<SystemInfo> {
    let mut graph = Graph::new();

    for (src, dst) in systems_ordering.iter() {
        if systems.contains_key(src) && systems.contains_key(dst) {
            graph.insert_edge(*src, *dst);
        }
    }

    for ty in systems.keys() {
        graph.insert_node(*ty);
    }

    graph.to_vec().into_iter().map(|t| {SystemInfo {
        type_id: t,
        system: systems.remove(&t).unwrap(),
    }}).collect::<Vec<_>>()
}

pub fn flatten_ordering(
    ordering: &HashSet<(OrderEntry, OrderEntry)>,
    groups: &HashMap<&'static str, HashSet<OrderEntry>>,
) -> HashSet<(TypeId, TypeId)> {
    let flat_groups = flatten_groups(groups);

    let mut graph_orderer = HashSet::new();

    let mut single_value_min_buffer = HashSet::new();
    let mut single_value_max_buffer = HashSet::new();

    for (min, max) in ordering {
        let min_values = get_systems(*min, &flat_groups, &mut single_value_min_buffer);
        let max_values = get_systems(*max, &flat_groups, &mut single_value_max_buffer);
        
        for &min_value in min_values {
            for &max_value in max_values {
                graph_orderer.insert((min_value, max_value));
            }
        }
    }

    graph_orderer
}
fn get_systems<'a>(
    node: OrderEntry,
    flat_groups: &'a HashMap<&'static str, HashSet<TypeId>>,
    single_value_buffer: &'a mut HashSet<TypeId>,
) -> &'a HashSet<TypeId>
{
    match node {
        OrderEntry::Group(group) => &flat_groups[group],
        OrderEntry::System(system) => {
            single_value_buffer.clear();
            single_value_buffer.insert(system);
            single_value_buffer
        }
    }
}
fn flatten_groups(
    groups: &HashMap<&'static str, HashSet<OrderEntry>>,
) -> HashMap<&'static str, HashSet<TypeId>>
{
    let mut group_hierarchy = Graph::<&'static str>::new();

    for (&parent_group, group_children) in groups.iter() {
        group_hierarchy.insert_node(parent_group);
        
        for group_child in group_children {
            if let OrderEntry::Group(child_group) = group_child {
                group_hierarchy.insert_edge(parent_group, child_group);
            }
        }
    }

    let mut flat_groups = HashMap::<&'static str, HashSet<TypeId>>::new();

    for group in group_hierarchy.to_vec_rev() {
        let mut flat_group_children = HashSet::<TypeId>::new();
        
        for &group_child in groups.get(group).iter().copied().flatten() {
            match group_child {
                OrderEntry::System(child_handler) => _ = flat_group_children.insert(child_handler),
                OrderEntry::Group(child_group) => {
                    for &ele in &flat_groups[child_group] {
                        flat_group_children.insert(ele);
                    }
                },
            }
        }

        flat_groups.insert(group, flat_group_children);
    }

    flat_groups
}