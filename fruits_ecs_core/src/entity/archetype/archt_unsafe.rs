use std::{any::TypeId, marker::PhantomData, mem::MaybeUninit};

use crate::{entity::{archetype::{archt_ffi::{ArchetypeUnsafeFfi, ArchetypeUnsafeFfiIterator, ArchetypeUnsafeFfiIteratorItem}, archt_layout::ArchetypeLayout}, unique_components_set::UniqueComponentsSet}, meta::Entity, *};

pub trait Component : 'static { }

pub unsafe trait ArchetypeIteratorItem {
    type Item<'w>: 'w + ArchetypeIteratorItem;
    type ReadOnlyItem<'w>: 'w + ArchetypeIteratorItem;
    type IterState;
    
    unsafe fn prepare_iter_state(layout: &ArchetypeLayout, types: &TypesRegistryCache) -> Self::IterState;
    unsafe fn next<'w>(item: &ArchetypeUnsafeFfiIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w>;
    fn fill_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache);
}

unsafe impl<C: Component> ArchetypeIteratorItem for &C {
    type Item<'w> = &'w C;
    type ReadOnlyItem<'w> = &'w C;
    type IterState = u64;
    
    unsafe fn prepare_iter_state(layout: &ArchetypeLayout, types: &TypesRegistryCache) -> Self::IterState {
        layout
            .get_component(types.get_or_register::<C>())
            .map(|i| i.chunk_offset)
            .unwrap_or(0)
    }

    unsafe fn next<'w>(item: &ArchetypeUnsafeFfiIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
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
    
    unsafe fn prepare_iter_state(layout: &ArchetypeLayout, types: &TypesRegistryCache) -> Self::IterState {
        layout
            .get_component(types.get_or_register::<C>())
            .map(|i| i.chunk_offset)
            .unwrap_or(0)
    }

    unsafe fn next<'w>(item: &ArchetypeUnsafeFfiIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
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
    
    unsafe fn prepare_iter_state(layout: &ArchetypeLayout, types: &TypesRegistryCache) -> Self::IterState {
        layout
            .get_component(types.get_or_register::<C>())
            .map(|t| t.chunk_offset)
    }

    unsafe fn next<'w>(item: &ArchetypeUnsafeFfiIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
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
    
    unsafe fn prepare_iter_state(layout: &ArchetypeLayout, types: &TypesRegistryCache) -> Self::IterState {
        layout
            .get_component(types.get_or_register::<C>())
            .map(|i| i.chunk_offset)
    }
    unsafe fn next<'w>(item: &ArchetypeUnsafeFfiIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
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
    
    unsafe fn prepare_iter_state(_layout: &ArchetypeLayout, _types: &TypesRegistryCache) -> Self::IterState { }

