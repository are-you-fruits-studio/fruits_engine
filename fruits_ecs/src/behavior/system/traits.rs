use crate::*;

pub unsafe trait System : 'static + Send + Sync {
    fn fill_data_usage(&self, usage: &mut DataUsage);
    /// Safety. Should be managed by system scheduler and data usage.
    unsafe fn execute<'e>(&self, data: &SystemInput<'e>);
    fn system_name(&self) -> &'static str;
}

pub unsafe trait SystemParam {
    type Item<'e> : 'e + SystemParam;

    fn fill_data_usage(usage: &mut DataUsage);
    /// Safety. Should be managed by system scheduler and data usage.
    unsafe fn new<'a>(input: &'a SystemInput<'a>) -> Result<Self::Item<'a>, &'static str>;
}

pub unsafe trait SystemWithMarker<M: 'static> : 'static + Send + Sync {
    fn fill_data_usage(&self, usage: &mut DataUsage);
    /// Safety. Should be managed by system scheduler and data usage.
    unsafe fn execute<'e>(&self, data: &SystemInput<'e>);
    fn into_system_generic(self) -> Box<dyn System>;
    fn system_name(&self) -> &'static str;
}