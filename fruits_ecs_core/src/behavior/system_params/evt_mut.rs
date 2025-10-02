use std::ops::{Deref, DerefMut};

use fruits_ffi::FfiVec;

use crate::*;

pub struct EvtMut<'e, E: 'static> {
    evt: &'e mut FfiVec<E>,
}

impl<'e, E: 'static> Deref for EvtMut<'e, E> {
    type Target = FfiVec<E>;

    fn deref(&self) -> &Self::Target {
        self.evt
    }
}
impl<'e, E: 'static> DerefMut for EvtMut<'e, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.evt
    }
}

unsafe impl<'e, E: 'static> SystemParam for EvtMut<'e, E> {
    type Item<'b> = EvtMut<'b, E>;

    fn fill_data_usage(usage: &mut DataUsageBuilder, types: &TypesRegistryCache) {
        usage.add(DataUsageEntry {
            type_id: types.get_or_register::<E>(),
            details: DataUsageDetails { is_mutable: true, is_required: true }
        });
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(EvtMut {
            // Safety. Managed by caller.
            evt: unsafe { &mut *input.world_data.events().get_or_create::<E>() },
        })
    }
}
