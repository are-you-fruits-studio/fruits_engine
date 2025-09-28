use fruits_ffi::{FfiOpaqueBox, FfiOption};

use crate::{SystemScheduleFfi, TypesRegistryAccessFfi, WorldDataUnsafeFfi};

#[repr(C)]
pub struct AppInitCtxFfi {
    pub world_mut: *mut WorldDataUnsafeFfi,
    pub types_ref: *const TypesRegistryAccessFfi,
    pub systems_mut: *mut SystemScheduleFfi,
    pub native_data_mut: *mut FfiOption<FfiOpaqueBox>,
}