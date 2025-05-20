use fruits_ecs_schedule::WorldBehaviorBuilder;

use fruits_ecs_data::WorldDataRef;

use crate::world::World;

pub struct WorldBuilder {
    data: WorldDataRef,
    behavior: WorldBehaviorBuilder,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            behavior: WorldBehaviorBuilder::new(),
            data: WorldDataRef::new(),
        }
    }

    pub fn behavior(&self) -> &WorldBehaviorBuilder {
        &self.behavior
    }

    pub fn behavior_mut(&mut self) -> &mut WorldBehaviorBuilder {
        &mut self.behavior
    }

    pub fn data(&self) -> &WorldDataRef {
        &self.data
    }

    pub fn build(self) -> World {
        World::new(self.data, self.behavior.build())
    }
}
