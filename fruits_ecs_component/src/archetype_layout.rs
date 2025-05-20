use std::{
    any::TypeId,
    collections::HashMap
};

use super::{
    entity::Entity, type_info::TypeInfo, unique_components_set::UniqueComponentsSet, unsafe_archetype::{
        ArchetypeItemPhysicalLocation,
        CHUNK_SIZE,
    }
};

#[derive(Copy, Clone, Debug)]
pub struct ArchetypeItemLayout {
    pub type_info: TypeInfo,
    pub offset: usize,
    pub order: usize,
}

pub struct ArchetypeLayout {
    components_set: UniqueComponentsSet,
    components: HashMap<TypeId, ArchetypeItemLayout>,
    entities_per_chunk_count: usize,
}

impl ArchetypeLayout {
    pub fn from_components(components_set: UniqueComponentsSet) -> Self {
        let entities_per_chunk_count = Self::to_entities_per_chunk_count(&components_set);
        let entity_info = TypeInfo::new::<Entity>();

        let mut components = HashMap::new();
        let mut offset = 0_usize;

        offset += entity_info.size() * entities_per_chunk_count + entity_info.align().max(1) - 1;

        for (order, component_info) in components_set.component_infos().values().enumerate() {
            components.insert(*component_info.id(), ArchetypeItemLayout {
                offset,
                type_info: *component_info,
                order: order + 1,
            });

            offset += component_info.size() * entities_per_chunk_count + component_info.align().max(1) - 1;
        }

        Self {
            components_set,
            components,
            entities_per_chunk_count,
        }
    }

    fn to_entities_per_chunk_count(components_set: &UniqueComponentsSet) -> usize {
        let entity_info = TypeInfo::new::<Entity>();
        let components = components_set.component_infos().values().chain(std::iter::once(&entity_info));

        let padding_sum = components.clone().map(|i| i.align().max(1) - 1).sum::<usize>();
        let size_sum = components.map(|i| i.size()).sum::<usize>();

        (CHUNK_SIZE - padding_sum) / size_sum
    }

    pub fn components_set(&self) -> &UniqueComponentsSet {
        &self.components_set
    }

    pub fn entity_item_layout() -> ArchetypeItemLayout {
        ArchetypeItemLayout {
            offset: 0,
            order: 0,
            type_info: TypeInfo::new::<Entity>(),
        }
    }

    pub fn components(&self) -> &HashMap<TypeId, ArchetypeItemLayout> {
        &self.components
    }

    pub fn entities_per_chunk_count(&self) -> usize {
        self.entities_per_chunk_count
    }
    
    pub fn is_component_the_only_difference(with_component: &Self, without_component: &Self, component: &TypeId) -> bool {
        let with_component = with_component.components();
        let without_component = without_component.components();
        
        if with_component.len() != without_component.len() + 1 {
            return false;
        }

        if without_component.contains_key(component) {
            return false;
        }

        if !with_component.contains_key(component) {
            return false;
        }

        for component_id in without_component.keys() {
            if !with_component.contains_key(component_id) {
                return false;
            }
        }

        return true;
    }
}

impl ArchetypeLayout {
    pub fn chunk_index(&self, entity_in_archetype_index: usize) -> usize {
        entity_in_archetype_index / self.entities_per_chunk_count
    }

    pub fn entity_in_chunk_index(&self, entity_in_archetype_index: usize) -> usize {
        entity_in_archetype_index % self.entities_per_chunk_count
    }

    pub fn component_memory_physical_location(&self, entity_in_archetype_index: usize, component: &TypeId) -> ArchetypeItemPhysicalLocation {
        let item_layout = self.components.get(component).unwrap();

        self.memory_physical_location(entity_in_archetype_index, item_layout)
    }

    pub fn entity_memory_physical_location(&self, entity_in_archetype_index: usize) -> ArchetypeItemPhysicalLocation {
        let item_layout = ArchetypeLayout::entity_item_layout();

        self.memory_physical_location(entity_in_archetype_index, &item_layout)
    }

    fn memory_physical_location(&self, entity_in_archetype_index: usize, item_layout: &ArchetypeItemLayout) -> ArchetypeItemPhysicalLocation {
        let entity_in_chunk_index = self.entity_in_chunk_index(entity_in_archetype_index);
        let memory_size = item_layout.type_info.size();
        let memory_align = item_layout.type_info.align();
        let memory_offset = item_layout.offset + entity_in_chunk_index * item_layout.type_info.size();

        ArchetypeItemPhysicalLocation {
            chunk_index: self.chunk_index(entity_in_archetype_index),
            memory_offset,
            memory_size,
            memory_align,
        }
    }
}