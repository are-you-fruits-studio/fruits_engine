use fruits_ecs_data_usage::DataUsage;

use crate::system_input::SystemInput;

pub unsafe trait SystemParam {
    type Item<'e> : 'e + SystemParam;

    fn fill_data_usage(usage: &mut DataUsage);
    fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str>;
}