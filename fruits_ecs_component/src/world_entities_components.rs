use std::{any::TypeId, cell::UnsafeCell, marker::PhantomData};

use fruits_ecs_data_usage::PerTypeDataUsage;

use crate::{DataRwLockGlobalGuard, DataRwLockReadGuard};

use super::{
    archetype::{Archetype, ArchetypeIteratorItem}, component::{Component, WorldArchetypes}, data_rw_lock::{DataRwLockRef, DataRwLockGuard},
    entity::{Entity, EntityLocation, WorldEntities}, unique_components_set::UniqueComponentsSet,
};

struct EntitiesComponentsData {
    archetypes: WorldArchetypes,
    entity_datas: WorldEntities,
}

pub struct EntitiesComponentsHolder {
    data: UnsafeCell<EntitiesComponentsData>,
    locks: DataRwLockRef,
}
impl EntitiesComponentsHolder {
    pub fn new() -> Self {
        Self {
            data: UnsafeCell::new(EntitiesComponentsData {
                archetypes: WorldArchetypes::new(),
                entity_datas: WorldEntities::new(),
            }),
            locks: DataRwLockRef::new(),
        }
    }

    pub fn unique(&mut self) -> EntitiesComponentsUniqueGuard {
        EntitiesComponentsUniqueGuard::new(self)
    }

    pub fn entities(&self) -> Option<EntitiesGuard> {
        EntitiesGuard::new(self)
    }

    pub fn query<A: ArchetypeIteratorItem>(&self) -> Option<EntitiesComponentsQueryGuard<A>> {
        EntitiesComponentsQueryGuard::new(self)
    }
}

// Safety. synchronization is handled by guards.
unsafe impl Send for EntitiesComponentsHolder { }
unsafe impl Sync for EntitiesComponentsHolder { }

pub struct EntitiesComponentsUniqueGuard<'d> {
    data: &'d mut EntitiesComponentsData,
    _guard: DataRwLockGlobalGuard,
}
impl<'d> EntitiesComponentsUniqueGuard<'d> {
    pub fn new(ec: &'d mut EntitiesComponentsHolder) -> Self {
        // todo: do we need to lock? Aren't lifetimes enough?
        let guard = ec.locks.global().unwrap();

        Self {
            data: ec.data.get_mut(),
            _guard: guard,
        }
    }