    unsafe fn next<'w>(item: &ArchetypeUnsafeFfiIteratorItem, _iter_state: &mut Self::IterState) -> Self::Item<'w> {
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

            unsafe fn prepare_iter_state(_layout: &ArchetypeLayout, _types: &TypesRegistryCache) -> Self::IterState {
                unsafe {
                    (
                        $($P::prepare_iter_state(_layout, _types)),+
                    )
                }
            }
            unsafe fn next<'w>(item: &ArchetypeUnsafeFfiIteratorItem, iter_state: &mut Self::IterState) -> Self::Item<'w> {
                #[allow(non_snake_case)]
                let (
                    $($P),+
                ) = iter_state;

                unsafe {
                    (
                        $($P::next(item, $P)),+
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
    iter: ArchetypeUnsafeFfiIterator<'a>,
    iter_state: A::IterState,
    _phantom: PhantomData<&'a mut A>,
}

impl<'a, A: ArchetypeIteratorItem> ArchetypeIterator<'a, A> {
    pub unsafe fn new(archetype: ArchetypeUnsafeRef<'a>) -> Self {
        let iter_state = unsafe { A::prepare_iter_state(archetype.archetype.layout(), &archetype.types) };
        
        Self {
            iter: archetype.archetype.iter(),
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

        Some(unsafe { A::next(&item, &mut self.iter_state) })
    }
}

//

fn archetype_unsafe_contains_component_type(archetype: &ArchetypeUnsafeFfi, types: &TypesRegistryCache, type_id: &TypeId) -> bool {
    let Some(type_id) = types.get_with_type_id(type_id) else {
        return false;
    };

    archetype.contains_component_type(type_id)
}

fn archetype_unsafe_iter<'a, A: ArchetypeIteratorItem>(archetype: ArchetypeUnsafeRef<'a>) -> ArchetypeIterator<'a, A> {
    unsafe { ArchetypeIterator::new(archetype) }
}

fn archetype_unsafe_get_component_ptr<C: 'static>(archetype: &ArchetypeUnsafeFfi, types: &TypesRegistryCache, entity_index: u64) -> Option<*mut C> {
    archetype.get_component_ptr(entity_index, types.get_or_register::<C>()).map(|p| p as *mut C)
}

/// Returns the last entity from src archetype before the movement.
fn archetype_unsafe_add_component<C: 'static>(
    src: &mut ArchetypeUnsafeFfi,
    dst: &mut ArchetypeUnsafeFfi,
    types: &TypesRegistryCache,
    src_entity_index: u64,
    component: C,
) -> Result<Entity, C> {
    unsafe {
        let mut component = MaybeUninit::new(component);

        if let Some(entity) = ArchetypeUnsafeFfi::add_component(
            src,
            dst,
            src_entity_index,
            component.as_mut_ptr() as *mut u8,
            types.get_or_register::<C>(),
        ) {
            Ok(entity)
        } else {
            Err(component.assume_init())
        }
    }
}

/// Returns the last entity from src archetype before the movement.
fn archetype_unsafe_remove_component<C: 'static>(
    src: &mut ArchetypeUnsafeFfi,
    dst: &mut ArchetypeUnsafeFfi,
    types: &TypesRegistryCache,
    src_entity_index: u64,
) -> Option<(Entity, C)> {
    unsafe {
        let mut component = MaybeUninit::<C>::uninit();

        if let Some(entity) = ArchetypeUnsafeFfi::remove_component(
            src,
            dst,
            src_entity_index,
            component.as_mut_ptr() as *mut u8,
            types.get_or_register::<C>(),
        ) {
            Some((entity, component.assume_init()))
        } else {
            None
        }
    }
}

//

pub struct ArchetypeUnsafe {
    archetype: ArchetypeUnsafeFfi,
    types: TypesRegistryCache,
}

impl ArchetypeUnsafe {
    pub fn new(components_set: UniqueComponentsSet, types: TypesRegistryCache) -> Self {
        Self {
            archetype: ArchetypeUnsafeFfi::new(unsafe { types.registry().clone() }, components_set),
            types,
        }
    }

    pub fn contains_component_type(&self, type_id: &TypeId) -> bool {
        archetype_unsafe_contains_component_type(&self.archetype, &self.types, type_id)
    }

    pub fn iter<'a, A: ArchetypeIteratorItem>(&'a self) -> ArchetypeIterator<'a, A> {
        // todo: threading guards
        unsafe {
            ArchetypeIterator::new(self.as_ref())
        }
    }

    pub fn entities_count(&self) -> u64 {
        self.archetype.entities_count()
    }

    pub fn get_entity(&self, entity_index: u64) -> Option<Entity> {
        self.archetype.get_entity(entity_index)
    }

    pub fn get_component_ptr<C: 'static>(&self, entity_index: u64) -> Option<*mut C> {
        archetype_unsafe_get_component_ptr(&self.archetype, &self.types, entity_index)
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
        archetype_unsafe_add_component::<C>(&mut src.archetype, &mut dst.archetype, &src.types, src_entity_index, component)
    }

    /// Returns the last entity from src archetype before the movement.
    pub fn remove_component<C: 'static>(src: &mut Self, dst: &mut Self, src_entity_index: u64) -> Option<(Entity, C)> {
        archetype_unsafe_remove_component::<C>(&mut src.archetype, &mut dst.archetype, &src.types, src_entity_index)
    }

    pub unsafe fn ffi_archetype(&self) -> &ArchetypeUnsafeFfi {
        &self.archetype
    }

    pub fn as_mut<'a>(&'a mut self) -> ArchetypeUnsafeMut<'a> {
        ArchetypeUnsafeMut::new(&mut self.archetype, &self.types)
    }

    pub fn as_ref<'a>(&'a self) -> ArchetypeUnsafeRef<'a> {
        ArchetypeUnsafeRef::new(&self.archetype, &self.types)
    }
}

