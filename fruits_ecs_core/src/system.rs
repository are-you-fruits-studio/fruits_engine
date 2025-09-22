use std::ffi::c_void;

use fruits_ffi::{FfiDroppableData, FfiString};

use crate::DataUsage;

#[repr(C)]
pub struct SystemFfi {
    system_name: FfiString,
    data_usage: DataUsage,
    system_data: FfiDroppableData,
    execute_fn: unsafe extern "C" fn(*const c_void),
}

impl SystemFfi {
    pub fn new<F: Fn()>(f: F) -> Self {
        unsafe extern "C" fn ffi_execute<D: Fn()>(system_data: *const c_void) {
            unsafe {
                (&*(system_data as *const D))();
            }
        }

        Self {
            // todo: use actual data usage.
            data_usage: DataUsage::global_mut(),
            execute_fn: ffi_execute::<F>,
            system_data: FfiDroppableData::new(f),
            system_name: FfiString::from_string(String::from(std::any::type_name::<F>())),
        }
    }

    pub fn data_usage(&self) -> &DataUsage {
        &self.data_usage
    }
    // todo:

    // /// # Safety
    // /// 
    // /// Should be managed by system scheduler and data usage.
    // pub unsafe fn execute<'e>(&self, data: &SystemInput<'e>) {

    // }

    pub fn execute(&self) {
        unsafe {
            (self.execute_fn)(self.system_data.as_ptr())
        }
    }

    pub fn system_name(&self) -> &str {
        self.system_name.as_str()
    }
}