use std::{cell::UnsafeCell, mem::ManuallyDrop, ops::{Deref, DerefMut}, sync::Mutex};

use fruits_ecs_component::EntitiesComponentsHolder;

use crate::ResourcesLftHolder;

pub struct WorldDataStorage {
    pub resources: ResourcesLftHolder,
    pub entities_components: EntitiesComponentsHolder,
}

#[derive(Default)]
struct WorldDataLockState{
    unique: bool,
    system_reserved: usize,
    system_unique: bool,
    system_shared: usize,
}

pub struct WorldData {
    data: UnsafeCell<WorldDataStorage>,
    lock: Mutex<WorldDataLockState>,
}
// Safety. Data struct isn't changed - internals support threaded access and send.
unsafe impl Send for WorldData { }
unsafe impl Sync for WorldData { }
impl WorldData {
    pub fn new() -> Self {
        Self {
            data: UnsafeCell::new(WorldDataStorage {
                resources: ResourcesLftHolder::new(),
                entities_components: EntitiesComponentsHolder::new(),
            }),
            lock: Mutex::new(WorldDataLockState::default()),
        }
    }

    pub fn unique(&mut self) -> WorldDataUniqueRef {
        WorldDataUniqueRef::new(self)
    }
    pub fn try_unique(&self) -> Option<WorldDataUniqueRef> {
        WorldDataUniqueRef::try_new(self)
    }

    pub fn system_reserved(&mut self) -> WorldDataSystemReservedRef {
        WorldDataSystemReservedRef::new(self)
    }
    pub fn try_system_reserved(&self) -> Option<WorldDataSystemReservedRef> {
        WorldDataSystemReservedRef::try_new(self)
    }
}

pub struct WorldDataUniqueRef<'d> {
    data: *mut WorldDataStorage,
    lock: &'d Mutex<WorldDataLockState>,
}
// Safety. Data struct isn't changed - internals support threaded access and send.
unsafe impl<'d> Send for WorldDataUniqueRef<'d> { }
unsafe impl<'d> Sync for WorldDataUniqueRef<'d> { }
impl<'d> WorldDataUniqueRef<'d> {
    fn new(world_data: &'d mut WorldData) -> Self {
        {
            let mut lock_state = world_data.lock.lock().unwrap();

            lock_state.unique = true;
        }

        Self {
            data: world_data.data.get(),
            lock: &world_data.lock,
        }
    }

    fn try_new(world_data: &'d WorldData) -> Option<Self> {
        {
            let mut lock_state = world_data.lock.lock().unwrap();

            if lock_state.unique || lock_state.system_reserved > 0 || lock_state.system_unique || lock_state.system_shared > 0 {
                return None;
            }

            lock_state.unique = true;
        }

        Some(Self {
            data: world_data.data.get(),
            lock: &world_data.lock,
        })
    }
}
impl<'d> Drop for WorldDataUniqueRef<'d> {
    fn drop(&mut self) {
        let mut lock_state = self.lock.lock().unwrap();

        lock_state.unique = false;
    }
}
impl<'d> Deref for WorldDataUniqueRef<'d> {
    type Target = WorldDataStorage;

    fn deref(&self) -> &Self::Target {
        // Safety. Locks and self lifetimes guarantee synchronization.
        unsafe { &*self.data }
    }
}
impl<'d> DerefMut for WorldDataUniqueRef<'d> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety. Locks and self lifetimes guarantee synchronization.
        unsafe { &mut *self.data }
    }
}


pub struct WorldDataSystemReservedRef<'d> {
    data: *mut WorldDataStorage,
    lock: &'d Mutex<WorldDataLockState>,
}
// Safety. Data struct isn't changed - internals support threaded access and send.
unsafe impl<'d> Send for WorldDataSystemReservedRef<'d> { }
unsafe impl<'d> Sync for WorldDataSystemReservedRef<'d> { }
impl<'d> WorldDataSystemReservedRef<'d> {
    fn new(world_data: &'d mut WorldData) -> Self {
        {
            let mut lock_state = world_data.lock.lock().unwrap();

            lock_state.system_reserved += 1;
        }

        Self {
            data: world_data.data.get(),
            lock: &world_data.lock,
        }
    }

    fn try_new(world_data: &'d WorldData) -> Option<Self> {
        {
            let mut lock_state = world_data.lock.lock().unwrap();

            if lock_state.unique {
                return None;
            }

            lock_state.system_reserved += 1;
        }

        Some(Self {
            data: world_data.data.get(),
            lock: &world_data.lock,
        })
    }

