use fruits_ffi::FfiVec;

use crate::{SystemCtxFfi, SystemFfi};

#[repr(C)]
pub struct SystemScheduleFfi {
    systems: FfiVec<SystemFfi>,
}

impl SystemScheduleFfi {
    pub fn new() -> Self {
        Self {
            systems: FfiVec::new(),
        }
    }

    pub fn insert(&mut self, system: SystemFfi) {
        self.systems.push(system);
    }

    pub fn execute(&mut self, ctx: SystemCtxFfi) {
        for system in &self.systems {
            system.execute(ctx);
        }
    }
}