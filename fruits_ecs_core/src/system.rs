use std::ffi::c_void;

use fruits_ffi::{FfiOpaqueBox, FfiOption, FfiString};

use crate::{DataUsage, ResourcesHolderUnsafeFfi, TypesRegistryAccessFfi};

#[repr(C)]
pub struct SystemFfi {
    system_name: FfiString,
    data_usage: DataUsage,
    system_data: FfiOpaqueBox,
    execute_fn: unsafe extern "C" fn(*const c_void, SystemCtxFfi),
}

impl SystemFfi {
    pub fn new<F: Fn(SystemCtxFfi)>(f: F) -> Self {
        unsafe extern "C" fn ffi_execute<D: Fn(SystemCtxFfi)>(system_data: *const c_void, ctx: SystemCtxFfi) {
            unsafe {
                (&*(system_data as *const D))(ctx);
            }
        }

        Self {
            // todo: use actual data usage.
            data_usage: DataUsage::global_mut(),
            execute_fn: ffi_execute::<F>,
            system_data: FfiOpaqueBox::new(f),
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

    pub fn execute(&self, ctx: SystemCtxFfi) {
        unsafe {
            (self.execute_fn)(self.system_data.as_ptr(), ctx)
        }
    }

    pub fn system_name(&self) -> &str {
        self.system_name.as_str()
    }
}

// todo
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SystemCtxFfi {
    pub res_mut: *mut ResourcesHolderUnsafeFfi,
    pub types_ref: *const TypesRegistryAccessFfi,
    pub native_data_mut: *mut FfiOption<FfiOpaqueBox>,
}