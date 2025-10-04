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

#[derive(Clone)]
pub struct WorldDataUnsafeRef<'w> {
    world: *mut WorldDataUnsafeFfi,
    types: &'w TypesRegistryCache,
}

impl<'w> WorldDataUnsafeRef<'w> {
    pub fn new(world: *mut WorldDataUnsafeFfi, types: &'w TypesRegistryCache) -> Self {
        Self {
            world,
            types,
        }
    }

    pub fn resources<'r>(&'r self) -> ResourcesHolderUnsafeRef<'r> where 'w : 'r {
        unsafe { ResourcesHolderUnsafeRef::new(&raw mut (*self.world).res, self.types) }
    }
    pub fn entities<'r>(&'r self) -> EntitiesHolderUnsafeRef<'r> where 'w : 'r {
        unsafe { EntitiesHolderUnsafeRef::new(&raw mut (*self.world).ent, self.types) }
    }
    pub fn events<'r>(&'r self) -> EventsHolderUnsafeRef<'r> where 'w : 'r {
        unsafe { EventsHolderUnsafeRef::new(&raw mut (*self.world).evt, self.types) }
    }

    pub unsafe fn ffi(&self) -> *mut WorldDataUnsafeFfi {
        self.world
    }

    pub unsafe fn into_safe_mut(self) -> WorldDataMut<'w> {
        WorldDataMut {
            world: self,
        }
    }

    pub fn from_safe(world: WorldDataMut<'w>) -> Self {
        world.world
    }
}

unsafe impl<'w> Send for WorldDataUnsafeRef<'w> { }
unsafe impl<'w> Sync for WorldDataUnsafeRef<'w> { }

//

#[repr(transparent)]
pub struct WorldDataMut<'w> {
    world: WorldDataUnsafeRef<'w>,
}

impl<'w> WorldDataMut<'w> {
    pub fn resources<'r>(&'r self) -> ResourcesHolderRef<'r>
        where 'w : 'r
    {
        ResourcesHolderRef::new(self.world.resources())
    }
    pub fn resources_mut<'r>(&'r mut self) -> ResourcesHolderMut<'r>
        where 'w : 'r
    {
        ResourcesHolderMut::new(self.world.resources())
    }
    pub fn entities<'r>(&'r self) -> EntitiesHolderRef<'r>
        where 'w : 'r
    {
        EntitiesHolderRef::new(self.world.entities())
    }
    pub fn entities_mut<'r>(&'r mut self) -> EntitiesHolderMut<'r>
        where 'w : 'r
    {
        EntitiesHolderMut::new(self.world.entities())
    }
    pub fn events<'r>(&'r self) -> EventsHolderRef<'r>
        where 'w : 'r
    {
        self.world.events().into_safe()
    }
    pub fn as_tuple_mut<'r>(&'r mut self) -> !
        where 'w : 'r { todo!()
    }

    pub fn as_mut(&'w mut self) -> WorldDataMut<'w> {
        WorldDataMut {
            world: self.world.clone(),
        }
    }
}

//

#[repr(transparent)]
pub struct WorldDataRef<'w> {
    world: WorldDataUnsafeRef<'w>,
}

impl<'w> WorldDataRef<'w> {
    pub fn resources<'r>(&'r self) -> ResourcesHolderRef<'r> where 'w : 'r {
        ResourcesHolderRef::new(self.world.resources())
    }
    pub fn components<'r>(&'r self) -> EntitiesHolderRef<'r> where 'w : 'r {
        EntitiesHolderRef::new(self.world.entities())
    }
    pub fn events<'r>(&'r self) -> EventsHolderRef<'r> where 'w : 'r {
        self.world.events().into_safe()
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
