use std::{any::TypeId, ops::{Deref, DerefMut}};

use fruits_ecs_data::{ResourceLftWriteGuard, WorldDataSystemSharedRef};
use fruits_ecs_data_usage::*;

use fruits_ecs_data::Resource;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct ResMut<'e, R: Resource> {
    resource: ResourceLftWriteGuard<'e, R>,
    // todo
    guard: WorldDataSystemSharedRef<'e>,
}

impl<'e, R: Resource> Deref for ResMut<'e, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

impl<'e, R: Resource> DerefMut for ResMut<'e, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}

unsafe impl<'e, R: Resource> SystemParam for ResMut<'e, R> {
    type Item<'b> = ResMut<'b, R>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_mutable(TypeId::of::<R>()));
    }

    fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        let world_ref = input.world_data.try_into_system_shared().ok_or("World locked.")?;

        Ok(ResMut {
            // Safety. Not self-referential. Resources are stored on Heap (via HashMap).
            resource: unsafe { std::mem::transmute::<_, ResourceLftWriteGuard<'a, R>>(world_ref.resources.get_mut::<R>().ok_or("Resource missing.")?) },
            guard: world_ref,
        })
    }
}