    pub fn try_into_system_unique(&'d self) -> Option<WorldDataSystemUniqueRef<'d>> {
        WorldDataSystemUniqueRef::try_from_reserved(self)
    }
    pub fn try_into_system_shared(&'d self) -> Option<WorldDataSystemSharedRef<'d>> {
        WorldDataSystemSharedRef::try_from_reserved(self)
    }
}
impl<'d> Clone for WorldDataSystemReservedRef<'d> {
    fn clone(&self) -> Self {
        {
            let mut lock_state = self.lock.lock().unwrap();

            lock_state.system_reserved += 1;
        }

        Self {
            data: self.data,
            lock: self.lock
        }
    }
}
impl<'d> Drop for WorldDataSystemReservedRef<'d> {
    fn drop(&mut self) {
        let mut lock_state = self.lock.lock().unwrap();

        lock_state.system_reserved -= 1;
    }
}

pub struct WorldDataSystemSharedRef<'d> {
    data: *mut WorldDataStorage,
    lock: &'d Mutex<WorldDataLockState>,
}
// Safety. Data struct isn't changed - internals support threaded access and send.
unsafe impl<'d> Send for WorldDataSystemSharedRef<'d> { }
unsafe impl<'d> Sync for WorldDataSystemSharedRef<'d> { }
impl<'d> WorldDataSystemSharedRef<'d> {
    fn try_new(world_data: &'d WorldData) -> Option<Self> {
        {
            let mut lock_state = world_data.lock.lock().unwrap();

            if lock_state.unique || lock_state.system_unique {
                return None;
            }

            lock_state.system_shared += 1;
        }

        Some(Self {
            data: world_data.data.get(),
            lock: &world_data.lock,
        })
    }

    fn try_from_reserved(reserved_ref: &'d WorldDataSystemReservedRef) -> Option<Self> {
        {
            let mut lock_state = reserved_ref.lock.lock().unwrap();

            if lock_state.system_unique {
                return None;
            }

            lock_state.system_shared += 1;
        }

        Some(Self {
            data: reserved_ref.data,
            lock: &reserved_ref.lock,
        })
    }
}
impl<'d> Clone for WorldDataSystemSharedRef<'d> {
    fn clone(&self) -> Self {
        {
            let mut lock_state = self.lock.lock().unwrap();

            lock_state.system_shared += 1;
        }

        Self {
            data: self.data,
            lock: self.lock
        }
    }
}
impl<'d> Drop for WorldDataSystemSharedRef<'d> {
    fn drop(&mut self) {
        let mut lock_state = self.lock.lock().unwrap();

        lock_state.system_shared -= 1;
    }
}
impl<'d> Deref for WorldDataSystemSharedRef<'d> {
    type Target = WorldDataStorage;

    fn deref(&self) -> &Self::Target {
        // Safety. Locks and self lifetimes guarantee synchronization.
        unsafe { &*self.data }
    }
}

pub struct WorldDataSystemUniqueRef<'d> {
    data: *mut WorldDataStorage,
    lock: &'d Mutex<WorldDataLockState>,
}
// Safety. Data struct isn't changed - internals support threaded access and send.
unsafe impl<'d> Send for WorldDataSystemUniqueRef<'d> { }
unsafe impl<'d> Sync for WorldDataSystemUniqueRef<'d> { }
impl<'d> WorldDataSystemUniqueRef<'d> {
    fn new(world_data: &'d mut WorldData) -> Self {
        {
            let mut lock_state = world_data.lock.lock().unwrap();

            lock_state.system_unique = true;
        }

        Self {
            data: world_data.data.get(),
            lock: &world_data.lock,
        }
    }

    fn try_new(world_data: &'d WorldData) -> Option<Self> {
        {
            let mut lock_state = world_data.lock.lock().unwrap();

            if lock_state.unique || lock_state.system_unique || lock_state.system_shared > 0 {
                return None;
            }

            lock_state.system_unique = true;
        }

        Some(Self {
            data: world_data.data.get(),
            lock: &world_data.lock,
        })
    }

    fn try_from_reserved(reserved_ref: &'d WorldDataSystemReservedRef) -> Option<Self> {
        {
            let mut lock_state = reserved_ref.lock.lock().unwrap();

            if lock_state.system_unique || lock_state.system_shared > 0 {
                return None;
            }

            lock_state.system_unique = true;
        }

        Some(Self {
            data: reserved_ref.data,
            lock: &reserved_ref.lock,
        })
    }

    fn try_from_system_shared(reserved_ref: WorldDataSystemSharedRef<'d>) -> Option<Self> {
        {
            let mut lock_state = reserved_ref.lock.lock().unwrap();

            if lock_state.system_shared > 1 {
                return None;
            }

            lock_state.system_shared = 0;
            lock_state.system_unique = true;
        }

        let reserved_ref = ManuallyDrop::new(reserved_ref);

        Some(Self {
            data: reserved_ref.data,
            lock: reserved_ref.lock,
        })
    }
}
impl<'d> Drop for WorldDataSystemUniqueRef<'d> {
    fn drop(&mut self) {
        let mut lock_state = self.lock.lock().unwrap();

        lock_state.system_unique = false;
    }
}
impl<'d> Deref for WorldDataSystemUniqueRef<'d> {
    type Target = WorldDataStorage;

    fn deref(&self) -> &Self::Target {
        // Safety. Locks and self lifetimes guarantee synchronization.
        unsafe { &*self.data }
    }
}
impl<'d> DerefMut for WorldDataSystemUniqueRef<'d> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety. Locks and self lifetimes guarantee synchronization.
        unsafe { &mut *self.data }
    }
}