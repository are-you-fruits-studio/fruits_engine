use std::{
    collections::HashMap, ffi::c_void
};

use fruits_ffi::{FfiDroppable, FfiVec};

use crate::*;

#[repr(C)]
struct ArchetypeLayoutItemsMap {
    data: FfiDroppable,
    get_fn: unsafe extern "C" fn(*const c_void, u64) -> *const ArchetypeItemLayout,
}

impl ArchetypeLayoutItemsMap {
    pub fn new(map: HashMap<u64, ArchetypeItemLayout>) -> Self {
        unsafe extern "C" fn ffi_get(this: *const c_void, key: u64) -> *const ArchetypeItemLayout  {
            unsafe {
                match (&*(this as *const HashMap<u64, ArchetypeItemLayout>)).get(&key) {
                    Some(l) => &raw const *l,
                    None => std::ptr::null_mut(),
                }
            }
        }

        Self {
            data: FfiDroppable::new(map),
            get_fn: ffi_get,
        }
    }

    pub fn get(&self, key: u64) -> Option<&ArchetypeItemLayout> {
        unsafe {
            let ptr = (self.get_fn)(self.data.get(), key);

            if ptr.is_null() {
                None
            } else {
                Some(&*ptr)
            }
        }
    }
}

//

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ArchetypeItemLayout {
    pub type_data: TypeData,
    pub chunk_offset: u64,
    pub order: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ArchetypeComponentLayout {
    pub type_data: StoredTypeData,
    pub chunk_offset: u64,
    pub order: u64,
}

impl ArchetypeComponentLayout {
    pub fn into_item_layout(self) -> ArchetypeItemLayout {
        ArchetypeItemLayout {
            chunk_offset: self.chunk_offset,
            order: self.order,
            type_data: self.type_data.data,
        }
    }
}

#[repr(C)]
pub struct ArchetypeLayout {
    types: TypesRegistryAccessFfi,
    components_list: FfiVec<ArchetypeComponentLayout>,
    components_map: ArchetypeLayoutItemsMap,
    components_set: UniqueComponentsSet,
    entities_per_chunk_count: u64,
}

impl ArchetypeLayout {
    pub fn new(types: TypesRegistryAccessFfi, components_set: UniqueComponentsSet) -> Self {
        let entity_data = TypeData::of::<Entity>();
        let entities_per_chunk_count = Self::to_entities_per_chunk_count(&components_set, &types);

        let mut components_map = HashMap::new();
        let mut components_list = FfiVec::new();
        let mut offset = 0;

        offset += entity_data.size * entities_per_chunk_count + entity_data.align.max(1) - 1;

        for (order, &component_id) in components_set.components().iter().enumerate() {
            let component_type = types.get(component_id).unwrap();

            let order = order as u64;
            let align = component_type.align;

            let overshoot = offset % align;

            let mut aligned_offset = offset;

            // todo: prohibit types with align >64.
            if overshoot > 0 {
                aligned_offset += align - overshoot;
            }

            components_map.insert(component_id, ArchetypeItemLayout {
                type_data: component_type,
                chunk_offset: aligned_offset,
                order: order + 1,
            });

            components_list.push(ArchetypeComponentLayout {
                type_data: StoredTypeData {
                    id: component_id,
                    data: component_type,
                },
                chunk_offset: aligned_offset,
                order: order + 1,
            });

            offset += component_type.size * entities_per_chunk_count + component_type.align.max(1) - 1;
        }

        Self {
            types,
            components_list,
            components_map: ArchetypeLayoutItemsMap::new(components_map),
            components_set,
            entities_per_chunk_count,
        }
    }

    fn to_entities_per_chunk_count(components_set: &UniqueComponentsSet, types: &TypesRegistryAccessFfi) -> u64 {
        let entity_data = TypeData::of::<Entity>();
        let components = components_set.components().iter().map(|t| types.get(*t).unwrap()).chain(std::iter::once(entity_data));

        let padding_sum = components.clone().map(|t| t.align.max(1) - 1).sum::<u64>();
        let size_sum = components.map(|t| t.size).sum::<u64>();

        (CHUNK_SIZE - padding_sum) / size_sum
    }

    pub fn components(&self) -> &[ArchetypeComponentLayout] {
        &self.components_list
    }

    pub fn components_set(&self) -> &UniqueComponentsSet {
        &self.components_set
    }

    pub fn entity_item_layout(&self) -> ArchetypeItemLayout {
        ArchetypeItemLayout {
            chunk_offset: 0,
            order: 0,
            type_data: TypeData::of::<Entity>(),
        }
    }

    pub fn get_component(&self, id: u64) -> Option<&ArchetypeItemLayout> {
        self.components_map.get(id)
    }

    pub fn entities_per_chunk_count(&self) -> u64 {
        self.entities_per_chunk_count
    }

    pub fn is_component_the_only_difference(with_component: &Self, without_component: &Self, component_id: u64) -> bool {
        if with_component.components().len() != without_component.components().len() + 1 {
            return false;
        }

        if without_component.get_component(component_id).is_some() {
            return false;
        }

        if with_component.get_component(component_id).is_none() {
            return false;
        }

        for without_type in without_component.components() {
            if with_component.get_component(without_type.type_data.id).is_none() {
                return false;
            }
        }

        true
    }
}

impl ArchetypeLayout {
    pub fn chunk_index(&self, entity_in_archetype_index: u64) -> u64 {
        entity_in_archetype_index / self.entities_per_chunk_count
    }

    pub fn entity_in_chunk_index(&self, entity_in_archetype_index: u64) -> u64 {
        entity_in_archetype_index % self.entities_per_chunk_count
    }

    pub fn component_memory_physical_location(&self, entity_in_archetype_index: u64, component_id: u64) -> ArchetypeItemPhysicalLocation {
        self.memory_physical_location(entity_in_archetype_index, self.components_map.get(component_id).unwrap())
    }

    pub fn component_line_memory_physical_location(&self, chunk_index: u64, component_id: u64) -> ArchetypeItemPhysicalLocation {
        self.memory_line_physical_location(chunk_index, self.components_map.get(component_id).unwrap())
    }

    pub fn entity_memory_physical_location(&self, entity_in_archetype_index: u64) -> ArchetypeItemPhysicalLocation {
        self.memory_physical_location(entity_in_archetype_index, &self.entity_item_layout())
    }

    pub fn entity_line_memory_physical_location(&self, chunk_index: u64) -> ArchetypeItemPhysicalLocation {
        self.memory_line_physical_location(chunk_index, &self.entity_item_layout())
    }

    pub fn memory_physical_location(&self, entity_in_archetype_index: u64, item_layout: &ArchetypeItemLayout) -> ArchetypeItemPhysicalLocation {
        let entity_in_chunk_index = self.entity_in_chunk_index(entity_in_archetype_index);
        let memory_offset = item_layout.chunk_offset + entity_in_chunk_index * item_layout.type_data.size;

        ArchetypeItemPhysicalLocation {
            chunk_index: self.chunk_index(entity_in_archetype_index),
            memory_offset,
        }
    }

    pub fn memory_line_physical_location(&self, chunk_index: u64, item_layout: &ArchetypeItemLayout) -> ArchetypeItemPhysicalLocation {
        ArchetypeItemPhysicalLocation {
            chunk_index,
            memory_offset: item_layout.chunk_offset,
        }
    }
}
