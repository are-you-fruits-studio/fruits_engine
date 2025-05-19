use std::{any::TypeId, marker::PhantomData, ops::Deref};

use fruits_ecs_data::ResourceReadGuard;
use fruits_ecs_data_usage::*;

use fruits_ecs_data::Resource;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct Res<'w, R: Resource> {
    resource: ResourceReadGuard<R>,
    _phantom: PhantomData<&'w R>,
}

impl<'w, R: Resource> Deref for Res<'w, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

unsafe impl<'a, R: Resource> SystemParam for Res<'a, R> {
    type Item<'d> = Res<'d, R>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_readonly(TypeId::of::<R>()));
    }

    fn new<'d>(input: SystemInput<'d>) -> Option<Self::Item<'d>> {
        Some(Res {
            resource: input.world_data.try_read().ok()?.resources().get::<R>()?,
            _phantom: Default::default(),
        })
    }
}