    pub fn query<A: ArchetypeIteratorItem>(&'d self) -> EntitiesComponentsUniqueQueryGuard<'d, A> {
        EntitiesComponentsUniqueQueryGuard::new(self)
    }

    pub fn query_mut<A: ArchetypeIteratorItem>(&'d mut self) -> EntitiesComponentsUniqueQueryMutGuard<'d, A> {
        EntitiesComponentsUniqueQueryMutGuard::new(self)
    }

    pub fn entities_count(&self) -> usize {
        self.data.entity_datas.len()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.data.entity_datas.contains(entity)
    }

    pub fn create_entity(&mut self) -> Entity {
        let data = &mut self.data;

        let archetype_id = data.archetypes.id_by_components_or_create(UniqueComponentsSet::new()).0;

        let archetype = data.archetypes.by_id_mut(archetype_id).unwrap();

        let entity_archetype_index = archetype.entities_count();
        
        let entity = data.entity_datas.insert(EntityLocation {
            archetype_id,
            entity_archetype_index,
        });
        
        archetype.create_entity(entity);

        entity
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        let data = &mut self.data;
        
        let Some(entity_location) = data.entity_datas.remove(entity) else {
            return false;
        };

        let archetype = data.archetypes.by_id_mut(entity_location.archetype_id).unwrap();

        let last_entity = archetype.destroy_entity(entity_location.entity_archetype_index).unwrap();

        if last_entity != entity {
            *data.entity_datas.get_mut(last_entity).unwrap() = entity_location;
        }

        return true;
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        let data = &mut self.data;

        let Some(entity_location) = data.entity_datas.get(entity) else {
            return Err(component);
        };

        let src_archetype_id = entity_location.archetype_id;
        
        let mut dst_components_set = {
            let src_archetype = data.archetypes.by_id_ref(src_archetype_id).unwrap();

            src_archetype.components_set().clone()
        };

        if !dst_components_set.insert::<C>() {
            return Err(component);
        }

        let dst_archetype_id = data.archetypes.id_by_components_or_create(dst_components_set).0;

        // len 0 1
        // len 1 1
        // todo: what?
        let (src_archetype, dst_archetype) = data.archetypes.by_2_ids_mut((src_archetype_id, dst_archetype_id)).unwrap();

        let entity_with_added_component_new_location = EntityLocation {
            archetype_id: dst_archetype_id,
            entity_archetype_index: dst_archetype.entities_count(),
        };

        // Safety. Access is unique and is only used in the method scope.
        let last_entity = unsafe {
            Archetype::add_component(src_archetype, dst_archetype, entity_location.entity_archetype_index, component).ok().unwrap()
        };

        if last_entity != entity {
            *data.entity_datas.get_mut(last_entity).unwrap() = *entity_location;
        }

        *data.entity_datas.get_mut(entity).unwrap() = entity_with_added_component_new_location;

        return Ok(());
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        let data = &mut self.data;

        let entity_location = data.entity_datas.get(entity)?;

        let src_archetype_id = entity_location.archetype_id;

        let mut dst_components_set = {
            let src_archetype = data.archetypes.by_id_ref(src_archetype_id).unwrap();

            src_archetype.components_set().clone()
        };

        if !dst_components_set.remove::<C>() {
            return None;
        }

        let dst_archetype_id = data.archetypes.id_by_components_or_create(dst_components_set).0;

        let (src_archetype, dst_archetype) = data.archetypes.by_2_ids_mut((src_archetype_id, dst_archetype_id)).unwrap();

        let entity_with_removed_component_new_location = EntityLocation {
            archetype_id: dst_archetype_id,
            entity_archetype_index: dst_archetype.entities_count(),
        };

        // Safety. Access is unique and is only used in the method scope.
        let (last_entity, component) = unsafe {
            Archetype::remove_component(src_archetype, dst_archetype, entity_location.entity_archetype_index).unwrap()
        };

        if last_entity != entity {
            *data.entity_datas.get_mut(last_entity).unwrap() = *entity_location;
        }

        *data.entity_datas.get_mut(entity).unwrap() = entity_with_removed_component_new_location;

        return Some(component);
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        // Safety. Safe, because ptr lives less than a guard.
        unsafe {
            self.get_component_ptr::<C>(entity).map(|p| &*p)
        }
    }

    pub fn get_component_mut<C: Component>(&mut self, entity: Entity) -> Option<&mut C> {
        // Safety. Safe, because ptr lives less than a guard.
        unsafe {
            self.get_component_ptr::<C>(entity).map(|p| &mut *p)
        }
    }

    /// Safety. No lifetime management. Make sure ptr lives less than a guard.
    unsafe fn get_component_ptr<C: Component>(&self, entity: Entity) -> Option<*mut C> {
        let data = &self.data;

        let entity_location = data.entity_datas.get(entity)?;

        let archetype = data.archetypes.by_id_ref(entity_location.archetype_id).unwrap();

        // Safety. Transfering safety management to caller.
        unsafe {
            archetype.get_component_ptr::<C>(entity_location.entity_archetype_index)
        }
    }
}

pub struct EntitiesGuard<'d> {
    data: *mut EntitiesComponentsData,
    _guard: DataRwLockReadGuard,
    _phantom: PhantomData<&'d EntitiesComponentsData>
}
impl<'d> EntitiesGuard<'d> {
    pub fn new(ec: &'d EntitiesComponentsHolder) -> Option<Self> {
        ec.locks.read(TypeId::of::<Entity>()).map(|g| Self {
            data: ec.data.get(),
            _guard: g,
            _phantom: Default::default(),
        })
    }

    pub fn entities_count(&self) -> usize {
        // Safety. Access is read-only. Access collisions are managed by the DataRwLock.
        let data = unsafe { &*self.data };

        data.entity_datas.len()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        // Safety. Access is read-only. Access collisions are managed by the DataRwLock.
        let data = unsafe { &*self.data };

        data.entity_datas.contains(entity)
    }
}

pub struct EntitiesComponentsQueryGuard<'d, A: ArchetypeIteratorItem> {
    query: UnsafeQueryGuard<A>,
    _guards: Box<[DataRwLockGuard]>,
    _phantom: PhantomData<&'d EntitiesComponentsData>
}

impl<'d, A: ArchetypeIteratorItem> EntitiesComponentsQueryGuard<'d, A> {
    fn new(ec: &'d EntitiesComponentsHolder) -> Option<Self> {
        let mut usage = PerTypeDataUsage::new();

        A::fill_usage(&mut usage);

        let guards = ec.locks.lock_by_type_usage(&usage)?;

        // Safety. Access is read-only. Access collisions are managed by the DataRwLock.
        let query = unsafe {
            UnsafeQueryGuard::<A>::new_unchecked(ec.data.get(), usage)
        };

        Some(Self {
            query,
            _guards: guards,
            _phantom: Default::default(),
        })
    }

