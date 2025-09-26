use fruits_ffi::{FfiOpaqueBox, FfiOption};

use crate::{ResourcesHolderUnsafeFfi, SystemScheduleFfi, TypesRegistryAccessFfi};

#[repr(C)]
pub struct AppInitCtxFfi {
    pub res_mut: *mut ResourcesHolderUnsafeFfi,
    pub types_ref: *const TypesRegistryAccessFfi,
    pub systems_mut: *mut SystemScheduleFfi,
    pub native_data_mut: *mut FfiOption<FfiOpaqueBox>,
}