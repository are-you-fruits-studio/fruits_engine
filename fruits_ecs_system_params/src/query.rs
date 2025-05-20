use std::{any::TypeId, marker::PhantomData};

use fruits_ecs_component::{ArchetypeIteratorItem, Component, Entity, EntitiesComponentsQueryGuard};
use fruits_ecs_data_usage::*;

use fruits_ecs_system::{SystemInput, SystemParam};

pub unsafe trait WorldQueryIterParam {
    fn component_type() -> TypeId;
    fn is_mutable() -> bool;
}

unsafe impl<P: Component> WorldQueryIterParam for &P {
    fn component_type() -> TypeId { TypeId::of::<P>() }
    fn is_mutable() -> bool { false }
}

unsafe impl<P: Component> WorldQueryIterParam for &mut P {
    fn component_type() -> TypeId { TypeId::of::<P>() }
    fn is_mutable() -> bool { true }
}

pub struct WorldQuery<'e, A: ArchetypeIteratorItem> {
    query: EntitiesComponentsQueryGuard<A>,
    _phantom: PhantomData<&'e A>,
}

// todo: unsafe to sealed trait
unsafe impl<'e, A: ArchetypeIteratorItem> SystemParam for WorldQuery<'e, A> {
    type Item<'b> = WorldQuery<'b, A::Item<'b>>;

    fn fill_data_usage(usage: &mut DataUsage) {
        if let DataUsage::PerType(per_type) = usage {
            A::fill_usage(per_type);
        }
    }

    fn new<'a>(input: &'a SystemInput<'a>) -> Option<Self::Item<'a>> {
        let query = input.world_data.entities_components().query::<A::Item<'a>>()?;

        Some(WorldQuery {
            query,
            _phantom: Default::default(),
        })
    }
}

impl<'e, A: ArchetypeIteratorItem> WorldQuery<'e, A> {
    pub fn iter<'r>(&'r self) -> impl Iterator<Item = <A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
        where 'e: 'r
    {
        self.query.iter()
    }
    pub fn iter_mut<'r>(&'r mut self) -> impl Iterator<Item = <A::Item<'static> as ArchetypeIteratorItem>::Item<'e>> + 'r
        where 'e: 'r
    {
        self.query.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.query.len()
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    pub fn get<'r>(&'r self, entity: Entity) -> Option<<A::ReadOnlyItem<'static> as ArchetypeIteratorItem>::Item<'e>>
        where 'e: 'r
    {
        self.query.get(entity)
    }

    pub fn get_mut<'r>(&'r mut self, entity: Entity) -> Option<<A::Item<'static> as ArchetypeIteratorItem>::Item<'e>>
        where 'e: 'r
    {
        self.query.get_mut(entity)
    }
}
