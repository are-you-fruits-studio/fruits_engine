use std::ops::{Deref, DerefMut};

use fruits_ecs_data::WorldData;
use fruits_ecs_data_usage::*;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct ExclusiveWorldAccess<'w> {
    world: &'w mut WorldData,
}

impl<'w> Deref for ExclusiveWorldAccess<'w> {
    type Target = WorldData;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}
impl<'w> DerefMut for ExclusiveWorldAccess<'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}

unsafe impl<'b> SystemParam for ExclusiveWorldAccess<'b> {
    type Item<'e> = ExclusiveWorldAccess<'e>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add_all_mut();
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(ExclusiveWorldAccess {
            world: input.world_data.as_safe(),
        })
    }
}