use std::{any::TypeId, marker::PhantomData, mem::MaybeUninit};

use crate::{entity::{archetype_layout::ArchetypeLayout, unique_components_set::UniqueComponentsSet, archetype_unsafe::{ArchetypeUnsafeFfi, ArchetypeUnsafeIterator, ArchetypeUnsafeIteratorItem}}, meta::Entity, *};

pub trait Component : 'static { }

pub unsafe trait ArchetypeIteratorItem {
    type Item<'w>: 'w + ArchetypeIteratorItem;
    type ReadOnlyItem<'w>: 'w + ArchetypeIteratorItem;
    type IterState;
    
    unsafe fn prepare_iter_state(archetype: &Archetype) -> Self::IterState;
    unsafe fn next<'w>(archetype: &Archetype, item: &ArchetypeUnsafeIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w>;
    fn fill_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache);
}

unsafe impl<C: Component> ArchetypeIteratorItem for &C {
    type Item<'w> = &'w C;
    type ReadOnlyItem<'w> = &'w C;
    type IterState = u64;
    
    unsafe fn prepare_iter_state(archetype: &Archetype) -> Self::IterState {
        archetype
            .archetype
            .component_layout(archetype.types.get_or_register::<C>())
            .map(|i| i.chunk_offset)
            .unwrap_or(0)
    }

    unsafe fn next<'w>(_archetype: &Archetype, item: &ArchetypeUnsafeIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
        unsafe { &*(item.chunk_ptr.add(*iter_state as usize + item.chunk_entity_idx as usize * std::mem::size_of::<C>()) as *mut C) }
    }
    
    fn fill_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
        usage.add(DataUsageEntry {
            type_id: types.get_or_register::<C>(),
            details: DataUsageDetails {
                is_mutable: false,
                is_required: true,
            }
        });
    }
}

unsafe impl<C: Component> ArchetypeIteratorItem for &mut C {
    type Item<'w> = &'w mut C;
    type ReadOnlyItem<'w> = &'w C;
    type IterState = u64;
    
    unsafe fn prepare_iter_state(archetype: &Archetype) -> Self::IterState {
        archetype
            .archetype
            .component_layout(archetype.types.get_or_register::<C>())
            .map(|i| i.chunk_offset)
            .unwrap_or(0)
    }

    unsafe fn next<'w>(_archetype: &Archetype, item: &ArchetypeUnsafeIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
        unsafe { &mut *(item.chunk_ptr.add(*iter_state as usize + item.chunk_entity_idx as usize * std::mem::size_of::<C>()) as *mut C) }
    }
    
    fn fill_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
        usage.add(DataUsageEntry {
            type_id: types.get_or_register::<C>(),
            details: DataUsageDetails {
                is_mutable: true,
                is_required: true,
            }
        });
    }
}

unsafe impl<C: Component> ArchetypeIteratorItem for Option<&C> {
    type Item<'w> = Option<&'w C>;
    type ReadOnlyItem<'w> = Option<&'w C>;
    type IterState = Option<u64>;
    
    unsafe fn prepare_iter_state(archetype: &Archetype) -> Self::IterState {
        if archetype.contains_component_type(&TypeId::of::<C>()) {
            Some(archetype
                .archetype
                .component_layout(archetype.types.get_or_register::<C>())
                .map(|i| i.chunk_offset)
                .unwrap_or(0))
        } else {
            None
        }
    }

    unsafe fn next<'w>(_archetype: &Archetype, item: &ArchetypeUnsafeIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
        Some(unsafe { &*(item.chunk_ptr.add((*iter_state)? as usize + item.chunk_entity_idx as usize * std::mem::size_of::<C>()) as *mut C) })
    }
    
    fn fill_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
        usage.add(DataUsageEntry {
            type_id: types.get_or_register::<C>(),
            details: DataUsageDetails {
                is_mutable: false,
                is_required: false,
            }
        });
    }
}

unsafe impl<C: Component> ArchetypeIteratorItem for Option<&mut C> {
    type Item<'w> = Option<&'w mut C>;
    type ReadOnlyItem<'w> = Option<&'w C>;
    type IterState = Option<u64>;
    
    unsafe fn prepare_iter_state(archetype: &Archetype) -> Self::IterState {
        if archetype.contains_component_type(&TypeId::of::<C>()) {
            Some(archetype
                .archetype
                .component_layout(archetype.types.get_or_register::<C>())
                .map(|i| i.chunk_offset)
                .unwrap_or(0))
        } else {
            None
        }
    }
    unsafe fn next<'w>(_archetype: &Archetype, item: &ArchetypeUnsafeIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
        Some(unsafe { &mut *(item.chunk_ptr.add((*iter_state)? as usize + item.chunk_entity_idx as usize * std::mem::size_of::<C>()) as *mut C) })
    }
    
    fn fill_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
        usage.add(DataUsageEntry {
            type_id: types.get_or_register::<C>(),
            details: DataUsageDetails {
                is_mutable: true,
                is_required: false,
            }
        });
    }
}

unsafe impl ArchetypeIteratorItem for Entity {
    type Item<'w> = Entity;
    type ReadOnlyItem<'w> = Entity;
    type IterState = ();
    
    unsafe fn prepare_iter_state(_archetype: &Archetype) -> Self::IterState { }

    unsafe fn next<'w>(_archetype: &Archetype, item: &ArchetypeUnsafeIteratorItem, _iter_state: &mut Self::IterState) -> Self::Item<'w> {
        unsafe { (item.chunk_ptr.add(item.chunk_entity_idx as usize * std::mem::size_of::<Entity>()) as *mut Entity).read() }
    }
    
