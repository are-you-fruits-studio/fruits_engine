use std::{any::TypeId, marker::PhantomData};

use crate::*;

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
    
    pub unsafe fn query<A: ArchetypeIteratorItem, F: QueryFilter>(&self) -> EntitiesComponentsQuery<A, F> {
        // Safety. Managed by caller.
        unsafe {
            EntitiesComponentsQuery::new(self)
        }
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

    pub unsafe fn as_safe(&self) -> &EntitiesComponentsHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EntitiesComponentsHolderUnsafe, &EntitiesComponentsHolder>(self) }
    }

    pub unsafe fn as_safe_mut(&mut self) -> &mut EntitiesComponentsHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EntitiesComponentsHolderUnsafe, &mut EntitiesComponentsHolder>(self) }
    }
}

#[repr(transparent)]
pub struct EntitiesComponentsHolder {
    data: EntitiesComponentsHolderUnsafe,
}
impl EntitiesComponentsHolder {
    pub fn new() -> Self {
        Self {
            data: EntitiesComponentsHolderUnsafe::new(),
        }
    }

    pub fn query<A: ArchetypeIteratorItem>(&self) -> EntitiesComponentsQuery<A::Item<'_>> {
        // Safety. Managed with lifetimes.
        unsafe {
            EntitiesComponentsQuery::new(&self.data)
        }
    }

    pub fn query_filtered<A: ArchetypeIteratorItem, F: QueryFilter>(&self) -> EntitiesComponentsQuery<A::Item<'_>, F> {
        // Safety. Managed with lifetimes.
        unsafe {
            EntitiesComponentsQuery::new(&self.data)
        }
    }

    pub fn query_mut<A: ArchetypeIteratorItem>(&mut self) -> EntitiesComponentsQuery<A::ReadOnlyItem<'_>> {
        // Safety. Managed with lifetimes.
        unsafe {
            EntitiesComponentsQuery::new(&mut self.data)
        }
    }

    pub fn query_filtered_mut<A: ArchetypeIteratorItem, F: QueryFilter>(&mut self) -> EntitiesComponentsQuery<A::ReadOnlyItem<'_>, F> {
        // Safety. Managed with lifetimes.
        unsafe {
            EntitiesComponentsQuery::new(&mut self.data)
        }
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
}

pub struct EntitiesComponentsQuery<'d, A: ArchetypeIteratorItem, F: QueryFilter = ()> {
    data: &'d EntitiesComponentsHolderUnsafe,
    archetype_indices: Vec<usize>,
    _phantom: (PhantomData<fn(A::Item<'static>) -> A::Item<'static>>, PhantomData<fn(F) -> F>),
}
impl<'d, A: ArchetypeIteratorItem, F: QueryFilter> EntitiesComponentsQuery<'d, A, F> {
    /// Safety. No access sync - needs to be managed by caller.
    unsafe fn new(data: &'d EntitiesComponentsHolderUnsafe) -> Self {
        let mut usage = PerTypeDataUsage::new();

        A::fill_usage(&mut usage);
        
        let mut components = usage.into_values();

        components.remove(&TypeId::of::<Entity>());

        if components.len() == 0 {
            // Query is with entities only (should iterate all entities).
            return Self {
                data: data,
                archetype_indices: (0..data.archetypes.all().len())
                    .filter(|ai| F::matches(data.archetypes.by_id_ref(*ai).unwrap().layout()))
                    .collect::<Vec<_>>(),
                _phantom: Default::default(),
            };
        }

        let required_components = components.iter().filter(|(_, d)| d.is_required).map(|(t, _)| t);

        let mut archetypes_with_rarest_component = None;

        for component in required_components.clone() {
            let Some(archetypes_with_component) = data.archetypes.ids_by_component(component) else {
                // Query is with some required component that no archetype has (should iterate none).
                return Self {
                    data: data,
                    archetype_indices: Vec::new(),
                    _phantom: Default::default(),
                };
            };

            let Some(archetypes_with_rarest_component) = &mut archetypes_with_rarest_component else {
                archetypes_with_rarest_component = Some((archetypes_with_component.len(), archetypes_with_component));
                continue;
            };

            if archetypes_with_component.len() < archetypes_with_rarest_component.0 {
                *archetypes_with_rarest_component = (archetypes_with_component.len(), archetypes_with_component);
            }
        }

        let Some((_, archetypes_with_rarest_component)) = archetypes_with_rarest_component else {
            // Query has optional components only (should iterate all entities).
            return Self {
                data: data,
                archetype_indices: (0..data.archetypes.all().len())
                    .filter(|ai| F::matches(data.archetypes.by_id_ref(*ai).unwrap().layout()))
                    .collect::<Vec<_>>(),
                _phantom: Default::default(),
            };
        };

        let mut suitable_archetypes = Vec::new();

        for archetype_id in archetypes_with_rarest_component.iter() {
            let archetype = data.archetypes.by_id_ref(*archetype_id).unwrap();
            
            let contains_all_components = required_components.clone().all(|c| {
                archetype.contains_component_type(c)
            });

            // Archetypes that are missing any required component are skipped.
            if contains_all_components && F::matches(archetype.layout()) {
                suitable_archetypes.push(*archetype_id);
            }
        }

        Self {
            data: data,
            archetype_indices: suitable_archetypes,
            _phantom: Default::default(),
        }
    }

