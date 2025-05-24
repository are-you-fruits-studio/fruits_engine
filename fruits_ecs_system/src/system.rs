use fruits_ecs_data_usage::DataUsage;

use crate::system_input::SystemInput;

pub unsafe trait System : 'static + Send + Sync {
    fn fill_data_usage(&self, usage: &mut DataUsage);
    /// Safety. Should be managed by system scheduler and data usage.
    unsafe fn execute<'e>(&self, data: &SystemInput<'e>);
    fn system_name(&self) -> &'static str;
}