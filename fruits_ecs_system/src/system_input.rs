use fruits_ecs_data::WorldDataRef;
use fruits_ecs_system_resource::SystemResourcesHolder;

#[derive(Clone)]
pub struct SystemInput<'a> {
    pub world_data: WorldDataRef,
    pub system_data: &'a SystemResourcesHolder,
}
