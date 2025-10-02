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

pub struct WorldDataUnsafeMut<'w> {
    world: &'w mut WorldDataUnsafeFfi,
    types: &'w TypesRegistryCache,
}

impl<'w> WorldDataUnsafeMut<'w> {
    pub fn new(world: &'w mut WorldDataUnsafeFfi, types: &'w TypesRegistryCache) -> Self {
        Self {
            world,
            types,
        }
    }

    pub fn resources<'r>(&'r self) -> ResourcesHolderUnsafeRef<'r> where 'w : 'r { ResourcesHolderUnsafeRef::new(&self.world.res, self.types) }
    pub fn resources_mut<'r>(&'r mut self) -> ResourcesHolderUnsafeRef<'r> where 'w : 'r { ResourcesHolderUnsafeRef::new(&mut self.world.res, self.types) }
    pub fn entities<'r>(&'r self) -> EntitiesHolderUnsafeRef<'r> where 'w : 'r { EntitiesHolderUnsafeRef::new(&self.world.ent, self.types) }
    pub fn entities_mut<'r>(&'r mut self) -> EntitiesHolderUnsafeRef<'r> where 'w : 'r { EntitiesHolderUnsafeRef::new(&mut self.world.ent, self.types) }
    pub fn events<'r>(&'r self) -> EventsHolderUnsafeRef<'r> where 'w : 'r { EventsHolderUnsafeRef::new(&self.world.evt, self.types) }
    pub fn as_tuple_mut<'r>(&'r mut self) -> ! where 'w : 'r { todo!() }

    pub fn as_mut(&'w mut self) -> WorldDataUnsafeMut<'w> {
        WorldDataUnsafeMut {
            world: self.world,
            types: self.types,
        }
    }

    pub fn into_safe(self) -> WorldDataMut<'w> {
        WorldDataMut {
            world: self,
        }
    }

    pub fn from_safe(world: WorldDataMut<'w>) -> Self {
        world.world
    }
}

//

pub struct WorldDataUnsafeRef<'w> {
    world: &'w WorldDataUnsafeFfi,
    types: &'w TypesRegistryCache,
}

impl<'w> WorldDataUnsafeRef<'w> {
    pub fn new(world: &'w WorldDataUnsafeFfi, types: &'w TypesRegistryCache) -> Self {
        Self {
            world,
            types,
        }
    }

    pub fn resources<'r>(&'r self) -> ResourcesHolderUnsafeRef<'r> where 'w : 'r { ResourcesHolderUnsafeRef::new(&self.world.res, self.types) }
    pub fn entities<'r>(&'r self) -> EntitiesHolderUnsafeRef<'r> where 'w : 'r { EntitiesHolderUnsafeRef::new(&self.world.ent, self.types) }
    pub fn events<'r>(&'r self) -> EventsHolderUnsafeRef<'r> where 'w : 'r { EventsHolderUnsafeRef::new(&self.world.evt, self.types) }

    pub fn into_safe(self) -> WorldDataRef<'w> {
        WorldDataRef {
            world: self,
        }
    }

    pub unsafe fn into_safe_mut(self) -> WorldDataMut<'w> {
        WorldDataUnsafeMut {
            types: self.types,
            world: self.world
        }.into_safe()
    }

    pub fn from_safe(world: WorldDataRef<'w>) -> Self {
        world.world
    }
}

//

#[repr(transparent)]
pub struct WorldDataMut<'w> {
    world: WorldDataUnsafeMut<'w>,
}

impl<'w> WorldDataMut<'w> {
    pub fn resources<'r>(&'r self) -> ResourcesHolderRef<'r> where 'w : 'r { self.world.resources().into_safe() }
    pub fn resources_mut<'r>(&'r mut self) -> ResourcesHolderMut<'r> where 'w : 'r { self.world.resources_mut().into_safe() }
    pub fn components<'r>(&'r self) -> EntitiesHolderRef<'r> where 'w : 'r { unsafe { self.world.entities().into_safe() } }
    pub fn components_mut<'r>(&'r mut self) -> EntitiesHolderMut<'r> where 'w : 'r { unsafe { self.world.entities_mut().into_safe() } }
    pub fn events<'r>(&'r self) -> EventsHolderRef<'r> where 'w : 'r { self.world.events().into_safe() }
    pub fn as_tuple_mut<'r>(&'r mut self) -> ! where 'w : 'r { todo!() }

    pub fn as_mut(&'w mut self) -> WorldDataMut<'w> {
        WorldDataMut {
            world: self.world.as_mut(),
        }
    }
}

//

#[repr(transparent)]
pub struct WorldDataRef<'w> {
    world: WorldDataUnsafeRef<'w>,
}

impl<'w> WorldDataRef<'w> {
    pub fn resources<'r>(&'r self) -> ResourcesHolderRef<'r> where 'w : 'r { self.world.resources().into_safe() }
    pub fn components<'r>(&'r self) -> EntitiesHolderRef<'r> where 'w : 'r { unsafe { self.world.entities().into_safe() } }
    pub fn events<'r>(&'r self) -> EventsHolderRef<'r> where 'w : 'r { self.world.events().into_safe() }
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
