use std::{any::TypeId, ops::Deref};

use fruits_ecs_data::{ResourceLftReadGuard, WorldDataSystemSharedRef};
use fruits_ecs_data_usage::*;

use fruits_ecs_data::Resource;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct Res<'e, R: Resource> {
    resource: ResourceLftReadGuard<'e, R>,
    // todo
    guard: WorldDataSystemSharedRef<'e>,
}

impl<'e, R: Resource> Deref for Res<'e, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

unsafe impl<'e, R: Resource> SystemParam for Res<'e, R> {
    type Item<'b> = Res<'b, R>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_readonly(TypeId::of::<R>()));
    }

    fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        let world_ref = input.world_data.try_into_system_shared().ok_or("World locked.")?;

        Ok(Res {
            // Safety. Not self-referential. Resources are stored on Heap (via HashMap).
            resource: unsafe { std::mem::transmute::<_, ResourceLftReadGuard<'a, R>>(world_ref.resources.get::<R>().ok_or("Resource missing.")?) },
            guard: world_ref,
        })
    }
}
