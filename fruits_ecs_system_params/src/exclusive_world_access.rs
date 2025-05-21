use std::ops::{Deref, DerefMut};

use fruits_ecs_data::{WorldDataStorage, WorldDataSystemUniqueRef};
use fruits_ecs_data_usage::*;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct ExclusiveWorldAccess<'w> {
    world_ref: WorldDataSystemUniqueRef<'w>,
}

impl<'w> Deref for ExclusiveWorldAccess<'w> {
    type Target = WorldDataStorage;

    fn deref(&self) -> &Self::Target {
        &self.world_ref
    }
}
impl<'w> DerefMut for ExclusiveWorldAccess<'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world_ref
    }
}

unsafe impl<'b> SystemParam for ExclusiveWorldAccess<'b> {
    type Item<'e> = ExclusiveWorldAccess<'e>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add_all_mut();
    }

    fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(ExclusiveWorldAccess {
            world_ref: input.world_data.try_into_system_unique().ok_or("World locked.")?,
        })
    }
}