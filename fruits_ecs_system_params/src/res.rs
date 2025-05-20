use std::{any::TypeId, marker::PhantomData, ops::Deref};

use fruits_ecs_data::ResourceReadGuard;
use fruits_ecs_data_usage::*;

use fruits_ecs_data::Resource;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct Res<'e, R: Resource> {
    resource: ResourceReadGuard<R>,
    _phantom: PhantomData<&'e R>,
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

    fn new<'a>(input: &'a SystemInput<'a>) -> Option<Self::Item<'a>> {
        Some(Res {
            resource: input.world_data.resources().get()?,
            _phantom: Default::default(),
        })
    }
}