    pub fn iter<'e: 'r, 'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
    {
        // Safety. Access is read-only. Access collisions are managed by the Self::new().
        unsafe { self.query.iter() }
    }
    
    pub fn iter_mut<'e: 'r, 'r>(&'r mut self) -> impl Iterator<Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
    {
        // Safety. Access is read-only. Access collisions are managed by the Self::new().
        unsafe { self.query.iter_mut() }
    }

    pub fn len(&self) -> usize {
        // Safety. Access is read-only. Access collisions are managed by the Self::new().
        unsafe { self.query.len() }
    }

    pub fn is_empty(&self) -> bool {
        // Safety. Access is read-only. Access collisions are managed by the Self::new().
        unsafe { self.query.is_empty() }
    }

    pub fn get<'e, 'r>(&'r self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>>
        where 'e: 'r
    {
        // Safety. Access is read-only. Access collisions are managed by the Self::new().
        unsafe { self.query.get(entity) }
    }

    pub fn get_mut<'e, 'r>(&'r mut self, entity: Entity) -> Option<<A::Item<'static> as ArchetypeIteratorItem>::Item<'e>>
        where 'e: 'r
    {
        // Safety. Access is read-only. Access collisions are managed by the Self::new().
        unsafe { self.query.get_mut(entity) }
    }
}

pub struct EntitiesComponentsUniqueQueryGuard<'g, A: ArchetypeIteratorItem> {
    query: UnsafeQueryGuard<A>,
    _guard: &'g EntitiesComponentsUniqueGuard<'g>,
}
impl<'g, A: ArchetypeIteratorItem> EntitiesComponentsUniqueQueryGuard<'g, A> {
    fn new(guard: &'g EntitiesComponentsUniqueGuard<'g>) -> Self {
        let mut usage = PerTypeDataUsage::new();

        A::fill_usage(&mut usage);
        
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        let query = unsafe {
            UnsafeQueryGuard::<A>::new_unchecked(guard.data as *const _, usage)
        };

        Self {
            query,
            _guard: guard,
        }
    }

    pub fn iter<'e: 'r, 'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
    {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.iter() }
    }

    pub fn len(&self) -> usize {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.len() }
    }

    pub fn is_empty(&self) -> bool {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.is_empty() }
    }

    pub fn get<'a>(&'a self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'a>> {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.get(entity) }
    }
}

pub struct EntitiesComponentsUniqueQueryMutGuard<'g, A: ArchetypeIteratorItem> {
    query: UnsafeQueryGuard<A>,
    _guard: &'g mut EntitiesComponentsUniqueGuard<'g>,
}
impl<'g, A: ArchetypeIteratorItem> EntitiesComponentsUniqueQueryMutGuard<'g, A> {
    fn new(guard: &'g mut EntitiesComponentsUniqueGuard<'g>) -> Self {
        let mut usage = PerTypeDataUsage::new();

        A::fill_usage(&mut usage);
        
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        let query = unsafe {
            UnsafeQueryGuard::<A>::new_unchecked(guard.data, usage)
        };

        Self {
            query,
            _guard: guard,
        }
    }

    pub fn iter<'e: 'r, 'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
    {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.iter() }
    }
    
    pub fn iter_mut<'e: 'r, 'r>(&'r mut self) -> impl Iterator<Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
    {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.iter_mut() }
    }

    pub fn len(&self) -> usize {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.len() }
    }

    pub fn is_empty(&self) -> bool {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.is_empty() }
    }

    pub fn get<'a>(&'a self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'a>> {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.get(entity) }
    }

    pub fn get_mut<'a>(&'a mut self, entity: Entity) -> Option<<A::Item<'static> as ArchetypeIteratorItem>::Item<'a>> {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.get_mut(entity) }
    }
}

