use fruits_ecs_data_usage::DataUsage;

use crate::{system::System, system_input::SystemInput};

pub unsafe trait SystemWithMarker<M: 'static> : 'static + Send + Sync {
    fn fill_data_usage(&self, usage: &mut DataUsage);
    /// Safety. Should be managed by system scheduler and data usage.
    unsafe fn execute<'e>(&self, data: &SystemInput<'e>);
    fn into_system_generic(self) -> Box<dyn System>;
    fn system_name(&self) -> &'static str;
}