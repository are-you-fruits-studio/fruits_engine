use std::{any::TypeId, marker::PhantomData, ops::{Deref, DerefMut}};

use fruits_ecs_data::ResourceWriteGuard;
use fruits_ecs_data_usage::*;

use fruits_ecs_data::Resource;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct ResMut<'d, R: Resource> {
    resource: ResourceWriteGuard<R>,
    _phantom: PhantomData<&'d mut R>,
}

impl<'d, R: Resource> Deref for ResMut<'d, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

impl<'d, R: Resource> DerefMut for ResMut<'d, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}

unsafe impl<'a, R: Resource> SystemParam for ResMut<'a, R> {
    type Item<'d> = ResMut<'d, R>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_mutable(TypeId::of::<R>()));
    }

    fn new<'d>(input: SystemInput<'d>) -> Option<Self::Item<'d>> {
        Some(ResMut {
            resource: input.world_data.try_read().ok()?.resources().get_mut::<R>()?,
            _phantom: Default::default(),
        })
    }
}