use crate::*;

pub struct World {
    data: WorldData,
    behavior: WorldBehavior,
}

impl World {
    pub fn new(data: WorldData, behavior: WorldBehavior) -> Self {
        Self {
            data,
            behavior,
        }
    }

    pub fn execute_iteration(&mut self, schedule: Schedule) {
        // println!("ayf: iteration {:?} start", schedule);
        self.behavior.get(schedule).execute_iteration(&mut self.data);
        // println!("ayf: iteration {:?} end", schedule);
    }

    pub fn data(&mut self) -> &mut WorldData {
        &mut self.data
    }
}

#[derive(Default)]
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

    pub fn data(&self) -> &WorldData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut WorldData {
        &mut self.data
    }

    pub fn build(self) -> World {
        World::new(self.data, self.behavior.build())
    }
}