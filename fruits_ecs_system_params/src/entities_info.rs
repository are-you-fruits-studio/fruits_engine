use std::{any::TypeId, marker::PhantomData};

use fruits_ecs_component::{EntitiesGuard, Entity};
use fruits_ecs_data::WorldDataSystemSharedRef;
use fruits_ecs_data_usage::{DataUsage, DataUsageEntry};
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct EntitiesInfo<'e> {
    guard: EntitiesGuard<'e>,
    // todo
    world_data: WorldDataSystemSharedRef<'e>,
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

    fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        let world_data: WorldDataSystemSharedRef<'a> = input.world_data.try_into_system_shared().ok_or("World locked.")?;

        Ok(EntitiesInfo {
            // Safety. Not self-referential. WorldDataSystemSharedRef stores a pointer to the entities data.
            guard: unsafe { std::mem::transmute::<_, EntitiesGuard<'a>>(world_data.entities_components.entities().ok_or("Entities locked.")?) },
            world_data: world_data,
        })
    }
}
