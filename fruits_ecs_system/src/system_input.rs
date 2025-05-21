use fruits_ecs_data::WorldDataSystemReservedRef;
use fruits_ecs_system_resource::SystemResourcesHolder;

pub struct SystemInput<'a> {
    pub world_data: WorldDataSystemReservedRef<'a>,
    pub system_data: &'a SystemResourcesHolder,
}
