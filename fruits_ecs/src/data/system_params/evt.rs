use std::ops::Deref;

use crate::*;

pub struct Evt<'e, E: Event> {
    evt: &'e [E],
}

impl<'e, E: Event> Deref for Evt<'e, E> {
    type Target = [E];

    fn deref(&self) -> &Self::Target {
        &self.evt
    }
}

unsafe impl<'e, E: Event> SystemParam for Evt<'e, E> {
    type Item<'b> = Evt<'b, E>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add(DataUsageEntry::new_static::<E>(DataUsageDetails{ is_mutable: false, is_required: true }));
    }

    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str> {
        Ok(Evt {
            // Safety. Managed by caller.
            evt: unsafe { input.world_data.events().get::<E>().map(|p| &*p as &[E]).unwrap_or(&[]) },
        })
    }
}
