use std::cell::UnsafeCell;

use fruits_ecs_component::{EntitiesComponentsHolderSafe, EntitiesComponentsHolderUnsafe};

use crate::{ResourceHolderSafe, ResourceHolderUnsafe};

struct WorldDataStorage {
    resources: ResourceHolderUnsafe,
    entities_components: EntitiesComponentsHolderUnsafe,
}

#[repr(transparent)]
pub struct WorldDataUnsafe {
    data: UnsafeCell<WorldDataStorage>,
}
impl WorldDataUnsafe {
    pub fn new() -> Self {
        Self {
            data: UnsafeCell::new(WorldDataStorage {
                resources: ResourceHolderUnsafe::new(),
                entities_components: EntitiesComponentsHolderUnsafe::new(),
            }),
        }
    }

    /// Safety. Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn resources(&self) -> &ResourceHolderUnsafe {
        // Safety. Lifetimes safety is managed by caller
        &unsafe { &*self.data.get() }.resources
    }

    /// Safety. Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn resources_mut(&self) -> &mut ResourceHolderUnsafe {
        // Safety. Lifetimes safety is managed by caller
        &mut unsafe { &mut *self.data.get() }.resources
    }

    /// Safety. Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn entities_components(&self) -> &EntitiesComponentsHolderUnsafe {
        // Safety. Lifetimes safety is managed by caller
        &unsafe { &*self.data.get() }.entities_components
    }

    /// Safety. Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn entities_components_mut(&self) -> &mut EntitiesComponentsHolderUnsafe {
        // Safety. Lifetimes safety is managed by caller
        &mut unsafe { &mut *self.data.get() }.entities_components
    }

    /// Safety. Ensure it never breaks shared-mut ref rules (multiple shared refs or single mut ref).
    pub unsafe fn as_safe(&self) -> &mut WorldData {
        // Safety. Transmute is safe, because of repr(transparent). Lifetimes safety is managed by caller
        unsafe { std::mem::transmute::<&mut WorldDataStorage, &mut WorldData>(&mut *self.data.get()) }
    }
}

#[repr(transparent)]
pub struct WorldData {
    data: WorldDataUnsafe,
}
// Safety. Data struct isn't changed - internals support threaded access and send.
unsafe impl Send for WorldDataUnsafe { }
unsafe impl Sync for WorldDataUnsafe { }

impl WorldData {
    pub fn new() -> Self {
        Self {
            data: WorldDataUnsafe::new(),
        }
    }

    pub fn resources(&self) -> &ResourceHolderSafe {
        // Safety. Managed with lifetimes.
        unsafe { self.data.resources().as_safe() }
    }

    pub fn resources_mut(&mut self) -> &mut ResourceHolderSafe {
        // Safety. Managed with lifetimes.
        unsafe { self.data.resources_mut().as_safe_mut() }
    }

    pub fn entities_components(&self) -> &EntitiesComponentsHolderSafe {
        // Safety. Managed with lifetimes.
        unsafe { self.data.entities_components().as_safe() }
    }

    pub fn entities_components_mut(&mut self) -> &mut EntitiesComponentsHolderSafe {
        // Safety. Managed with lifetimes.
        unsafe { self.data.entities_components_mut().as_safe_mut() }
    }

    pub unsafe fn as_unsafe(&mut self) -> &mut WorldDataUnsafe {
        &mut self.data
    }
}