use std::ffi::c_void;

use crate::*;

#[repr(C)]
pub struct ArchetypeUnsafeFfiIteratorItem {
    pub chunk_ptr: *mut u8,
    pub chunk_entity_idx: u64,
}

pub struct ArchetypeUnsafeFfiIterator<'a> {
    archetype: &'a ArchetypeUnsafeFfi,
    entity_archetype_idx: u64,
}

impl<'a> ArchetypeUnsafeFfiIterator<'a> {
    pub fn new(archetype: &'a ArchetypeUnsafeFfi) -> Self {
        Self {
            archetype,
            entity_archetype_idx: 0,
        }
    }
}

impl<'a> Iterator for ArchetypeUnsafeFfiIterator<'a> {
    type Item = ArchetypeUnsafeFfiIteratorItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entity_archetype_idx >= self.archetype.alive_entities_count {
            return None;
        }

        let chunk_entity = self
            .archetype
            .layout
            .entity_in_chunk_index(self.entity_archetype_idx);
        let chunk_ptr = self
            .archetype
            .archetype
            .get_chunk(self.archetype.layout.chunk_index(self.entity_archetype_idx));

        self.entity_archetype_idx += 1;

        Some(ArchetypeUnsafeFfiIteratorItem {
            chunk_ptr,
            chunk_entity_idx: chunk_entity,
        })
    }
}

//

// todo: drop components on Archetype drop?
#[repr(C)]
pub struct ArchetypeUnsafeFfi {
    layout: ArchetypeLayout,
    archetype: ArchetypeRaw,
    alive_entities_count: u64,
}

impl ArchetypeUnsafeFfi {
    pub fn new(types: TypesRegistryAccessFfi, components_set: UniqueComponentsSet) -> Self {
        Self {
            layout: ArchetypeLayout::new(types, components_set),
            archetype: ArchetypeRaw::new(),
            alive_entities_count: 0,
        }
    }

    pub fn contains_component_type(&self, type_id: u64) -> bool {
        self.layout.get_component(type_id).is_some()
    }