//

pub struct ArchetypeUnsafeMut<'a> {
    archetype: &'a mut ArchetypeUnsafeFfi,
    types: &'a TypesRegistryCache,
}

impl<'a> ArchetypeUnsafeMut<'a> {
    pub fn new(archetype: &'a mut ArchetypeUnsafeFfi, types: &'a TypesRegistryCache) -> Self {
        Self {
            archetype,
            types,
        }
    }

    pub fn contains_component_type(&self, type_id: &TypeId) -> bool {
        archetype_unsafe_contains_component_type(self.archetype, &self.types, type_id)
    }

    pub fn iter<'r, A: ArchetypeIteratorItem>(&'r self) -> ArchetypeIterator<'r, A>
        where 'a: 'r
    {
        // todo: threading guards
        unsafe {
            ArchetypeIterator::new(self.as_ref())
        }
    }

    pub fn entities_count(&self) -> u64 {
        self.archetype.entities_count()
    }

    pub fn get_entity(&self, entity_index: u64) -> Option<Entity> {
        self.archetype.get_entity(entity_index)
    }

    pub fn get_component_ptr<C: 'static>(&self, entity_index: u64) -> Option<*mut C> {
        archetype_unsafe_get_component_ptr(self.archetype, &self.types, entity_index)
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
        archetype_unsafe_add_component::<C>(src.archetype, dst.archetype, &src.types, src_entity_index, component)
    }

    /// Returns the last entity from src archetype before the movement.
    pub fn remove_component<C: 'static>(src: &mut Self, dst: &mut Self, src_entity_index: u64) -> Option<(Entity, C)> {
        archetype_unsafe_remove_component::<C>(src.archetype, dst.archetype, &src.types, src_entity_index)
    }

    pub unsafe fn ffi_archetype(&self) -> &ArchetypeUnsafeFfi {
        &self.archetype
    }

    pub fn as_mut<'r>(&'r mut self) -> ArchetypeUnsafeMut<'r>
        where 'a: 'r
    {
        ArchetypeUnsafeMut::new(self.archetype, &self.types)
    }

    pub fn as_ref<'r>(&'r self) -> ArchetypeUnsafeRef<'r>
        where 'a: 'r
    {
        ArchetypeUnsafeRef::new(self.archetype, &self.types)
    }
}

//

pub struct ArchetypeUnsafeRef<'a> {
    archetype: &'a ArchetypeUnsafeFfi,
    types: &'a TypesRegistryCache,
}

impl<'a> ArchetypeUnsafeRef<'a> {
    pub fn new(archetype: &'a ArchetypeUnsafeFfi, types: &'a TypesRegistryCache) -> Self {
        Self {
            archetype,
            types,
        }
    }

    pub fn contains_component_type(&self, type_id: &TypeId) -> bool {
        archetype_unsafe_contains_component_type(self.archetype, &self.types, type_id)
    }

    pub fn iter<'r, A: ArchetypeIteratorItem>(&'r self) -> ArchetypeIterator<'r, A>
        where 'a: 'r
    {
        archetype_unsafe_iter(self.as_ref())
    }

    // todo: merge with iter
    pub fn into_iter<A: ArchetypeIteratorItem>(self) -> ArchetypeIterator<'a, A> {
        archetype_unsafe_iter(self)
    }

    pub fn entities_count(&self) -> u64 {
        self.archetype.entities_count()
    }

    pub fn get_entity(&self, entity_index: u64) -> Option<Entity> {
        self.archetype.get_entity(entity_index)
    }

    pub fn get_component_ptr<C: 'static>(&self, entity_index: u64) -> Option<*mut C> {
        archetype_unsafe_get_component_ptr(self.archetype, &self.types, entity_index)
    }

    pub unsafe fn ffi_archetype(&self) -> &ArchetypeUnsafeFfi {
        &self.archetype
    }

    pub fn as_ref<'r>(&'r self) -> ArchetypeUnsafeRef<'r>
        where 'a: 'r
    {
        ArchetypeUnsafeRef::new(self.archetype, &self.types)
    }
}