    // todo: don't remove until the new version fully checked.
    pub fn iter_old<'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>> + 'r
        where 'd: 'r
    {
        self.archetypes_iter()
            .flat_map(move |a| a.iter::<A::ReadOnlyItem<'static>>())
    }

    pub fn iter<'r>(&'r self) -> QueryIter<'r, A>
        where 'd: 'r
    {
        QueryIter::new(self.archetypes_iter())
    }
    
    pub fn iter_mut_old<'r>(&'r mut self) -> impl Iterator<Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'r>> + 'r
        where 'd: 'r
    {
        self.archetypes_iter()
            .flat_map(move |a| a.iter::<A::Item<'static>>())
    }
    
    pub fn iter_mut<'r>(&'r mut self) -> QueryIterMut<'r, A>
        where 'd: 'r
    {
        QueryIterMut::new(self.archetypes_iter())
    }

    pub fn len(&self) -> usize {
        self.archetypes_iter()
            .map(|a| a.entities_count())
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        !self.archetypes_iter().any(|a| a.entities_count() > 0)
    }

    pub fn get<'r>(&'r self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>>
        where 'd: 'r,
    {
        let location = self.data.entity_datas.get(entity)?;

        let archetype = self.data.archetypes.by_id_ref(location.archetype_id)?;

        <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::from_archetype(
            location.entity_archetype_index,
            unsafe { archetype.unsafe_archetype() },
            archetype.layout(),
        )
    }

    pub fn get_mut<'r>(&'r mut self, entity: Entity) -> Option<<A::Item<'static> as ArchetypeIteratorItem>::Item<'r>>
        where 'd: 'r,
    {
        let location = self.data.entity_datas.get(entity)?;

        let archetype = self.data.archetypes.by_id_ref(location.archetype_id)?;

        <A::Item<'static> as ArchetypeIteratorItem>::from_archetype(
            location.entity_archetype_index,
            unsafe { archetype.unsafe_archetype() },
            archetype.layout(),
        )
    }

    fn archetypes_iter(&self) -> ArchetypesIter {
        ArchetypesIter::new(&self.data.archetypes, self.archetype_indices.iter())
    }
}

struct ArchetypesIter<'a> {
    archetypes: &'a WorldArchetypes,
    iter: std::slice::Iter<'a, usize>,
}

impl<'a> ArchetypesIter<'a> {
    pub fn new(archetypes: &'a WorldArchetypes, iter: std::slice::Iter<'a, usize>) -> Self {
        Self {
            archetypes,
            iter,
        }
    }
}

impl<'a> Iterator for ArchetypesIter<'a> {
    type Item = &'a Archetype;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.archetypes.by_id_ref(*self.iter.next()?).unwrap())
    }
}

pub struct QueryIter<'a, A: ArchetypeIteratorItem> {
    iter: ArchetypesIter<'a>,
    iter2: Option<ArchetypeIterator<'a, A::ReadOnlyItem<'static>>>,
}

impl<'a, A: ArchetypeIteratorItem> QueryIter<'a, A> {
    fn new(iter: ArchetypesIter<'a>) -> Self {
        Self {
            iter,
            iter2: None,
        }
    }
}
impl<'a, A: ArchetypeIteratorItem> Iterator for QueryIter<'a, A> {
    type Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // if let item @ Some(_) = self.iter2.as_mut().map(Iterator::next).flatten() {}
            if let Some(iter2) = &mut self.iter2 && let item @ Some(_) = iter2.next() {
                return item;
            } else if let Some(iter_item) = self.iter.next() {
                self.iter2 = Some(iter_item.iter::<A::ReadOnlyItem<'static>>());
            } else {
                return None;
            }
        }
    }
}