    pub fn iter<'a>(&'a self) -> ArchetypeUnsafeFfiIterator<'a> {
        ArchetypeUnsafeFfiIterator::new(&self)
    }

    pub fn entities_count(&self) -> u64 {
        self.alive_entities_count
    }

    pub fn get_entity(&self, entity_index: u64) -> Option<Entity> {
        if entity_index >= self.alive_entities_count {
            return None;
        }

        Some(unsafe {
            let physical_location = self.layout.entity_memory_physical_location(entity_index);

            let memory = self.archetype.get_memory(&physical_location);

            (memory as *const Entity).read()
        })
    }

    pub fn get_component_ptr(&self, entity_index: u64, component_id: u64) -> Option<*mut u8> {
        if entity_index >= self.alive_entities_count {
            return None;
        }

        if self.layout.get_component(component_id).is_none() {
            return None;
        }

        let physical_location = self
            .layout
            .component_memory_physical_location(entity_index, component_id);

        Some(self.archetype.get_memory(&physical_location))
    }

    fn create_place_for_entity(&mut self) -> u64 {
        // todo: initialize components
        let entity_in_archetype_index = self.alive_entities_count;

        if self.layout.chunk_index(entity_in_archetype_index) as u64
            >= self.archetype.chunks_count()
        {
            self.archetype.push_chunk();
        }

        self.alive_entities_count += 1;

        entity_in_archetype_index
    }

    pub fn create_entity(&mut self, entity: Entity) {
        let entity_in_archetype_index = self.create_place_for_entity();

        unsafe {
            let entity_location = self
                .layout
                .entity_memory_physical_location(entity_in_archetype_index);

            let memory = self.archetype.get_memory(&entity_location);

            (memory as *mut Entity).write(entity);
        }
    }

    /// Returns the last entity before the destroy.
    pub fn destroy_entity(&mut self, entity_index: u64) -> Option<Entity> {
        if entity_index >= self.alive_entities_count {
            return None;
        }

        for item_layout in self.layout.components() {
            let Some(drop_fn) = item_layout.type_data.data.drop_fn else {
                continue;
            };

            let item_layout = item_layout.into_item_layout();

            unsafe {
                let location = self
                    .layout
                    .memory_physical_location(entity_index, &item_layout);

                let memory = self.archetype.get_memory(&location);

                drop_fn(memory as *mut c_void);
            }
        }

        self.erase_entity(entity_index)
    }

    fn erase_entity(&mut self, entity_index: u64) -> Option<Entity> {
        // todo: Release the unneded chunks?
        if entity_index >= self.alive_entities_count {
            return None;
        }

        let last_index = self.alive_entities_count - 1;

        if entity_index != last_index {
            let last_components_locations =
                Self::get_items_locations_iter(&self.layout, last_index, &self.layout);
            let entity_components_locations =
                Self::get_items_locations_iter(&self.layout, entity_index, &self.layout);

            for ((src_location, size), (dst_location, _)) in
                last_components_locations.zip(entity_components_locations)
            {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        self.archetype.get_memory(&src_location),
                        self.archetype.get_memory(&dst_location),
                        size,
                    )
                }
            }
        }

        let last_entity_location = self.layout.entity_memory_physical_location(last_index);

        let last_entity = unsafe {
            let last_enity_memory = self.archetype.get_memory(&last_entity_location);

            (last_enity_memory as *const Entity).read()
        };

        self.alive_entities_count -= 1;

        Some(last_entity)
    }

    /// Returns the last entity from src archetype before the movement.
    ///
    /// # Safety
    ///
    /// Memory is automatically deallocated. Needs manual lifetime-management.
    pub unsafe fn add_component(
        src: &mut Self,
        dst: &mut Self,
        src_entity_index: u64,
        component_ptr: *mut u8,
        component_id: u64,
    ) -> Option<Entity> {
        if !ArchetypeLayout::is_component_the_only_difference(
            &dst.layout,
            &src.layout,
            component_id,
        ) {
            return None;
        }

        let dst_entity_index = dst.create_place_for_entity();

        let src_components_locations =
            Self::get_items_locations_iter(&src.layout, src_entity_index, &src.layout);
        let dst_components_locations =
            Self::get_items_locations_iter(&src.layout, dst_entity_index, &dst.layout);

        for ((src_location, size), (dst_location, _)) in
            src_components_locations.zip(dst_components_locations)
        {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.archetype.get_memory(&src_location),
                    dst.archetype.get_memory(&dst_location),
                    size,
                );
            }
        }

        let item_layout = dst.layout.get_component(component_id).unwrap();
        let added_component_location = dst
            .layout
            .memory_physical_location(dst_entity_index, item_layout);

        unsafe {
            let added_mem = dst.archetype.get_memory(&added_component_location);

            std::ptr::copy_nonoverlapping(
                component_ptr,
                added_mem,
                item_layout.type_data.size as usize,
            );
        }

        Some(src.erase_entity(src_entity_index).unwrap())
    }

    fn get_items_locations_iter<'l>(
        componens_layout: &'l ArchetypeLayout,
        entity_in_archetype_index: u64,
        memory_layout: &'l ArchetypeLayout,
    ) -> impl Iterator<Item = (ArchetypeItemPhysicalLocation, usize)> + 'l {
        componens_layout
            .components()
            .iter()
            .map(move |t| {
                let item_layout = memory_layout.get_component(t.type_data.id).unwrap();
                (
                    memory_layout.memory_physical_location(entity_in_archetype_index, item_layout),
                    item_layout.type_data.size as usize,
                )
            })
            .chain(std::iter::once({
                let item_layout = memory_layout.entity_item_layout();
                (
                    memory_layout.entity_memory_physical_location(entity_in_archetype_index),
                    item_layout.type_data.size as usize,
                )
            }))
    }

    /// Returns the last entity from src archetype before the movement.
    pub unsafe fn remove_component(
        src: &mut Self,
        dst: &mut Self,
        src_entity_index: u64,
        component_ptr: *mut u8,
        component_id: u64,
    ) -> Option<Entity> {
        if !ArchetypeLayout::is_component_the_only_difference(
            &src.layout,
            &dst.layout,
            component_id,
        ) {
            return None;
        }

        let dst_entity_index = dst.create_place_for_entity();

        let src_components_locations =
            Self::get_items_locations_iter(&dst.layout, src_entity_index, &src.layout);
        let dst_components_locations =
            Self::get_items_locations_iter(&dst.layout, dst_entity_index, &dst.layout);

        for ((src_location, size), (dst_location, _)) in
            src_components_locations.zip(dst_components_locations)
        {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    src.archetype.get_memory(&src_location),
                    dst.archetype.get_memory(&dst_location),
                    size,
                )
            }
        }

        // Safety: Managed by caller.
        unsafe {
            let item_layout = src.layout.get_component(component_id).unwrap();
            let removed_component_location = src
                .layout
                .memory_physical_location(dst_entity_index, item_layout);
            let removed_ptr = src.archetype.get_memory(&removed_component_location);

            std::ptr::copy_nonoverlapping(
                removed_ptr,
                component_ptr,
                item_layout.type_data.size as usize,
            );
        };

        Some(src.erase_entity(src_entity_index).unwrap())
    }

    pub fn layout(&self) -> &ArchetypeLayout {
        &self.layout
    }

    pub unsafe fn raw_archetype(&self) -> &ArchetypeRaw {
        &self.archetype
    }
}
