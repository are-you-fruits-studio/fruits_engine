use fruits_ecs_data::WorldData;
use fruits_ecs_schedule::{Schedule, WorldBehavior};

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
        self.behavior.get(schedule).execute_iteration(self.data.system_reserved());
    }

    pub fn data(&mut self) -> &mut WorldData {
        &mut self.data
    }
}
