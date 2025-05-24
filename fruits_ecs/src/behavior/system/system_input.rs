use crate::*;

pub struct SystemInput<'a> {
    pub world_data: &'a WorldDataUnsafe,
    pub system_data: &'a SystemResourcesHolder,
}
