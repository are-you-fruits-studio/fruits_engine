use crate::{Schedule, TypesRegistryAccessFfi, TypesRegistryCache, WorldBehaviorBuilderFfi, WorldBehaviorBuilderMut, WorldBehaviorFfi, WorldDataMut, WorldDataUnsafeFfi};

#[repr(C)]
pub struct WorldBuilderUnsafeFfi {
    data: WorldDataUnsafeFfi,
    behavior: WorldBehaviorBuilderFfi,
}

impl WorldBuilderUnsafeFfi {
    pub fn new(types: TypesRegistryAccessFfi) -> Self {
        Self {
            data: WorldDataUnsafeFfi::new(types.clone()),
            behavior: WorldBehaviorBuilderFfi::new(types),
        }
    }

    pub fn build(self) -> WorldUnsafeFfi {
        WorldUnsafeFfi {
            data: self.data,
            behavior: self.behavior.build(),
        }
    }
}

#[repr(C)]
pub struct WorldUnsafeFfi {
    data: WorldDataUnsafeFfi,
    behavior: WorldBehaviorFfi,
}

impl WorldUnsafeFfi {
    pub fn execute_iteration(&mut self, schedule: Schedule) {
        self.behavior.get_mut(schedule).execute_iteration(&raw mut self.data);
    }
}

//

pub struct WorldBuilderMut<'w> {
    world: &'w mut WorldBuilderUnsafeFfi,
    types: &'w TypesRegistryCache,
}

impl<'w> WorldBuilderMut<'w> {
    pub fn new(world: &'w mut WorldBuilderUnsafeFfi, types: &'w TypesRegistryCache) -> Self {
        Self {
            world,
            types,
        }
    }

    pub fn data<'r>(&'r mut self) -> WorldDataMut<'r>
        where 'w: 'r
    {
        unsafe { WorldDataMut::new(&raw mut self.world.data, self.types) }
    }

    pub fn behavior<'r>(&'r mut self) -> WorldBehaviorBuilderMut<'r>
        where 'w: 'r
    {
        WorldBehaviorBuilderMut::new(&mut self.world.behavior, self.types)
    }
}
