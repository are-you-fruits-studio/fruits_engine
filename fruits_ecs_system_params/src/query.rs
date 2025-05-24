use std::any::TypeId;

use fruits_ecs_component::{ArchetypeIteratorItem, Component, Entity, SafeQuery};
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
    q: SafeQuery<'e, A>,
}

impl<'e, A: ArchetypeIteratorItem> WorldQuery<'e, A> {
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

    pub fn len(&self) -> usize {
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
unsafe impl<'e, A: ArchetypeIteratorItem> SystemParam for WorldQuery<'e, A> {
    type Item<'b> = WorldQuery<'b, A::Item<'b>>;

    fn fill_data_usage(usage: &mut DataUsage) {
        if let DataUsage::PerType(per_type) = usage {
            A::fill_usage(per_type);
        }
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(WorldQuery {
            // Safety. Managed by caller.
            q: unsafe { input.world_data.entities_components().query::<A::Item<'_>>() },
        })
    }
}
