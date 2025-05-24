use std::any::TypeId;

use fruits_ecs_component::{EntitiesComponentsHolderUnsafe, Entity};
use fruits_ecs_data_usage::{DataUsage, DataUsageEntry};
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct EntitiesInfo<'e> {
    ec: &'e EntitiesComponentsHolderUnsafe,
}

impl<'e> EntitiesInfo<'e> {
    pub fn exists(&self, entity: Entity) -> bool {
        self.ec.contains_entity(entity)
    }
    
    pub fn entities_count(&self) -> usize {
        self.ec.entities_count()
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
        Ok(EntitiesInfo {
            // Safety. Managed by caller
            ec: unsafe { input.world_data.entities_components() },
        })
    }
}
