use std::ops::{Deref, DerefMut};

use crate::*;

pub struct EvtMut<'e, E: Event> {
    evt: &'e mut Vec<E>,
}

impl<'e, E: Event> Deref for EvtMut<'e, E> {
    type Target = Vec<E>;

    fn deref(&self) -> &Self::Target {
        self.evt
    }
}
impl<'e, E: Event> DerefMut for EvtMut<'e, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.evt
    }
}

unsafe impl<'e, E: Event> SystemParam for EvtMut<'e, E> {
    type Item<'b> = EvtMut<'b, E>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_static::<E>(DataUsageDetails{ is_mutable: true, is_required: true }));
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(EvtMut {
            // Safety. Managed by caller.
            evt: unsafe { &mut *input.world_data.events().get_or_create::<E>() },
        })
    }
}
