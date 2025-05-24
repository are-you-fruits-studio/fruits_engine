use std::{any::TypeId, marker::PhantomData};

use fruits_ecs_data_usage::PerTypeDataUsage;

use super::{
    archetype::{Archetype, ArchetypeIteratorItem}, component::{Component, WorldArchetypes}, entity::{Entity, EntityLocation, WorldEntities}, unique_components_set::UniqueComponentsSet,
};

// todo: update unsafe modifier for all functions inside this file.

pub struct EntitiesComponentsHolderUnsafe {
    archetypes: WorldArchetypes,
    entity_datas: WorldEntities,
}
impl EntitiesComponentsHolderUnsafe {
    pub fn new() -> Self {
        Self {
            archetypes: WorldArchetypes::new(),
            entity_datas: WorldEntities::new(),
        }
    }
    
    pub unsafe fn query<A: ArchetypeIteratorItem>(&self) -> SafeQuery<A> {
        SafeQuery::new(self)
    }

    pub fn entities_count(&self) -> usize {
        self.entity_datas.len()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entity_datas.contains(entity)
    }

    pub fn create_entity(&mut self) -> Entity {
        let archetype_id = self.archetypes.id_by_components_or_create(UniqueComponentsSet::new()).0;

        let archetype = self.archetypes.by_id_mut(archetype_id).unwrap();

        let entity_archetype_index = archetype.entities_count();
        
        let entity = self.entity_datas.insert(EntityLocation {
            archetype_id,
            entity_archetype_index,
        });
        
        archetype.create_entity(entity);

        entity
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        let Some(entity_location) = self.entity_datas.remove(entity) else {
            return false;
        };

        let archetype = self.archetypes.by_id_mut(entity_location.archetype_id).unwrap();

        let last_entity = archetype.destroy_entity(entity_location.entity_archetype_index).unwrap();

        if last_entity != entity {
            *self.entity_datas.get_mut(last_entity).unwrap() = entity_location;
        }

        return true;
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        let Some(entity_location) = self.entity_datas.get(entity) else {
            return Err(component);
        };

        let src_archetype_id = entity_location.archetype_id;
        
        let mut dst_components_set = {
            let src_archetype = self.archetypes.by_id_ref(src_archetype_id).unwrap();

            src_archetype.components_set().clone()
        };

        if !dst_components_set.insert::<C>() {
            return Err(component);
        }

        let dst_archetype_id = self.archetypes.id_by_components_or_create(dst_components_set).0;

        // len 0 1
        // len 1 1
        // todo: what?
        let (src_archetype, dst_archetype) = self.archetypes.by_2_ids_mut((src_archetype_id, dst_archetype_id)).unwrap();

        let entity_with_added_component_new_location = EntityLocation {
            archetype_id: dst_archetype_id,
            entity_archetype_index: dst_archetype.entities_count(),
        };

        // Safety. Access is unique and is only used in the method scope.
        let last_entity = unsafe {
            Archetype::add_component(src_archetype, dst_archetype, entity_location.entity_archetype_index, component).ok().unwrap()
        };

        if last_entity != entity {
            *self.entity_datas.get_mut(last_entity).unwrap() = *entity_location;
        }

        *self.entity_datas.get_mut(entity).unwrap() = entity_with_added_component_new_location;

        return Ok(());
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        let entity_location = self.entity_datas.get(entity)?;

        let src_archetype_id = entity_location.archetype_id;

        let mut dst_components_set = {
            let src_archetype = self.archetypes.by_id_ref(src_archetype_id).unwrap();

            src_archetype.components_set().clone()
        };

        if !dst_components_set.remove::<C>() {
            return None;
        }

        let dst_archetype_id = self.archetypes.id_by_components_or_create(dst_components_set).0;

        let (src_archetype, dst_archetype) = self.archetypes.by_2_ids_mut((src_archetype_id, dst_archetype_id)).unwrap();

        let entity_with_removed_component_new_location = EntityLocation {
            archetype_id: dst_archetype_id,
            entity_archetype_index: dst_archetype.entities_count(),
        };

        // Safety. Access is unique and is only used in the method scope.
        let (last_entity, component) = unsafe {
            Archetype::remove_component(src_archetype, dst_archetype, entity_location.entity_archetype_index).unwrap()
        };

        if last_entity != entity {
            *self.entity_datas.get_mut(last_entity).unwrap() = *entity_location;
        }

        *self.entity_datas.get_mut(entity).unwrap() = entity_with_removed_component_new_location;

        return Some(component);
    }

