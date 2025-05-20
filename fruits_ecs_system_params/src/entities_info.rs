use std::{any::TypeId, marker::PhantomData};

use fruits_ecs_component::{EntitiesGuard, Entity};
use fruits_ecs_data_usage::{DataUsage, DataUsageEntry};
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct EntitiesInfo<'e> {
    guard: EntitiesGuard,
    _phantom: PhantomData<&'e Entity>
}

impl<'e> EntitiesInfo<'e> {
    pub fn exists(&self, entity: Entity) -> bool {
        self.guard.contains_entity(entity)
    }
    
    pub fn entities_count(&self) -> usize {
        self.guard.entities_count()
    }
}

unsafe impl<'e> SystemParam for EntitiesInfo<'e> {
    type Item<'b> = EntitiesInfo<'b>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry {
            data_type: TypeId::of::<Entity>(),
            is_mutable: false,
        });
    }

    fn new<'a>(input: &'a SystemInput<'a>) -> Option<Self::Item<'a>> {
        Some(EntitiesInfo {
            guard: input.world_data.entities_components().entities()?,
            _phantom: Default::default(),
        })
    }
}
