use fruits_ecs_component::EntitiesComponentsUniqueGuard;
use fruits_ecs_data::ResourcesHolder;
use fruits_ecs_data_usage::*;
use fruits_ecs_system::{SystemInput, SystemParam};

pub struct ExclusiveWorldAccess<'w> {
    resources: &'w ResourcesHolder,
    entities_components: EntitiesComponentsUniqueGuard,
}

impl<'w> ExclusiveWorldAccess<'w> {
    pub fn resources(&self) -> &ResourcesHolder {
        &self.resources
    }

    pub fn entities_components(&self) -> &EntitiesComponentsUniqueGuard {
        &self.entities_components
    }

    pub fn entities_components_mut(&mut self) -> &mut EntitiesComponentsUniqueGuard {
        &mut self.entities_components
    }
}

unsafe impl<'b> SystemParam for ExclusiveWorldAccess<'b> {
    type Item<'e> = ExclusiveWorldAccess<'e>;

    fn fill_data_usage(usage: &mut DataUsage) {
        usage.add_all_mut();
    }

    fn new<'a>(input: &'a SystemInput<'a>) -> Option<Self::Item<'a>> {
        let resources = input.world_data.resources();
        let entities_components = input.world_data.entities_components().unique()?;

        Some(ExclusiveWorldAccess {
            resources,
            entities_components
        })
    }
}