    /// Safety. No lifetime management. Make sure ptr lives less than a guard.
    pub unsafe fn get_component_ptr<C: Component>(&self, entity: Entity) -> Option<*mut C> {
        let entity_location = self.entity_datas.get(entity)?;

        let archetype = self.archetypes.by_id_ref(entity_location.archetype_id).unwrap();

        // Safety. Transfering safety management to caller.
        unsafe {
            archetype.get_component_ptr::<C>(entity_location.entity_archetype_index)
        }
    }

    pub unsafe fn as_safe(&self) -> &EntitiesComponentsHolderSafe {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesComponentsHolderUnsafe, &EntitiesComponentsHolderSafe>(self) }
    }

    pub unsafe fn as_safe_mut(&mut self) -> &mut EntitiesComponentsHolderSafe {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EntitiesComponentsHolderUnsafe, &mut EntitiesComponentsHolderSafe>(self) }
    }
}

#[repr(transparent)]
pub struct EntitiesComponentsHolderSafe {
    data: EntitiesComponentsHolderUnsafe,
}
impl EntitiesComponentsHolderSafe {
    pub fn new() -> Self {
        Self {
            data: EntitiesComponentsHolderUnsafe::new(),
        }
    }

    pub fn query<A: ArchetypeIteratorItem>(&self) -> SafeQuery<A::Item<'_>> {
        SafeQuery::new(&self.data)
    }

    pub fn query_mut<A: ArchetypeIteratorItem>(&mut self) -> SafeQuery<A::ReadOnlyItem<'_>> {
        SafeQuery::new(&mut self.data)
    }

    pub fn entities_count(&self) -> usize {
        self.data.entities_count()
    }

    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.data.contains_entity(entity)
    }

    pub fn create_entity(&mut self) -> Entity {
        self.data.create_entity()
    }

    pub fn destroy_entity(&mut self, entity: Entity) -> bool {
        self.data.destroy_entity(entity)
    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<(), C> {
        self.data.add_component(entity, component)
    }

    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        self.data.remove_component(entity)
    }

    pub fn get_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.get_component_ptr(entity).map(|p| &*p) }
    }

    pub fn get_component_mut<C: Component>(&self, entity: Entity) -> Option<&mut C> {
        // Safety. Managed with lifetimes.
        unsafe { self.data.get_component_ptr(entity).map(|p| &mut *p) }
    }

    pub unsafe fn as_unsafe(&self) -> &EntitiesComponentsHolderUnsafe {
        &self.data
    }

    pub unsafe fn as_unsafe_mut(&mut self) -> &mut EntitiesComponentsHolderUnsafe {
        &mut self.data
    }
}

pub struct SafeQuery<'d, A: ArchetypeIteratorItem> {
    query: UnsafeQuery<'d, A>,
}
impl<'d, A: ArchetypeIteratorItem> SafeQuery<'d, A> {
    fn new(data: &'d EntitiesComponentsHolderUnsafe) -> Self {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        let query = unsafe {
            UnsafeQuery::<A>::new(data)
        };

