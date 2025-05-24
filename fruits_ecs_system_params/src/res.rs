use std::{any::TypeId, ops::Deref};

use fruits_ecs_data_usage::*;

use fruits_ecs_data::Resource;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct Res<'e, R: Resource> {
    res: &'e R,
}

impl<'e, R: Resource> Deref for Res<'e, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.res
    }
}

unsafe impl<'e, R: Resource> SystemParam for Res<'e, R> {
    type Item<'b> = Res<'b, R>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_readonly(TypeId::of::<R>()));
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(Res {
            // Safety. Managed by caller.
            res: unsafe { &*input.world_data.resources().get::<R>().ok_or("Resource missing.")? },
        })
    }
}
