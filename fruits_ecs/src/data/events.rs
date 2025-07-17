use std::{any::{Any, TypeId}, cell::UnsafeCell, collections::HashMap};

pub trait Event : 'static + Send + Sync { }

trait AbstractEvents {
    unsafe fn clear(&self);
    fn cell_as_any(&self) -> &dyn Any;
}

struct VirtualEvents<E: Event> {
    pub events: UnsafeCell<Vec<E>>,
}
impl<E: Event> VirtualEvents<E> {
    pub fn new() -> Self {
        Self {
            events: UnsafeCell::new(Vec::new()),
        }
    }
}
impl<E: Event> AbstractEvents for VirtualEvents<E> {
    unsafe fn clear(&self) {
        unsafe { &mut *self.events.get() }.clear();
    }
    
    fn cell_as_any(&self) -> &dyn Any {
        &self.events
    }
}

pub struct EventsHolderUnsafe {
    events: UnsafeCell<HashMap<TypeId, Box<dyn AbstractEvents>>>,
}
impl EventsHolderUnsafe {
    pub fn new() -> Self {
        Self {
            events: UnsafeCell::new(HashMap::new()),
        }
    }

    /// Safety. Lifetimes and access sync should be managed by caller. Deallocation is managed by EventHolderUnsafe.
    pub unsafe fn get<E: Event>(&self) -> Option<*mut Vec<E>> {
        unsafe { 
            let events = &mut *(self.events.get());

            Some(events.get(&TypeId::of::<E>())?.cell_as_any().downcast_ref::<UnsafeCell<Vec<E>>>().unwrap().get())
        }
    }

    /// Safety. Lifetimes and access sync should be managed by caller. Deallocation is managed by EventHolderUnsafe.
    pub unsafe fn get_or_create<E: Event>(&self) -> *mut Vec<E> {
        unsafe {
            let events = &mut *(self.events.get());

            events
                .entry(TypeId::of::<E>())
                .or_insert_with(|| Box::new(VirtualEvents::<E>::new()))
                .cell_as_any()
                .downcast_ref::<UnsafeCell<Vec<E>>>().unwrap().get()
        }
    }

    /// Safety. Lifetimes and access sync should be managed by caller. Deallocation is managed by EventHolderUnsafe.
    pub unsafe fn clear(&self) {
        unsafe {
            let events = &mut *(self.events.get());

            for values in events.values() {
                values.clear();
            }
        }
    }

    pub fn as_safe(&self) -> &EventsHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&EventsHolderUnsafe, &EventsHolder>(self) }
    }

    pub fn as_safe_mut(&mut self) -> &mut EventsHolder {
        // Safety. Safe, because of repr(transparent).
        unsafe { std::mem::transmute::<&mut EventsHolderUnsafe, &mut EventsHolder>(self) }
    }
}
// Safety. Is safe itself. Ptr usage is managed by caller
unsafe impl Send for EventsHolderUnsafe { }
unsafe impl Sync for EventsHolderUnsafe { }

#[repr(transparent)]
pub struct EventsHolder {
    events: EventsHolderUnsafe,
}
impl EventsHolder {
    pub fn new() -> Self {
        Self {
            events: EventsHolderUnsafe::new(),
        }
    }

    pub fn get<E: Event>(&self) -> &[E] {
        // Safety. Lifetimes manage the access and syncing.
        unsafe {
            match self.events.get() {
                Some(e) => &*e,
                None => &[],
            }
        }
    }
    pub fn get_mut<E: Event>(&mut self) -> &mut Vec<E> {
        // Safety. Lifetimes manage the access and syncing.
        unsafe {
            &mut *self.events.get_or_create()
        }
    }
    pub fn clear(&mut self) {
        // Safety. Lifetimes manage the access and syncing.
        unsafe {
            self.events.clear();
        }
    }
}