        Self {
            query,
        }
    }

    pub fn iter<'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>> + 'r
        where 'd: 'r,
    {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.iter() }
    }
    
    pub fn iter_mut<'r>(&'r mut self) -> impl Iterator<Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'r>> + 'r
        where 'd: 'r,
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

    pub fn get<'r>(&'r self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>>
        where 'd: 'r,
    {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.get(entity) }
    }

    pub fn get_mut<'r>(&'r mut self, entity: Entity) -> Option<<A::Item<'static> as ArchetypeIteratorItem>::Item<'r>>
        where 'd: 'r,
    {
        // Safety. Access is unique because of borrow-rules of EntitiesComponentsUniqueGuard references.
        unsafe { self.query.get_mut(entity) }
    }
}

struct UnsafeQuery<'d, A: ArchetypeIteratorItem> {
    data: &'d EntitiesComponentsHolderUnsafe,
    archetype_indices: Box<[usize]>,
    _phantom: PhantomData<fn(A::Item<'static>) -> A::Item<'static>>,
}
impl<'d, A: ArchetypeIteratorItem> UnsafeQuery<'d, A> {
    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn new(data: &'d EntitiesComponentsHolderUnsafe) -> Self {
        let mut usage = PerTypeDataUsage::new();

        A::fill_usage(&mut usage);
        
        let mut components = usage.into_values();

        components.remove(&TypeId::of::<Entity>());

        if components.len() == 0 {
            return Self {
                data: data,
                archetype_indices: (0..data.archetypes.all().len()).collect::<Box<_>>(),
                _phantom: Default::default(),
            };
        }

        let archetypes_with_rarest_component = components
            .keys()
            .map(|c| data.archetypes.ids_by_component(c))
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
                let Some(archetypes_with_component) = data.archetypes.ids_by_component(c) else {
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
    unsafe fn iter<'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>> + 'r
        where 'd: 'r
    {
        self.archetype_indices.iter()
            .copied()
            .map(|i| self.data.archetypes.by_id_ref(i).unwrap())
            .flat_map(move |a| a.iter::<A::ReadOnlyItem<'static>>())
    }
    
    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn iter_mut<'r>(&'r mut self) -> impl Iterator<Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'r>> + 'r
        where 'd: 'r
    {
        self.archetype_indices.iter()
            .copied()
            .map(|i| self.data.archetypes.by_id_ref(i).unwrap())
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
    unsafe fn get<'r>(&'r self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>>
        where 'd: 'r,
    {
        if TypeId::of::<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'static>>() == TypeId::of::<Entity>() {
            let item = unsafe {
                std::mem::transmute_copy::<_, <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>>(&entity)
            };

            return Some(item);
        }

        let location = self.data.entity_datas.get(entity)?;

        let archetype = self.data.archetypes.by_id_ref(location.archetype_id)?;

        Some(<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::from_archetype(
            location.entity_archetype_index,
            unsafe { archetype.unsafe_archetype() },
            archetype.layout(),
        ))
    }

    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn get_mut<'r>(&'r mut self, entity: Entity) -> Option<<A::Item<'static> as ArchetypeIteratorItem>::Item<'r>>
        where 'd: 'r,
    {
        if TypeId::of::<<A::Item<'static> as ArchetypeIteratorItem>::Item<'static>>() == TypeId::of::<Entity>() {
            let item = unsafe {
                std::mem::transmute_copy::<_, <A::Item<'static> as ArchetypeIteratorItem>::Item<'r>>(&entity)
            };

            return Some(item);
        }

        let location = self.data.entity_datas.get(entity)?;

        let archetype = self.data.archetypes.by_id_ref(location.archetype_id)?;

        Some(<A::Item<'static> as ArchetypeIteratorItem>::from_archetype(
            location.entity_archetype_index,
            unsafe { archetype.unsafe_archetype() },
            archetype.layout(),
        ))
    }

    /// Safety. No lifetime or locking is applied - needs to be managed by caller.
    unsafe fn archetypes_iter<'a>(&'a self) -> impl Iterator<Item = &'a Archetype> + 'a
    {
        self.archetype_indices.iter()
            .map(|i| self.data.archetypes.by_id_ref(*i).unwrap())
    }
}