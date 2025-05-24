use std::any::TypeId;

use fruits_ecs_component::{EntitiesGuard, Entity};
use fruits_ecs_data_usage::{DataUsage, DataUsageEntry};
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct EntitiesInfo<'e> {
    guard: EntitiesGuard<'e>,
    // todo
    ec: WorldDataSystemSharedRef<'e>,
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

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        let ec = input.world_data.entities_components();

        Ok(EntitiesInfo {
            // Safety. Not self-referential. WorldDataSystemSharedRef stores a pointer to the entities data.
            guard: unsafe { std::mem::transmute::<_, EntitiesGuard<'a>>(ec.entities().ok_or("Entities locked.")?) },
            ec,
        })
    }
}