    fn fill_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
        usage.add(DataUsageEntry {
            type_id: types.get_or_register::<Entity>(),
            details: DataUsageDetails {
                is_mutable: false,
                is_required: true,
            }
        });
    }
    
}

macro_rules! archetype_iterator_item_impl {
    ($($P: ident),+) => {
        #[allow(unused_parens)]
        unsafe impl<$($P),+> ArchetypeIteratorItem for ($($P),+)
        where
            $($P: ArchetypeIteratorItem),+
        {
            type Item<'w> = (
                $($P::Item<'w>),+
            );
            type ReadOnlyItem<'w> = (
                $($P::ReadOnlyItem<'w>),+
            );
            type IterState = (
                $($P::IterState),+
            );

            unsafe fn prepare_iter_state(_archetype: &Archetype) -> Self::IterState {
                unsafe {
                    (
                        $($P::prepare_iter_state(_archetype)),+
                    )
                }
            }
            unsafe fn next<'w>(archetype: &Archetype, item: &ArchetypeUnsafeIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
                #[allow(non_snake_case)]
                let (
                    $($P),+
                ) = iter_state;

                unsafe {
                    (
                        $($P::next(archetype, item, $P)),+
                    )
                }
            }
            
            fn fill_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
                $($P::fill_usage(usage, types));+;
            }
        }
    };
}

archetype_iterator_item_impl!(P0, P1);
archetype_iterator_item_impl!(P0, P1, P2);
archetype_iterator_item_impl!(P0, P1, P2, P3);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6, P7);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13);
archetype_iterator_item_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14);


pub struct ArchetypeIterator<'a, A: ArchetypeIteratorItem> {
    iter: ArchetypeUnsafeIterator<'a>,
    archetype: &'a Archetype,
    iter_state: A::IterState,
    _phantom: PhantomData<&'a mut A>,
}

impl<'a, A: ArchetypeIteratorItem> ArchetypeIterator<'a, A> {
    pub unsafe fn new(archetype: &'a Archetype) -> Self {
        let iter_state = unsafe { A::prepare_iter_state(archetype) };
        
        Self {
            iter: archetype.archetype.iter(),
            archetype,
            iter_state,
            _phantom: Default::default(),
        }
    }
}

impl<'a, A: ArchetypeIteratorItem> Iterator for ArchetypeIterator<'a, A> {
    type Item = A::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(item) = self.iter.next() else {
            return None;
        };

        Some(unsafe { A::next(self.archetype, &item, &mut self.iter_state) })
    }
}

// todo: drop components on Archetype drop?



//

pub struct Archetype {
    archetype: ArchetypeUnsafeFfi,
    types: TypesRegistryCache,
}

impl Archetype {
    pub fn new(components_set: UniqueComponentsSet, types: TypesRegistryCache) -> Self {
        Self {
            archetype: ArchetypeUnsafeFfi::new_from_components(components_set),
            types,
        }
    }

    pub fn contains_component_type(&self, type_id: &TypeId) -> bool {
        let Some(type_id) = self.types.get_with_type_id(type_id) else {
            return false;
        };

        self.archetype.contains_component_type(type_id)
    }

    pub fn iter<'a, A: ArchetypeIteratorItem>(&'a self) -> ArchetypeIterator<'a, A> {
        // todo: threading guards
        unsafe {
            ArchetypeIterator::new(self)
        }
    }

    pub fn entities_count(&self) -> u64 {
        self.archetype.entities_count()
    }

    pub fn get_entity(&self, entity_index: u64) -> Option<Entity> {
        self.archetype.get_entity(entity_index)
    }

    pub fn get_component_ptr<C: 'static>(&self, entity_index: u64) -> Option<*mut C> {
        self.archetype.get_component_ptr(entity_index, self.types.get_or_register::<C>()).map(|p| p as *mut C)
    }

    pub fn create_entity(&mut self, entity: Entity) {
        self.archetype.create_entity(entity)
    }

    /// Returns the last entity before the destroy.
    pub fn destroy_entity(&mut self, entity_index: u64) -> Option<Entity> {
        self.archetype.destroy_entity(entity_index)
    }

    /// Returns the last entity from src archetype before the movement.
    pub fn add_component<C: 'static>(src: &mut Self, dst: &mut Self, src_entity_index: u64, component: C) -> Result<Entity, C> {
        unsafe {
            let mut component = MaybeUninit::new(component);

            if let Ok(entity) = ArchetypeUnsafeFfi::add_component(
                &mut src.archetype,
                &mut dst.archetype,
                src_entity_index,
                component.as_mut_ptr() as *mut u8,
                src.types.get_or_register::<C>(),
            ) {
                Ok(entity)
            } else {
                Err(component.assume_init())
            }
        }
    }

    /// Returns the last entity from src archetype before the movement.
    pub fn remove_component<C: 'static>(src: &mut Self, dst: &mut Self, src_entity_index: u64) -> Option<(Entity, C)> {
        unsafe {
            let mut component = MaybeUninit::<C>::uninit();

            if let Some(entity) = ArchetypeUnsafeFfi::remove_component(
                &mut src.archetype,
                &mut dst.archetype,
                src_entity_index,
                component.as_mut_ptr() as *mut u8,
                src.types.get_or_register::<C>(),
            ) {
                Some((entity, component.assume_init()))
            } else {
                None
            }
        }
    }

    pub unsafe fn unsafe_archetype(&self) -> &ArchetypeUnsafeFfi {
        &self.archetype
    }
}

//