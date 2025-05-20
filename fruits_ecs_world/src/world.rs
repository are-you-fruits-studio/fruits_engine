use std::sync::{Arc, RwLock, RwLockReadGuard};

use fruits_ecs_data::WorldDataRef;
use fruits_ecs_schedule::{Schedule, WorldBehavior};

pub struct World {
    data: WorldDataRef,
    behavior: WorldBehavior,
}

impl World {
    pub fn new(data: WorldDataRef, behavior: WorldBehavior) -> Self {
        Self {
            data,
            behavior,
        }
    }

    pub fn execute_iteration(&self, schedule: Schedule) {
        self.behavior.get(schedule).execute_iteration(&self.data);
    }

    pub fn data(&self) -> &WorldDataRef {
        &self.data
    }
}
