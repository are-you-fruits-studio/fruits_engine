use std::{any::TypeId, marker::PhantomData, ops::{Deref, DerefMut}};

use fruits_ecs_data::ResourceWriteGuard;
use fruits_ecs_data_usage::*;

use fruits_ecs_data::Resource;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct ResMut<'e, R: Resource> {
    resource: ResourceWriteGuard<R>,
    _phantom: PhantomData<&'e mut R>,
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

    fn new<'a>(input: &'a SystemInput<'a>) -> Option<Self::Item<'a>> {
        Some(ResMut {
            resource: input.world_data.resources().get_mut()?,
            _phantom: Default::default(),
        })
    }
}