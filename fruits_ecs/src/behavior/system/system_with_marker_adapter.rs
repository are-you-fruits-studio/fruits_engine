use crate::*;

pub struct SystemWithMarkerAdapter<M: 'static> {
    system_with_marker: Box<dyn SystemWithMarker<M>>,
}

impl<M: 'static> SystemWithMarkerAdapter<M> {
    pub fn new(system_with_marker: Box<dyn SystemWithMarker<M>>) -> Self {
        Self {
            system_with_marker,
        }
    }
}

unsafe impl<M: 'static> System for SystemWithMarkerAdapter<M>
where {
    fn fill_data_usage(&self, usage: &mut DataUsage) {
        self.system_with_marker.fill_data_usage(usage)
    }

    unsafe fn execute<'e>(&self, data: &SystemInput<'e>) {
        // Safety. Managed by caller.
        unsafe { self.system_with_marker.execute(data) }
    }

    fn system_name(&self) -> &'static str {
        self.system_with_marker.system_name()
    }
}