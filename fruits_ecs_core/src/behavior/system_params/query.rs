use crate::*;

pub struct WorldQuery<'e, A: ArchetypeIteratorItem, F: QueryFilter = ()> {
    q: EntitiesHolderQuery<'e, A, F>,
}

impl<'e, A: ArchetypeIteratorItem, F: QueryFilter> WorldQuery<'e, A, F> {
    pub fn iter<'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>> + 'r
        where 'e: 'r
    {
        self.q.iter()
    }
    pub fn iter_mut<'r>(&'r mut self) -> impl Iterator<Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'r>> + 'r
        where 'e: 'r
    {
        self.q.iter_mut()
    }

    pub fn len(&self) -> u64 {
        self.q.len()
    }

    pub fn is_empty(&self) -> bool {
        self.q.is_empty()
    }

    pub fn get<'r>(&'r self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'r>>
        where 'e: 'r
    {
        self.q.get(entity)
    }

    pub fn get_mut<'r>(&'r mut self, entity: Entity) -> Option<<A::Item<'static> as ArchetypeIteratorItem>::Item<'r>>
        where 'e: 'r
    {
        self.q.get_mut(entity)
    }
}

// todo: unsafe to sealed trait
unsafe impl<'e, A: ArchetypeIteratorItem, F: QueryFilter> SystemParam for WorldQuery<'e, A, F> {
    type Item<'b> = WorldQuery<'b, A::Item<'b>, F>;

    fn fill_data_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
        if !usage.can_add_anything() {
            // todo
            panic!("fruits: Invalid system DataUsage.");
        }
        
        A::fill_usage(usage, types);
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(WorldQuery {
            // Safety. Managed by caller.
            q: unsafe { input.world_data.entities().into_query::<A::Item<'_>, F>() },
        })
    }
}
