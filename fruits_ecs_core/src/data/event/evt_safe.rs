use fruits_ffi::FfiVec;

use crate::*;

#[repr(transparent)]
pub struct EventsHolderRef<'e> {
    evt: EventsHolderUnsafeRef<'e>,
}
impl<'e> EventsHolderRef<'e> {
    pub fn get<'r, E: 'static>(&self) -> &[E]
        where 'e: 'r
    {
        // Safety. Lifetimes manage the access and syncing.
        unsafe {
            match self.evt.get() {
                Some(e) => &*e,
                None => &[],
            }
        }
    }

    pub fn get_mut<'r, E: 'static>(&mut self) -> &mut FfiVec<E>
        where 'e: 'r
    {
        // Safety. Lifetimes manage the access and syncing.
        unsafe {
            &mut *self.evt.get_or_create()
        }
    }

    pub fn clear(&mut self) {
        self.evt.clear();
    }

    pub fn as_ref<'r>(&'r mut self) -> EventsHolderRef<'r>
        where 'e: 'r
    {
        EventsHolderRef {
            evt: self.evt.as_ref(),
        }
    }
}