use fruits_ecs_data::WorldDataUnsafe;
use fruits_ecs_system_resource::SystemResourcesHolder;

pub struct SystemInput<'a> {
    pub world_data: &'a WorldDataUnsafe,
    pub system_data: &'a SystemResourcesHolder,
}
