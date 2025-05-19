use std::sync::Arc;

use fruits_ecs_component::WorldEntitiesComponents;

use crate::ResourcesHolder;

//

pub struct WorldDataNew {
    resources: ResourcesHolder,
}

#[derive(Clone)]
pub struct WorldDataRefNew {
    data: Arc<WorldDataNew>,
}
impl WorldDataRefNew {
    pub fn resources(&self) -> &ResourcesHolder {
        &self.data.resources
    }
}

//

pub struct WorldData {
    resources: ResourcesHolder,
    entities_components: WorldEntitiesComponents,
}

impl WorldData {
    pub fn new() -> Self {
        Self {
            resources: ResourcesHolder::new(),
            entities_components: WorldEntitiesComponents::new(),
        }
    }

    pub fn resources(&self) -> &ResourcesHolder {
        &self.resources
    }

    pub fn entities_components(&self) -> &WorldEntitiesComponents {
        &self.entities_components
    }

    pub fn entities_components_mut(&mut self) -> &mut WorldEntitiesComponents {
        &mut self.entities_components
    }
}