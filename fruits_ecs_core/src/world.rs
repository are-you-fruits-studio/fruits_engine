use crate::{evt_ffi::EventsHolderUnsafeFfi, evt_safe::EventsHolderRef, evt_unsafe::EventsHolderUnsafeRef, res_ffi::ResourcesHolderUnsafeFfi, res_safe::{ResourcesHolderMut, ResourcesHolderRef}, res_unsafe::{ResourcesHolderUnsafeMut, ResourcesHolderUnsafeRef}, TypesRegistryAccessFfi, TypesRegistryCache};

// todo: access modifiers
// todo: unsafe modifiers

#[repr(C)]
pub struct WorldDataUnsafeFfi {
    pub res: ResourcesHolderUnsafeFfi,
    pub evt: EventsHolderUnsafeFfi,
}

impl WorldDataUnsafeFfi {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        Self {
            res: ResourcesHolderUnsafeFfi::new(types.clone()),
            evt: EventsHolderUnsafeFfi::new(types)
        }
    }
}

//

pub struct WorldDataUnsafeMut<'w> {
    world: &'w mut WorldDataUnsafeFfi,
    types: TypesRegistryCache,
}

impl<'w> WorldDataUnsafeMut<'w> {
    pub fn new(world: &'w mut WorldDataUnsafeFfi, types: TypesRegistryCache) -> Self {
        Self {
            world,
            types,
        }
    }

    pub fn resources<'r>(&'r self) -> ResourcesHolderUnsafeRef<'r> where 'w : 'r { ResourcesHolderUnsafeRef::new(&self.world.res, self.types.clone()) }
    pub fn resources_mut<'r>(&'r mut self) -> ResourcesHolderUnsafeMut<'r> where 'w : 'r { ResourcesHolderUnsafeMut::new(&mut self.world.res, self.types.clone()) }
    pub fn components<'r>(&'r self) -> ! where 'w : 'r { todo!() }
    pub fn components_mut<'r>(&'r mut self) -> ! where 'w : 'r { todo!() }
    pub fn events<'r>(&'r self) -> EventsHolderUnsafeRef<'r> where 'w : 'r { EventsHolderUnsafeRef::new(&self.world.evt, self.types.clone()) }
    pub fn as_tuple_mut<'r>(&'r mut self) -> ! where 'w : 'r { todo!() }

    pub fn as_mut(&'w mut self) -> WorldDataUnsafeMut<'w> {
        WorldDataUnsafeMut {
            world: self.world,
            types: self.types.clone(),
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

#[repr(transparent)]
pub struct WorldDataMut<'w> {
    world: WorldDataUnsafeMut<'w>,
}

impl<'w> WorldDataMut<'w> {
    pub fn resources<'r>(&'r self) -> ResourcesHolderRef<'r> where 'w : 'r { self.world.resources().into_safe() }
    pub fn resources_mut<'r>(&'r mut self) -> ResourcesHolderMut<'r> where 'w : 'r { self.world.resources_mut().into_safe() }
    pub fn components<'r>(&'r self) -> ! where 'w : 'r { todo!() }
    pub fn components_mut<'r>(&'r mut self) -> ! where 'w : 'r { todo!() }
    pub fn events<'r>(&'r self) -> EventsHolderRef<'r> where 'w : 'r { self.world.events().into_safe() }
    pub fn as_tuple_mut<'r>(&'r mut self) -> ! where 'w : 'r { todo!() }

    pub fn as_mut(&'w mut self) -> WorldDataMut<'w> {
        WorldDataMut {
            world: self.world.as_mut(),
        }
    }
}


// todo:
// what lib needs:
// - structural
//     - data
//         - safe resources
//         - safe events
//         - safe components
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