pub struct QueryIterMut<'a, A: ArchetypeIteratorItem> {
    iter: ArchetypesIter<'a>,
    iter2: Option<ArchetypeIterator<'a, A::Item<'static>>>,
}

impl<'a, A: ArchetypeIteratorItem> QueryIterMut<'a, A> {
    fn new(iter: ArchetypesIter<'a>) -> Self {
        Self {
            iter,
            iter2: None,
        }
    }
}
impl<'a, A: ArchetypeIteratorItem> Iterator for QueryIterMut<'a, A> {
    type Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // if let item @ Some(_) = self.iter2.as_mut().map(Iterator::next).flatten() {}
            if let Some(iter2) = &mut self.iter2 && let item @ Some(_) = iter2.next() {
                return item;
            } else if let Some(iter_item) = self.iter.next() {
                self.iter2 = Some(iter_item.iter::<A::Item<'static>>());
            } else {
                return None;
            }
        }
    }
}

// todo

// impl<'d, A: ArchetypeIteratorItem, F: QueryFilter> IntoIterator for &EntitiesComponentsQuery<'d, A, F> {
//     type Item = <QueryIter<'d, A> as Iterator>::Item;

//     type IntoIter = QueryIter<'d, A>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

pub unsafe trait QueryFilter: 'static {
    fn matches(layout: &ArchetypeLayout) -> bool;
}

pub struct WithoutFilter<T: Component>
{
    _phantom: PhantomData<fn(T) -> T>,
}

unsafe impl<C: 'static + Component> QueryFilter for WithoutFilter<C> {
    fn matches(layout: &ArchetypeLayout) -> bool {
        !layout.components().contains_key(&TypeId::of::<C>())
    }
}

pub struct WithFilter<T: Component>
{
    _phantom: PhantomData<fn(T) -> T>,
}

unsafe impl<C: 'static + Component> QueryFilter for WithFilter<C> {
    fn matches(layout: &ArchetypeLayout) -> bool {
        layout.components().contains_key(&TypeId::of::<C>())
    }
}

macro_rules! query_filter_tuple {
    ($($q: ident),*) => {
        unsafe impl<
        $($q: QueryFilter),*
        > QueryFilter for (
        $($q,)*
        ) {
            fn matches(_layout: &ArchetypeLayout) -> bool {
                true
                $(&& $q::matches(_layout))*
            }
        }
    };
}

query_filter_tuple!();
query_filter_tuple!(Q1);
query_filter_tuple!(Q1, Q2);
query_filter_tuple!(Q1, Q2, Q3);
query_filter_tuple!(Q1, Q2, Q3, Q4);
query_filter_tuple!(Q1, Q2, Q3, Q4, Q5);
query_filter_tuple!(Q1, Q2, Q3, Q4, Q5, Q6);
query_filter_tuple!(Q1, Q2, Q3, Q4, Q5, Q6, Q7);

// todo: to sealed
pub unsafe trait OrFilterArg: 'static {
    fn or(layout: &ArchetypeLayout) -> bool;
}

pub struct OrFilter<A: OrFilterArg>
{
    _phantom: PhantomData<fn(A) -> A>,
}

unsafe impl<A: OrFilterArg> QueryFilter for OrFilter<A> {
    fn matches(layout: &ArchetypeLayout) -> bool {
        A::or(layout)
    }
}

macro_rules! query_filter_or {
    ($($q: ident),*) => {
        unsafe impl<
        $($q: QueryFilter),*
        > OrFilterArg for (
        $($q,)*
        ) {
            fn or(_layout: &ArchetypeLayout) -> bool {
                false
                $(|| $q::matches(_layout))*
            }
        }
    };
}

query_filter_or!();
query_filter_or!(Q1);
query_filter_or!(Q1, Q2);
query_filter_or!(Q1, Q2, Q3);
query_filter_or!(Q1, Q2, Q3, Q4);
query_filter_or!(Q1, Q2, Q3, Q4, Q5);
query_filter_or!(Q1, Q2, Q3, Q4, Q5, Q6);
query_filter_or!(Q1, Q2, Q3, Q4, Q5, Q6, Q7);