use fruits_ecs_schedule::WorldBehaviorBuilder;

use fruits_ecs_data::{WorldData, WorldDataUniqueRef};

use crate::world::World;

pub struct WorldBuilder {
    data: WorldData,
    behavior: WorldBehaviorBuilder,
}

impl WorldBuilder {
    pub fn new() -> Self {
        Self {
            behavior: WorldBehaviorBuilder::new(),
            data: WorldData::new(),
        }
    }

    pub fn behavior(&self) -> &WorldBehaviorBuilder {
        &self.behavior
    }

    pub fn behavior_mut(&mut self) -> &mut WorldBehaviorBuilder {
        &mut self.behavior
    }

    pub fn data(&mut self) -> WorldDataUniqueRef {
        self.data.unique()
    }

    pub fn build(self) -> World {
        World::new(self.data, self.behavior.build())
    }
}