struct UnsafeQueryGuard<A: ArchetypeIteratorItem> {
    data: *const EntitiesComponentsData,
    archetype_indices: Box<[usize]>,
    _phantom: PhantomData<fn(A::Item<'static>) -> A::Item<'static>>,
}
impl<A: ArchetypeIteratorItem> UnsafeQueryGuard<A> {
    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn new_unchecked(
        data: *const EntitiesComponentsData,
        usage: PerTypeDataUsage,
    ) -> Self {
        let mut components = usage.into_values();

        components.remove(&TypeId::of::<Entity>());

        // Safety. Managed by caller.
        let entities_components = unsafe { &*data };

        if components.len() == 0 {
            return Self {
                data: data,
                archetype_indices: (0..entities_components.archetypes.all().len()).collect::<Box<_>>(),
                _phantom: Default::default(),
            };
        }

        let archetypes_with_rarest_component = components
            .keys()
            .map(|c| entities_components.archetypes.ids_by_component(c))
            .flatten()
            .min_by_key(|a| a.len());

        let Some(archetypes_with_rarest_component) = archetypes_with_rarest_component else {
            return Self {
                data: data,
                archetype_indices: Box::new([]),
                _phantom: Default::default(),
            };
        };

        let mut suitable_archetypes = Vec::new();

        for archetype in archetypes_with_rarest_component.iter() {
            let contains_all_components = components.keys().all(|c| {
                let Some(archetypes_with_component) = entities_components.archetypes.ids_by_component(c) else {
                    return false;
                };

                archetypes_with_component.contains(archetype)
            });

            if contains_all_components {
                suitable_archetypes.push(*archetype);
            }
        }

        Self {
            data: data,
            archetype_indices: suitable_archetypes.into_boxed_slice(),
            _phantom: Default::default(),
        }
    }

    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn iter<'e: 'r, 'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
    {
        // Safety. Managed by caller.
        let data = unsafe { &*self.data };

        self.archetype_indices.iter()
            .copied()
            .map(|i| data.archetypes.by_id_ref(i).unwrap())
            .flat_map(move |a| a.iter::<A::ReadOnlyItem<'static>>())
    }
    
    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn iter_mut<'e: 'r, 'r>(&'r mut self) -> impl Iterator<Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
    {
        // Safety. Managed by caller.
        let data = unsafe { &*self.data };

        self.archetype_indices.iter()
            .copied()
            .map(|i| data.archetypes.by_id_ref(i).unwrap())
            .flat_map(move |a| a.iter::<A::Item<'static>>())
    }

    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn len(&self) -> usize {
        // Safety. Managed by caller.
        unsafe {
            self.archetypes_iter()
                .map(|a| a.entities_count())
                .sum()
        }
    }

    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn is_empty(&self) -> bool {
        // Safety. Managed by caller.
        unsafe {
            !self.archetypes_iter().any(|a| a.entities_count() > 0)
        }
    }

    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn get<'e, 'r>(&'r self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>>
        where 'e: 'r
    {
        // Safety. Managed by caller.
        let data = unsafe { &*self.data };

        if TypeId::of::<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'static>>() == TypeId::of::<Entity>() {
            let item = unsafe {
                std::mem::transmute_copy::<_, <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>>(&entity)
            };

            return Some(item);
        }

        let location = data.entity_datas.get(entity)?;

        let archetype = data.archetypes.by_id_ref(location.archetype_id)?;

        Some(<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::from_archetype(
            location.entity_archetype_index,
            unsafe { archetype.unsafe_archetype() },
            archetype.layout(),
        ))
    }

    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn get_mut<'e, 'r>(&'r mut self, entity: Entity) -> Option<<A::Item<'static> as ArchetypeIteratorItem>::Item<'e>>
        where 'e: 'r
    {
        // Safety. Managed by caller.
        let data = unsafe { &*self.data };

        if TypeId::of::<<A::Item<'static> as ArchetypeIteratorItem>::Item<'static>>() == TypeId::of::<Entity>() {
            let item = unsafe {
                std::mem::transmute_copy::<_, <A::Item<'static> as ArchetypeIteratorItem>::Item<'e>>(&entity)
            };

            return Some(item);
        }

        let location = data.entity_datas.get(entity)?;

        let archetype = data.archetypes.by_id_ref(location.archetype_id)?;

        Some(<A::Item<'static> as ArchetypeIteratorItem>::from_archetype(
            location.entity_archetype_index,
            unsafe { archetype.unsafe_archetype() },
            archetype.layout(),
        ))
    }

    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn archetypes_iter<'a>(&'a self) -> impl Iterator<Item = &'a Archetype> + 'a
    {
        // Safety. Managed by caller.
        let data = unsafe { &*self.data };

        self.archetype_indices.iter()
            .map(|i| data.archetypes.by_id_ref(*i).unwrap())
    }
}