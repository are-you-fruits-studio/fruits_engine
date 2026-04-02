use std::ops::{Deref, DerefMut};

use crate::*;

pub struct Local<'e, S: SystemResource> {
    data: &'e mut S,
}

impl<'e, S: SystemResource> Deref for Local<'e, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'e, S: SystemResource> DerefMut for Local<'e, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

unsafe impl<'e, S: SystemResource> SystemParam for Local<'e, S> {
    type Item<'b> = Local<'b, S>;

    fn fill_data_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
        usage.add_system(types.get_or_register::<S>());
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        unsafe {
            Ok(Local {
                data: &mut *input.system_data.get_or_insert::<S>(),
            })
        }
    }
}
