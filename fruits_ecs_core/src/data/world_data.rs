use crate::*;

// todo: access modifiers
// todo: unsafe modifiers

#[repr(C)]
pub struct WorldDataUnsafeFfi {
    pub res: ResourcesHolderUnsafeFfi,
    pub evt: EventsHolderUnsafeFfi,
    pub ent: EntitiesHolderUnsafeFfi,
}

impl WorldDataUnsafeFfi {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        Self {
            res: ResourcesHolderUnsafeFfi::new(types.clone()),
            evt: EventsHolderUnsafeFfi::new(types.clone()),
            ent: EntitiesHolderUnsafeFfi::new(types),
        }
    }
}

//

pub struct WorldDataMut<'w> {
    world: *mut WorldDataUnsafeFfi,
    types: &'w TypesRegistryCache,
}

impl<'w> WorldDataMut<'w> {
    pub unsafe fn new(world: *mut WorldDataUnsafeFfi, types: &'w TypesRegistryCache) -> Self {
        Self {
            world,
            types,
        }
    }

    pub fn resources<'r>(&'r self) -> ResourcesHolderRef<'r> where 'w : 'r {
        unsafe { ResourcesHolderRef::new(&(&*self.world).res, self.types) }
    }
    pub fn resources_mut<'r>(&'r mut self) -> ResourcesHolderMut<'r> where 'w : 'r {
        unsafe { ResourcesHolderMut::new(&mut (&mut *self.world).res, self.types) }
    }
    pub fn entities<'r>(&'r self) -> EntitiesHolderRef<'r> where 'w : 'r {
        unsafe { EntitiesHolderRef::new(&(&*self.world).ent, self.types) }
    }
    pub fn entities_mut<'r>(&'r mut self) -> EntitiesHolderMut<'r> where 'w : 'r {
        unsafe { EntitiesHolderMut::new(&mut (&mut *self.world).ent, self.types) }
    }
    pub fn events<'r>(&'r self) -> EventsHolderRef<'r> where 'w : 'r {
        unsafe { EventsHolderRef::new(&(&*self.world).evt, self.types) }
    }
    pub fn events_mut<'r>(&'r mut self) -> EventsHolderMut<'r> where 'w : 'r {
        unsafe { EventsHolderMut::new(&mut (&mut *self.world).evt, self.types) }
    }

    pub unsafe fn ffi(&self) -> *mut WorldDataUnsafeFfi {
        self.world
    }

    pub fn as_tuple_mut<'r>(&'r mut self) -> (ResourcesHolderMut<'r>, EntitiesHolderMut<'r>, EventsHolderMut<'r>) where 'w : 'r {
        unsafe {
            let world = &mut *self.world;
            (
                ResourcesHolderMut::new(&mut world.res, self.types),
                EntitiesHolderMut::new(&mut world.ent, self.types),
                EventsHolderMut::new(&mut world.evt, self.types),
            )
        }
    }

    pub fn as_mut(&'w mut self) -> WorldDataMut<'w> {
        WorldDataMut {
            world: self.world,
            types: self.types,
        }
    }
}

unsafe impl<'w> Send for WorldDataMut<'w> { }
unsafe impl<'w> Sync for WorldDataMut<'w> { }

//

#[derive(Copy, Clone)]
pub struct WorldDataRef<'w> {
    world: *mut WorldDataUnsafeFfi,
    types: &'w TypesRegistryCache,
}

impl<'w> WorldDataRef<'w> {
    pub unsafe fn new(world: *mut WorldDataUnsafeFfi, types: &'w TypesRegistryCache) -> Self {
        Self {
            world,
            types,
        }
    }

    pub fn resources(self) -> ResourcesHolderRef<'w> {
        unsafe { ResourcesHolderRef::new(&(&*self.world).res, self.types) }
    }
    pub fn entities(self) -> EntitiesHolderRef<'w> {
        unsafe { EntitiesHolderRef::new(&(&*self.world).ent, self.types) }
    }
    pub fn events(self) -> EventsHolderRef<'w> {
        unsafe { EventsHolderRef::new(&(&*self.world).evt, self.types) }
    }

    pub unsafe fn ffi(&self) -> *mut WorldDataUnsafeFfi {
        self.world
    }

    pub unsafe fn into_mut(self) -> WorldDataMut<'w> {
        WorldDataMut {
            world: self.world,
            types: self.types,
        }
    }
}


// todo:
// what lib needs:
// - structural
//     - data
//         + safe resources
//         + safe events
//         + safe entities
//         - Res
//         - ResMut
//         - Evt
//         - EvtMut
//         - Local
//         - Query
//         - EntitiesData
//         - ExclusiveWorldAccess
//     - behavior
//         - systems registering
//         - systems ordering
// - specific
//     - specific resources
//     - specific components
//     - specific events
