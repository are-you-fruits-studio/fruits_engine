use std::{marker::PhantomData, ops::{Deref, DerefMut}};

use crate::*;

pub struct Local<'e, S: SystemResource> {
    data: SystemResourceGuard<S>,
    _phantom: PhantomData<&'e mut S>,
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

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_static::<S>(DataUsageDetails{ is_mutable: true, is_required: true }));
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(Local {
            data: input.system_data.get_or_create::<S>(),
            _phantom: Default::default(),
        })
    }
}