use crate::{ResourcesHolderUnsafeFfi, TypesRegistryAccessFfi};

#[repr(C)]
pub struct AppInitCtxFfi {
    pub res_ref: *mut ResourcesHolderUnsafeFfi,
    pub types_ref: *const TypesRegistryAccessFfi,
}