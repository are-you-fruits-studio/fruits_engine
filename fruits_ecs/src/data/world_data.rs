use std::cell::UnsafeCell;

use crate::*;

#[derive(Default)]
struct WorldDataStorage {
    resources: ResourcesHolderUnsafe,
    entities_components: EntitiesComponentsHolderUnsafe,
    events: EventsHolderUnsafe,
}
impl WorldDataStorage {
    pub fn new() -> Self {
        Self {
            resources: ResourcesHolderUnsafe::new(),
            entities_components: EntitiesComponentsHolderUnsafe::new(),
            events: EventsHolderUnsafe::new(),
        }
    }
}

#[repr(transparent)]
pub struct WorldDataUnsafe {
    data: UnsafeCell<WorldDataStorage>,
}
impl WorldDataUnsafe {
    pub fn new() -> Self {
        Self {
            data: UnsafeCell::new(WorldDataStorage::new()),
        }
    }

    /// # Safety
    /// 
    /// Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn resources(&self) -> *mut ResourcesHolderUnsafe {
        // Safety. Managed by caller.
        unsafe { &raw mut (*self.data.get()).resources }
    }

    /// # Safety
    /// 
    /// Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn entities_components(&self) -> *mut EntitiesComponentsHolderUnsafe {
        // Safety. Managed by caller.
        unsafe { &raw mut (*self.data.get()).entities_components }
    }

    /// # Safety
    /// 
    /// Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn events(&self) -> *mut EventsHolderUnsafe {
        // Safety. Managed by caller.
        unsafe { &raw mut (*self.data.get()).events }
    }

    /// # Safety
    /// 
    /// Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn from_safe_mut(world: &mut WorldData) -> &mut Self {
        &mut world.data
    }

    /// # Safety
    /// 
    /// Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn as_safe_mut(&self) -> &mut WorldData {
        // Safety. Transmute is safe, because of repr(transparent). Lifetimes safety is managed by caller
        unsafe { &mut *(self.data.get() as *mut WorldData) }
    }
}

impl Default for WorldDataUnsafe {
    fn default() -> Self {
        Self::new()
    }
}

// Safety. Data struct isn't changed - internals support threaded access and send.
unsafe impl Send for WorldDataUnsafe { }
unsafe impl Sync for WorldDataUnsafe { }

#[derive(Default)]
#[repr(transparent)]
pub struct WorldData {
    data: WorldDataUnsafe,
}

impl WorldData {
    pub fn new() -> Self {
        Self {
            data: WorldDataUnsafe::new(),
        }
    }

    pub fn resources(&self) -> &ResourcesHolder {
        // Safety. Managed with lifetimes.
        unsafe { (&*self.data.resources()).as_safe() }
    }

    pub fn resources_mut(&mut self) -> &mut ResourcesHolder {
        // Safety. Managed with lifetimes.
        unsafe { (&mut *self.data.resources()).as_safe_mut() }
    }

    pub fn entities_components(&self) -> &EntitiesComponentsHolder {
        // Safety. Managed with lifetimes.
        unsafe { (&*self.data.entities_components()).as_safe() }
    }

    pub fn entities_components_mut(&mut self) -> &mut EntitiesComponentsHolder {
        // Safety. Managed with lifetimes.
        unsafe { (&mut *self.data.entities_components()).as_safe_mut() }
    }

    pub fn events(&self) -> &EventsHolder {
        // Safety. Managed with lifetimes.
        unsafe { (&*self.data.events()).as_safe() }
    }

    pub fn events_mut(&mut self) -> &mut EventsHolder {
        // Safety. Managed with lifetimes.
        unsafe { (&mut *self.data.events()).as_safe_mut() }
    }

    pub fn as_tuple_mut(&mut self) -> (&mut ResourcesHolder, &mut EntitiesComponentsHolder, &mut EventsHolder) {
        unsafe { (
            (&mut *self.data.resources()).as_safe_mut(),
            (&mut *self.data.entities_components()).as_safe_mut(),
            (&mut *self.data.events()).as_safe_mut(),
        ) }
    }
}