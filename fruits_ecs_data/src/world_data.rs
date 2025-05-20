use std::sync::Arc;

use fruits_ecs_component::EntitiesComponentsHolderRef;

use crate::ResourcesHolder;

struct WorldData {
    resources: ResourcesHolder,
    entities_components: EntitiesComponentsHolderRef,
}

#[derive(Clone)]
pub struct WorldDataRef {
    data: Arc<WorldData>,
}
impl WorldDataRef {
    pub fn new() -> Self {
        Self {
            data: Arc::new(WorldData {
                resources: ResourcesHolder::new(),
                entities_components: EntitiesComponentsHolderRef::new(),
            }),
        }
    }

    pub fn resources(&self) -> &ResourcesHolder {
        &self.data.resources
    }

    pub fn entities_components(&self) -> &EntitiesComponentsHolderRef {
        &self.data.entities_components
    }
}